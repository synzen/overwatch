use crate::{
    types::app_state::AppState,
    utils::{app_error::AppError, mta_client::MtaClientError, validated_query::ValidatedQuery},
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

#[derive(Validate, Deserialize)]
pub struct GetTransitStopsForRoute {
    #[validate(length(min = 1, message = "Must be at least 1 character"))]
    pub route_id: String,
}

#[derive(Serialize, Deserialize)]

pub struct GetTransitStopsForRouteResponseGroupStop {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetTransitStopsForRouteResponseGroup {
    pub id: String,
    pub name: String,
    pub stops: Vec<GetTransitStopsForRouteResponseGroupStop>,
}

#[derive(Serialize, Deserialize)]
pub struct GetTransitStopsForRouteResponseData {
    pub groups: Vec<GetTransitStopsForRouteResponseGroup>,
}

#[derive(Serialize, Deserialize)]
pub struct GetTransitStopsForRouteResponse {
    pub data: GetTransitStopsForRouteResponseData,
}

pub async fn get_transit_stops_for_route(
    State(state): State<AppState>,
    ValidatedQuery(payload): ValidatedQuery<GetTransitStopsForRoute>,
) -> Result<Response, AppError> {
    let groups = state
        .mta_client
        .get_stops_for_route(payload.route_id)
        .await
        .map_err(|e| match e {
            MtaClientError::ResourceNotFound => {
                AppError::new(StatusCode::NOT_FOUND, "Route does not exist")
            }
            _ => {
                error!("Failed to fetch stops for route: {}", e);
                AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
        })?
        .groups
        .iter()
        .map(|s| GetTransitStopsForRouteResponseGroup {
            id: s.id.clone(),
            name: s.name.clone(),
            stops: s
                .stops
                .iter()
                .map(|stop| GetTransitStopsForRouteResponseGroupStop {
                    id: stop.id.clone(),
                    name: stop.name.clone(),
                })
                .collect(),
        })
        .collect::<Vec<GetTransitStopsForRouteResponseGroup>>();

    Ok((
        StatusCode::OK,
        Json(GetTransitStopsForRouteResponse {
            data: GetTransitStopsForRouteResponseData { groups },
        }),
    )
        .into_response())
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use serde_json::json;
    use tower::ServiceExt;
    use tracing_test::traced_test;

    use crate::{
        app::gen_app,
        types::mta_get_stops_for_route_response::{
            GetStopsForRouteResponse, GetStopsForRouteResponseData,
            GetStopsForRouteResponseDataEntry, GetStopsForRouteResponseDataEntryStopGrouping,
            GetStopsForRouteResponseDataEntryStopGroupingStopGroup,
            GetStopsForRouteResponseDataEntryStopGroupingStopGroupName,
            GetStopsForRouteResponseDataEntryStopGroupingType,
            GetStopsForRouteResponseDataReferences, GetStopsForRouteResponseDataReferencesStop,
        },
    };

    use super::*;

    #[tokio::test]
    #[traced_test]
    async fn get_response() {
        let mut mock_server = mockito::Server::new_async().await;

        let app = gen_app(mock_server.url().as_str(), "key", None);

        let mock_response = GetStopsForRouteResponse {
            data: GetStopsForRouteResponseData {
                entry: GetStopsForRouteResponseDataEntry {
                    stopGroupings: vec![GetStopsForRouteResponseDataEntryStopGrouping {
                        r#type: GetStopsForRouteResponseDataEntryStopGroupingType::Direction,
                        stopGroups: vec![GetStopsForRouteResponseDataEntryStopGroupingStopGroup {
                            id: "group id".to_string(),
                            name: GetStopsForRouteResponseDataEntryStopGroupingStopGroupName {
                                name: "group name".to_string(),
                            },
                            stopIds: vec!["s1".to_string(), "s2".to_string()],
                        }],
                    }],
                },
                references: GetStopsForRouteResponseDataReferences {
                    stops: vec![
                        GetStopsForRouteResponseDataReferencesStop {
                            id: "s1".to_string(),
                            name: "stop 1".to_string(),
                        },
                        GetStopsForRouteResponseDataReferencesStop {
                            id: "s2".to_string(),
                            name: "stop 2".to_string(),
                        },
                    ],
                },
            },
        };

        // NOTE: "B1" is a parameter to the mock URL!
        let mock_server = mock_server
            .mock("GET", "/api/where/stops-for-route/B1%2B.json")
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&mock_response).unwrap())
            .match_query(mockito::Matcher::Regex(".*".to_string()))
            .create_async()
            .await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/transit-stops-for-route?route_id=B1%2B")
                    .header("content-type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        mock_server.assert();

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: GetTransitStopsForRouteResponse = serde_json::from_slice(&body).unwrap();

        // assert_eq!(body.data.groups.len(), 1);
        assert_eq!(body.data.groups[0].id, "group id");
        assert_eq!(body.data.groups[0].stops.len(), 2);
        assert_eq!(body.data.groups[0].stops[0].id, "s1");
        assert_eq!(body.data.groups[0].stops[0].name, "stop 1");
        assert_eq!(body.data.groups[0].stops[1].id, "s2");
        assert_eq!(body.data.groups[0].stops[1].name, "stop 2");
    }

    #[tokio::test]
    #[traced_test]
    async fn test_not_found() {
        let mut mock_server = mockito::Server::new_async().await;

        let app = gen_app(mock_server.url().as_str(), "key", None);

        // NOTE: "B1" is a parameter to the mock URL!
        let mock_server = mock_server
            .mock("GET", "/api/where/stops-for-route/B1%2B.json")
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&json!({})).unwrap())
            .with_status(404)
            .match_query(mockito::Matcher::Regex(".*".to_string()))
            .create_async()
            .await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/transit-stops-for-route?route_id=B1%2B")
                    .header("content-type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        mock_server.assert();
    }
}
