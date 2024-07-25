use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct TomTomSearchResponseResultPosition {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Deserialize, Serialize)]
pub struct TomTomSearchResponseResult {
    pub id: String,
    pub position: TomTomSearchResponseResultPosition,
}

#[derive(Deserialize, Serialize)]
pub struct TomTomSearchResponse {
    pub results: Vec<TomTomSearchResponseResult>,
}
