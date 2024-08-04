use std::collections::HashSet;

use crate::{
    types::{app_state::AppState, lat_long_location::LatLongLocation},
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

#[derive(Validate, Deserialize)]
pub struct GetTransitStopsAtLocation {
    pub lat: String,
    pub lon: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetTransitStopsAtLocationResponseRouteStop {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetTransitStopsAtLocationResponseRouteGrouping {
    pub name: String,
    pub stops: Vec<GetTransitStopsAtLocationResponseRouteStop>,
}

#[derive(Serialize, Deserialize)]
pub struct GetTransitStopsAtLocationResponseRoute {
    pub id: String,
    pub name: String,
    pub groupings: Vec<GetTransitStopsAtLocationResponseRouteGrouping>,
}

#[derive(Serialize, Deserialize)]
pub struct GetTransitStopsAtLocationResponseData {
    pub routes: Vec<GetTransitStopsAtLocationResponseRoute>,
}

#[derive(Serialize, Deserialize)]
pub struct GetTransitStopsAtLocationResponse {
    pub data: GetTransitStopsAtLocationResponseData,
}

pub async fn get_transit_stops_at_location(
    State(state): State<AppState>,
    ValidatedQuery(payload): ValidatedQuery<GetTransitStopsAtLocation>,
) -> Result<Response, AppError> {
    let groups = state
        .transit_service
        .get_stops_at_location(LatLongLocation {
            latitude: payload.lat,
            longitude: payload.lon,
        })
        .await
        .map_err(|e| {
            error!("Failed to fetch stops at location: {}", e);
            AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
        })?
        .groups;

    let mut res = GetTransitStopsAtLocationResponse {
        data: GetTransitStopsAtLocationResponseData {
            routes: Vec::<GetTransitStopsAtLocationResponseRoute>::new(),
        },
    };

    let mut seen_route_ids = HashSet::<String>::new();

    for route in &groups {
        if seen_route_ids.contains(&route.route_id) {
            continue;
        }

        seen_route_ids.insert(route.route_id.clone());

        let mut new_route = GetTransitStopsAtLocationResponseRoute {
            id: route.route_id.clone(),
            name: route.route_name.clone(),
            groupings: Vec::<GetTransitStopsAtLocationResponseRouteGrouping>::new(),
        };

        for g in &groups {
            if route.route_name != g.route_name {
                continue;
            }

            let mut grouping = GetTransitStopsAtLocationResponseRouteGrouping {
                name: g.name.clone(),
                stops: Vec::<GetTransitStopsAtLocationResponseRouteStop>::new(),
            };

            for stop in &g.stops {
                grouping
                    .stops
                    .push(GetTransitStopsAtLocationResponseRouteStop {
                        id: stop.id.clone(),
                        name: stop.name.clone(),
                    });
            }

            new_route.groupings.push(grouping);
        }

        res.data.routes.push(new_route);
    }

    Ok((StatusCode::OK, Json(res)).into_response())
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    use tracing_test::traced_test;

    use crate::{
        app::gen_mock_app,
        services::transit_service::types::{
            mta_get_stops_at_location_response::{
                GetStopsAtLocationResponse, GetStopsAtLocationResponseStops, StopAtLocation,
                StopAtLocationRoute,
            },
            mta_get_stops_for_route_response::{
                GetStopsForRouteResponse, GetStopsForRouteResponseData,
                GetStopsForRouteResponseDataEntry, GetStopsForRouteResponseDataReferences,
            },
        },
    };

    #[tokio::test]
    #[traced_test]
    async fn test_response() {
        let mut mock_app = gen_mock_app().await;

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

        mock_app
            .mta_server
            .mock("GET", "/api/where/stops-for-location.json")
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&stops_for_location_response).unwrap())
            .match_query(mockito::Matcher::Regex(".*".to_string()))
            .create_async()
            .await;

        mock_app
            .mta_server
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

        let response = mock_app
            .app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/transit-stops-at-location?lat={}&lon={}",
                        1.0, 1.0
                    ))
                    .header("content-type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
