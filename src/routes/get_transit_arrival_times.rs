use crate::{
    types::app_state::AppState,
    utils::{app_error::AppError, validated_query::ValidatedQuery},
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use validator::Validate;

#[derive(Serialize, Deserialize)]
pub struct StopResponseDataArrival {
    pub expected_arrival_time: String,
    pub minutes_until_arrival: i64,
}

#[derive(Serialize, Deserialize)]
pub struct StopResponseData {
    pub arrival: Option<StopResponseDataArrival>,
}

#[derive(Serialize, Deserialize)]
pub struct StopResponse {
    pub data: StopResponseData,
}

#[derive(Validate, Deserialize)]
pub struct GetTransitStopPayload {
    #[validate(length(min = 1, message = "Must be at least 1 character"))]
    pub stop_id: String,
}

pub async fn get_transit_arrival_times(
    State(state): State<AppState>,
    ValidatedQuery(payload): ValidatedQuery<GetTransitStopPayload>,
) -> Result<Response, AppError> {
    let result = match state
        .mta_client
        .fetch_stop_info(&payload.stop_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch stop info: {}", e);
            AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
        })?
        .map(|s| {
            (
                StatusCode::OK,
                Json(StopResponse {
                    data: StopResponseData {
                        arrival: Some(StopResponseDataArrival {
                            expected_arrival_time: s.expected_arrival_time,
                            minutes_until_arrival: s.minutes_until_arrival,
                        }),
                    },
                }),
            )
                .into_response()
        }) {
        Some(r) => r,
        None => (
            StatusCode::OK,
            Json(StopResponse {
                data: StopResponseData { arrival: None },
            }),
        )
            .into_response(),
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use chrono::{Duration, Utc};
    use tower::ServiceExt;

    use crate::{
        app::{gen_app, AppConfig},
        types::response_formats::{
            GetStopInfoResponse, MonitoredCall, MonitoredStopVisit, MonitoredVehicleJourney,
            ServiceDelivery, Siri, StopMonitoringDelivery,
        },
    };

    use super::*;

    #[tokio::test]
    async fn get_response() {
        let mut mock_server = mockito::Server::new_async().await;

        let app = gen_app(AppConfig {
            mta_host: mock_server.url(),
            mta_key: "key".to_string(),
            tomtom_key: "key".to_string(),
            tomtom_host: "host".to_string(),
            auth_key: None,
        });

        let future_date = Utc::now() + Duration::minutes(2);
        let mock_response = GetStopInfoResponse {
            Siri: Siri {
                ServiceDelivery: ServiceDelivery {
                    StopMonitoringDelivery: Vec::from([StopMonitoringDelivery {
                        MonitoredStopVisit: Vec::from([MonitoredStopVisit {
                            MonitoredVehicleJourney: MonitoredVehicleJourney {
                                MonitoredCall: MonitoredCall {
                                    ExpectedArrivalTime: Some(future_date.to_rfc3339()),
                                },
                            },
                        }]),
                    }]),
                },
            },
        };

        let mock_server = mock_server
            .mock("GET", "/api/siri/stop-monitoring.json")
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::to_string(&mock_response).expect("Failed to serialize test response"),
            )
            .match_query(mockito::Matcher::Regex(".*".to_string()))
            .create_async()
            .await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/transit-arrival-times?stop_id=123")
                    .header("content-type", "application/json")
                    .body(Body::empty())
                    .expect("Failed to create request"),
            )
            .await
            .expect("Failed to get response");

        assert_eq!(response.status(), StatusCode::OK);

        mock_server.assert();

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: StopResponse = serde_json::from_slice(&body)
            .expect("Failed to deserialize response body into StopResponse struct");

        match &body.data.arrival {
            Some(arrival) => {
                assert_eq!(arrival.minutes_until_arrival > 0, true);
                assert_eq!(arrival.minutes_until_arrival < 4, true);
                assert_eq!(arrival.expected_arrival_time, future_date.to_rfc3339());
            }
            None => panic!("Expected arrival time not found"),
        }
    }
}
