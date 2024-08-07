use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub enum GetStopsAtLocationInput {
    LatLong(String, String),
    GooglePlaceId(String),
}
