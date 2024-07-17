use crate::{types::app_state::AppState, utils::app_error::AppError};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use tracing::error;

#[derive(Serialize, Debug)]
struct StopResponse {
    expected_arrival_time: String,
    minutes_until_arrival: i64,
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

#[derive(serde::Serialize)]
pub struct StopInformation {
    minutes: String,
    path_id: String,
}
