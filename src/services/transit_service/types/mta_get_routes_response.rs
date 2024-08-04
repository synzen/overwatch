#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct GetRoutesResponseRoute {
    pub id: String,
    pub shortName: String,
}

#[derive(Deserialize, Serialize)]
pub struct GetRoutesResponseData {
    pub list: Vec<GetRoutesResponseRoute>,
}

#[derive(Deserialize, Serialize)]
pub struct GetRoutesResponse {
    pub data: GetRoutesResponseData,
}
