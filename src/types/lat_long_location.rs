use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct LatLongLocation {
    pub latitude: String,
    pub longitude: String,
}
