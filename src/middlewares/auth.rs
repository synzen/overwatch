use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::{types::app_state::AppState, utils::app_error::AppError};

pub async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    match state.auth_key {
        Some(ref auth_key) => match headers.get("authorization") {
            Some(header) if header == auth_key => Ok(next.run(request).await),
            _ => Err(AppError::new(StatusCode::UNAUTHORIZED, "Unauthorized")),
        },
        None => {
            let response = next.run(request).await;
            return Ok(response);
        }
    }
}
