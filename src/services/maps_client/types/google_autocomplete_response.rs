use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct StructuredFormatting {
    pub main_text: String,
    pub secondary_text: String,
}

#[derive(Debug, Deserialize)]
pub struct Prediction {
    pub place_id: String,
    pub structed_formatting: StructuredFormatting,
}

#[derive(Debug, Deserialize)]
pub struct GoogleAutocompleteResponse {
    pub predictions: Vec<Prediction>,
}
