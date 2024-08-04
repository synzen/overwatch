use crate::{
    services::maps_client::maps_service::AutocompleteSearchInput,
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

#[derive(Validate, Deserialize)]
pub struct GetLocationSearchAutocompletePayload {
    #[validate(length(min = 1, message = "Must be at least 1 character"))]
    pub search: String,

    #[validate(length(min = 1, message = "Must be at least 1 character"))]
    pub lat: String,

    #[validate(length(min = 1, message = "Must be at least 1 character"))]
    pub lon: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetLocationSearchAutocompleteResponseDataPrediction {
    pub main_text: String,
    pub secondary_text: String,
    pub place_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetLocationSearchAutocompleteResponseData {
    pub predictions: Vec<GetLocationSearchAutocompleteResponseDataPrediction>,
}

#[derive(Serialize, Deserialize)]
pub struct GetLocationSearchAutocompleteResponse {
    pub data: GetLocationSearchAutocompleteResponseData,
}

pub async fn get_location_search_autocomplete(
    State(state): State<AppState>,
    ValidatedQuery(GetLocationSearchAutocompletePayload { lat, lon, search }): ValidatedQuery<
        GetLocationSearchAutocompletePayload,
    >,
) -> Result<Response, AppError> {
    let predictions = state
        .maps_service
        .get_autocomplete(AutocompleteSearchInput {
            input: search,
            lat,
            lon,
        })
        .await
        .map_err(|e| {
            error!("Failed to fetch location search autocomplete: {}", e);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to fetch location search autocomplete",
            )
        })?;

    Ok(Json(GetLocationSearchAutocompleteResponse {
        data: GetLocationSearchAutocompleteResponseData {
            predictions: predictions
                .predictions
                .into_iter()
                .map(|p| GetLocationSearchAutocompleteResponseDataPrediction {
                    main_text: p.main_text,
                    secondary_text: p.secondary_text,
                    place_id: p.place_id,
                })
                .collect(),
        },
    })
    .into_response())
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{to_bytes, Body},
        http::Request,
    };
    use tower::ServiceExt;

    use super::*;
    use crate::{
        app::gen_mock_app,
        services::maps_client::types::google_autocomplete_response::{
            GoogleAutocompleteResponse, GoogleAutocompleteResponsePrediction,
            GoogleAutocompleteResponsePredictionStructuredFormatting,
        },
    };

    #[tokio::test]
    async fn test_get_location_search_autocomplete() {
        let mut mock_app = gen_mock_app().await;

        let mock_google_response = GoogleAutocompleteResponse {
            predictions: vec![GoogleAutocompleteResponsePrediction {
                place_id: "123".to_string(),
                structed_formatting: GoogleAutocompleteResponsePredictionStructuredFormatting {
                    main_text: "Test Main".to_string(),
                    secondary_text: "Test Sec".to_string(),
                },
            }],
        };

        let mock_server = mock_app
            .google_server
            .mock("GET", "/maps/api/place/autocomplete/json")
            .with_body(serde_json::to_string(&mock_google_response).unwrap())
            .match_query(mockito::Matcher::Regex(".*".to_string()))
            .create_async()
            .await;

        let response = mock_app
            .app
            .oneshot(
                Request::builder()
                    .uri("/location-search-autocomplete?search=test&lat=40.7128&lon=74.0060")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        mock_server.assert();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: GetLocationSearchAutocompleteResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(body.data.predictions.len(), 1);
        assert_eq!(body.data.predictions[0].main_text, "Test Main");
        assert_eq!(body.data.predictions[0].secondary_text, "Test Sec");
        assert_eq!(body.data.predictions[0].place_id, "123");
    }
}
