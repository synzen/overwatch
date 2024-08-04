use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GoogleAutocompleteResponsePredictionStructuredFormatting {
    pub main_text: String,
    pub secondary_text: String,
}

#[derive(Serialize, Deserialize)]
pub struct GoogleAutocompleteResponsePrediction {
    pub place_id: String,
    pub structed_formatting: GoogleAutocompleteResponsePredictionStructuredFormatting,
}

#[derive(Serialize, Deserialize)]
pub struct GoogleAutocompleteResponse {
    pub predictions: Vec<GoogleAutocompleteResponsePrediction>,
}
