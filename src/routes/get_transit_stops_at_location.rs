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

use super::get_transit_stops_for_route::{
    GetTransitStopsForRouteResponseData, GetTransitStopsForRouteResponseGroup,
    GetTransitStopsForRouteResponseGroupStop,
};

#[derive(Validate, Deserialize)]
pub struct GetTransitStopsAtLocation {
    #[validate(length(min = 1, message = "Must be at least 1 character"))]
    pub query: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetTransitStopsForRouteResponse {
    pub data: GetTransitStopsForRouteResponseData,
}

pub async fn get_transit_stops_at_location(
    State(state): State<AppState>,
    ValidatedQuery(payload): ValidatedQuery<GetTransitStopsAtLocation>,
) -> Result<Response, AppError> {
    let groups = state
        .mta_client
        .get_stops_at_location(payload.query)
        .await
        .map_err(|e| {
            error!("Failed to fetch stops at location: {}", e);
            AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
        })?
        .groups
        .iter()
        .map(|s| GetTransitStopsForRouteResponseGroup {
            id: s.id.clone(),
            name: s.name.clone(),
            route_name: s.route_name.clone(),
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
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    use tracing_test::traced_test;
    use urlencoding::encode;

    use crate::{
        app::{gen_app, AppConfig},
        types::{
            mta_get_stops_at_location_response::{
                GetStopsAtLocationResponse, GetStopsAtLocationResponseStops, StopAtLocation,
                StopAtLocationRoute,
            },
            mta_get_stops_for_route_response::{
                GetStopsForRouteResponse, GetStopsForRouteResponseData,
                GetStopsForRouteResponseDataEntry, GetStopsForRouteResponseDataReferences,
            },
            tomtom_search_response::{
                TomTomSearchResponse, TomTomSearchResponseResult,
                TomTomSearchResponseResultPosition,
            },
        },
    };

    #[tokio::test]
    #[traced_test]
    async fn test_response() {
        let mut mock_server = mockito::Server::new_async().await;

        let app = gen_app(AppConfig {
            mta_host: mock_server.url(),
            mta_key: "key".to_string(),
            tomtom_key: "key".to_string(),
            tomtom_host: mock_server.url(),
            auth_key: None,
        });

        let search = encode("search");

        let tomtom_response = TomTomSearchResponse {
            results: vec![TomTomSearchResponseResult {
                id: "1".to_string(),
                position: TomTomSearchResponseResultPosition { lat: 1.0, lon: 1.0 },
            }],
        };

        let stops_for_location_response = GetStopsAtLocationResponse {
            data: GetStopsAtLocationResponseStops {
                stops: vec![StopAtLocation {
                    id: "1".to_string(),
                    routes: vec![StopAtLocationRoute {
                        id: "1".to_string(),
                    }],
                }],
            },
        };

        let stops_for_route_response = GetStopsForRouteResponse {
            data: GetStopsForRouteResponseData {
                entry: GetStopsForRouteResponseDataEntry {
                    stopGroupings: vec![],
                },
                references: GetStopsForRouteResponseDataReferences {
                    stops: vec![],
                    routes: vec![],
                },
            },
        };

        mock_server
            .mock(
                "GET",
                mockito::Matcher::Regex(format!("/search/2/geocode/{}.json", search).to_string()), //    "/search/2/geocode/B1%2B.json"
            )
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&tomtom_response).unwrap())
            .match_query(mockito::Matcher::Regex(".*".to_string()))
            .create_async()
            .await;

        mock_server
            .mock("GET", "/api/where/stops-for-location.json")
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&stops_for_location_response).unwrap())
            .match_query(mockito::Matcher::Regex(".*".to_string()))
            .create_async()
            .await;

        mock_server
            .mock(
                "GET",
                mockito::Matcher::Regex(
                    format!("/api/where/stops-for-route/{}.json", "1").to_string(),
                ),
            )
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&stops_for_route_response).unwrap())
            .match_query(mockito::Matcher::Regex(".*".to_string()))
            .create_async()
            .await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/transit-stops-at-location?query={}", search))
                    .header("content-type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
