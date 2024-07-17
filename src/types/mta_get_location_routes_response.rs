#![allow(non_snake_case)]
#![allow(dead_code)]
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RouteForLocation {
    pub id: String,
    pub description: String,
    pub shortName: String,
}

#[derive(Deserialize)]
pub struct GetRoutesForLocationResponseData {
    pub routes: Vec<RouteForLocation>,
}

#[derive(Deserialize)]
pub struct GetRoutesForLocationResponse {
    pub data: GetRoutesForLocationResponseData,
}
