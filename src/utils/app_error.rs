use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Serialize;

#[derive(Debug)]
pub struct AppError {
    pub code: StatusCode,
    pub message: String,
}

impl AppError {
    pub fn new(code: StatusCode, message: &str) -> Self {
        AppError {
            code: code,
            message: message.to_string(),
        }
    }
}

#[derive(Serialize)]
struct ResponseJson {
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response<Body> {
        (
            self.code,
            Json(ResponseJson {
                message: self.message,
            }),
        )
            .into_response()
    }
}
