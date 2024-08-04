use urlencoding::encode;

use super::types::{
    google_autocomplete_response::GoogleAutocompleteResponse, maps_service_error::MapsServiceError,
};

#[derive(Clone)]
pub struct MapsServiceConfig {
    pub api_key: String,
    pub host: String,
}

#[derive(Clone)]
pub struct MapsService {
    config: MapsServiceConfig,
    client: reqwest::Client,
}

pub struct AutocompleteSearchInput {
    pub input: String,
    pub lat: String,
    pub lon: String,
}

pub struct AutocompleteSearchOutputPrediction {
    pub main_text: String,
    pub secondary_text: String,
    pub place_id: String,
}

pub struct AutocompleteSearchOutput {
    pub predictions: Vec<AutocompleteSearchOutputPrediction>,
}

impl MapsService {
    pub fn new(config: MapsServiceConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_autocomplete(
        &self,
        input: AutocompleteSearchInput,
    ) -> Result<AutocompleteSearchOutput, MapsServiceError> {
        let url = format!(
            "{}/maps/api/place/autocomplete/json?input={}&location={},{}&radius=500&key={}",
            self.config.host,
            encode(&input.input),
            input.lat,
            input.lon,
            self.config.api_key
        );

        let resp =
            self.client.get(&url).send().await.map_err(|e| {
                MapsServiceError::Internal(format!("Failed to send request: {}", e))
            })?;

        let body = resp
            .json::<GoogleAutocompleteResponse>()
            .await
            .map_err(|e| {
                MapsServiceError::Internal(format!("Failed to get response body: {}", e))
            })?;

        Ok(AutocompleteSearchOutput {
            predictions: body
                .predictions
                .into_iter()
                .map(|p| AutocompleteSearchOutputPrediction {
                    main_text: p.structed_formatting.main_text,
                    secondary_text: p.structed_formatting.secondary_text,
                    place_id: p.place_id,
                })
                .collect(),
        })
    }
}
