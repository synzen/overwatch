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
    pub expected_arrival_time: Option<String>,
    pub minutes_until_arrival: Option<i64>,
    pub stop_id: String,
    pub route_label: String,
}

#[derive(Serialize, Deserialize)]
pub struct TransitArrivalsData {
    pub arrivals: Vec<StopResponseDataArrival>,
}

#[derive(Serialize, Deserialize)]
pub struct TransitArrivalsResponse {
    pub data: TransitArrivalsData,
}

#[derive(Validate, Deserialize)]
pub struct GetTransitStopPayload {
    #[validate(length(min = 1, message = "Must be at least 1 character"))]
    pub stop_ids: String,
}

pub async fn get_transit_arrival_times(
    State(state): State<AppState>,
    ValidatedQuery(payload): ValidatedQuery<GetTransitStopPayload>,
) -> Result<Response, AppError> {
    let stop_ids = payload.stop_ids.split(",").collect::<Vec<&str>>();

    match state
        .mta_client
        .fetch_multiple_stop_arrivals(stop_ids)
        .await
        .map_err(|e| {
            error!("Failed to fetch stop info: {}", e);
            AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
        }) {
        Ok(v) => {
            let response_json = TransitArrivalsResponse {
                data: TransitArrivalsData {
                    arrivals: v
                        .iter()
                        .map(|s| StopResponseDataArrival {
                            stop_id: s.stop_id.clone(),
                            expected_arrival_time: s.expected_arrival_time.clone(),
                            minutes_until_arrival: s.minutes_until_arrival,
                            route_label: s.route_label.clone(),
                        })
                        .collect::<Vec<StopResponseDataArrival>>(),
                },
            };

            Ok((StatusCode::OK, Json(response_json)).into_response())
        }
        Err(e) => Err(e),
    }
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
                                PublishedLineName: "A".to_string(),
                            },
                        }]),
                    }]),
                },
            },
        };

        let mock1 = mock_server
            .mock("GET", "/api/siri/stop-monitoring.json")
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&mock_response).unwrap())
            .match_query(mockito::Matcher::Regex(".*123.*".to_string()))
            .create_async()
            .await;

        let future_date2 = Utc::now() + Duration::minutes(10);

        let mock_response2 = GetStopInfoResponse {
            Siri: Siri {
                ServiceDelivery: ServiceDelivery {
                    StopMonitoringDelivery: Vec::from([StopMonitoringDelivery {
                        MonitoredStopVisit: Vec::from([MonitoredStopVisit {
                            MonitoredVehicleJourney: MonitoredVehicleJourney {
                                MonitoredCall: MonitoredCall {
                                    ExpectedArrivalTime: Some(future_date2.to_rfc3339()),
                                },
                                PublishedLineName: "B".to_string(),
                            },
                        }]),
                    }]),
                },
            },
        };
        let mock2 = mock_server
            .mock("GET", "/api/siri/stop-monitoring.json")
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&mock_response2).unwrap())
            .match_query(mockito::Matcher::Regex(".*abc.*".to_string()))
            .create_async()
            .await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/transit-arrival-times?stop_ids=123,abc")
                    .header("content-type", "application/json")
                    .body(Body::empty())
                    .expect("Failed to create request"),
            )
            .await
            .expect("Failed to get response");

        assert_eq!(response.status(), StatusCode::OK);

        mock1.assert();
        mock2.assert();

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: TransitArrivalsResponse = serde_json::from_slice(&body)
            .expect("Failed to deserialize response body into TransitArrivalsResponse struct");

        assert_eq!(body.data.arrivals.len(), 2);

        // test 0th element is 123
        assert_eq!(body.data.arrivals[0].stop_id, "123");
        assert_eq!(body.data.arrivals[0].route_label, "A");
        assert_eq!(
            body.data.arrivals[0].expected_arrival_time,
            Some(future_date.to_rfc3339())
        );
        assert_eq!(body.data.arrivals[0].minutes_until_arrival < Some(4), true);
        assert_eq!(body.data.arrivals[0].minutes_until_arrival > Some(0), true);

        // test 1st element is abc
        assert_eq!(body.data.arrivals[1].stop_id, "abc");
        assert_eq!(body.data.arrivals[1].route_label, "B");
        assert_eq!(
            body.data.arrivals[1].expected_arrival_time,
            Some(future_date2.to_rfc3339())
        );
        assert_eq!(body.data.arrivals[1].minutes_until_arrival < Some(12), true);
        assert_eq!(body.data.arrivals[1].minutes_until_arrival > Some(8), true);
    }
}
