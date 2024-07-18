use crate::{types::app_state::AppState, utils::app_error::AppError};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Serialize, Deserialize)]
pub struct StopResponse {
    pub expected_arrival_time: String,
    pub minutes_until_arrival: i64,
}

pub async fn get_transit_stop(State(state): State<AppState>) -> Result<Response, AppError> {
    let result = state
        .mta_client
        .fetch_stop_info()
        .await
        .map_err(|e| {
            error!("Failed to fetch stop info: {}", e);
            AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
        })?
        .map(|s| {
            (
                StatusCode::OK,
                Json(StopResponse {
                    expected_arrival_time: s.expected_arrival_time,
                    minutes_until_arrival: s.minutes_until_arrival,
                }),
            )
                .into_response()
        })
        .ok_or_else(|| {
            error!("No expected arrival time found");
            AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
        });

    result
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
        app::gen_app,
        types::response_formats::{
            GetStopInfoResponse, MonitoredCall, MonitoredStopVisit, MonitoredVehicleJourney,
            ServiceDelivery, Siri, StopMonitoringDelivery,
        },
    };

    use super::*;

    #[tokio::test]
    async fn get_response() {
        let mut mock_server = mockito::Server::new_async().await;

        let app = gen_app(mock_server.url().as_str(), "key");

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
                    .uri("/transit-stops/1")
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

        assert_eq!(body.minutes_until_arrival > 0, true);
        assert_eq!(body.minutes_until_arrival < 4, true);
        assert_eq!(body.expected_arrival_time, future_date.to_rfc3339());
    }
}
