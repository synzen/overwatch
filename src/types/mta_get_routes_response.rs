#![allow(non_snake_case)]
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GetRoutesResponseRoute {
    pub id: String,
    pub shortName: String,
}

#[derive(Deserialize)]
pub struct GetRoutesResponseData {
    pub list: Vec<GetRoutesResponseRoute>,
}

#[derive(Deserialize)]
pub struct GetRoutesResponse {
    pub data: GetRoutesResponseData,
}
