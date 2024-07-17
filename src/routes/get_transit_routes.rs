use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use tracing::error;

use crate::{types::app_state::AppState, utils::app_error::AppError};

#[derive(serde::Deserialize)]
pub struct GetTransitRoutesPayload {
    pub search: String,
}

#[derive(serde::Serialize)]
pub struct GetTransitRoutesResponseDataRoute {
    pub id: String,
    pub name: String,
}

#[derive(serde::Serialize)]
pub struct GetTransitRoutesResponseData {
    pub routes: Vec<GetTransitRoutesResponseDataRoute>,
}

#[derive(serde::Serialize)]
pub struct GetTransitRoutesResponse {
    pub data: GetTransitRoutesResponseData,
}

pub async fn get_transit_routes(
    Query(payload): Query<GetTransitRoutesPayload>,
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    let routes = state
        .mta_client
        .get_routes(payload.search)
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
