use std::error::Error;

use axum::{
    async_trait,
    extract::{FromRequest, Query, Request},
    http::StatusCode,
};
use serde::de::DeserializeOwned;
use validator::Validate;

use super::app_error::AppError;

pub struct ValidatedQuery<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedQuery<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Query(data) = match Query::<T>::from_request(req, state).await {
            Ok(data) => data,
            Err(e) => match e.source() {
                Some(source) => {
                    return Err(AppError::new(
                        StatusCode::BAD_REQUEST,
                        format!("Invalid query: {}", source.to_string()).as_str(),
                    ));
                }
                None => {
                    return Err(AppError::new(
                        StatusCode::BAD_REQUEST,
                        e.body_text().as_str(),
                    ));
                }
            },
        };

        let data = match data.validate().map(|_| ValidatedQuery(data)).map_err(|e| {
            AppError::new(
                StatusCode::BAD_REQUEST,
                format!("Invalid query: {}", e.to_string()).as_str(),
            )
        }) {
            Ok(data) => data,
            Err(e) => return Err(e),
        };

        Ok(data)
    }
}
