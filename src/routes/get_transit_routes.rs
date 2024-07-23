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
#[cfg(test)]
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};
use tracing::error;
use validator::Validate;

#[derive(Validate, Deserialize)]
pub struct GetTransitRoutesPayload {
    #[validate(length(min = 1, message = "Must be at least 1 character"))]
    pub search: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetTransitRoutesResponseDataRoute {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetTransitRoutesResponseData {
    pub routes: Vec<GetTransitRoutesResponseDataRoute>,
}

#[derive(Serialize, Deserialize)]
pub struct GetTransitRoutesResponse {
    pub data: GetTransitRoutesResponseData,
}

#[cfg_attr(test, debug_handler)]
pub async fn get_transit_routes(
    State(state): State<AppState>,
    ValidatedQuery(payload): ValidatedQuery<GetTransitRoutesPayload>,
) -> Result<Response, AppError> {
    let routes = state
        .mta_client
        .get_routes(&payload.search)
        .await
        .map_err(|e| {
            error!("Failed to fetch transit routes: {}", e);
            AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
        })?
        .routes
        .iter()
        .map(|r| GetTransitRoutesResponseDataRoute {
            id: r.id.clone(),
            name: r.name.clone(),
        })
        .collect::<Vec<GetTransitRoutesResponseDataRoute>>();

    Ok((
        StatusCode::OK,
        Json(GetTransitRoutesResponse {
            data: GetTransitRoutesResponseData { routes },
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
    use tower::ServiceExt;

    use crate::{
        app::gen_app,
        types::mta_get_routes_response::{
            GetRoutesResponse, GetRoutesResponseData, GetRoutesResponseRoute,
        },
    };

    use super::*;

    #[tokio::test]
    async fn get_response() {
        let mut mock_server = mockito::Server::new_async().await;

        let app = gen_app(mock_server.url().as_str(), "key", None);

        let mock_response = GetRoutesResponse {
            data: GetRoutesResponseData {
                list: vec![
                    GetRoutesResponseRoute {
                        id: "1".to_string(),
                        shortName: "A".to_string(),
                    },
                    GetRoutesResponseRoute {
                        id: "2".to_string(),
                        shortName: "B".to_string(),
                    },
                ],
            },
        };

        let mock_server = mock_server
            .mock("GET", "/api/where/routes-for-agency/MTA%20NYCT.json")
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&mock_response).unwrap())
            .match_query(mockito::Matcher::Regex(".*".to_string()))
            .create_async()
            .await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/transit-routes?search=A")
                    .header("content-type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        mock_server.assert();

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: GetTransitRoutesResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(body.data.routes.len(), 1);
        assert_eq!(body.data.routes[0].id, "1");
        assert_eq!(body.data.routes[0].name, "A");
    }
}
