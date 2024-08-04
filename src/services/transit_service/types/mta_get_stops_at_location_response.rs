#![allow(non_snake_case)]
#![allow(dead_code)]
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct StopAtLocationRoute {
    pub id: String,
}

#[derive(Deserialize, Serialize)]
pub struct StopAtLocation {
    pub id: String,
    pub routes: Vec<StopAtLocationRoute>,
}

#[derive(Deserialize, Serialize)]
pub struct GetStopsAtLocationResponseStops {
    pub stops: Vec<StopAtLocation>,
}

#[derive(Deserialize, Serialize)]
pub struct GetStopsAtLocationResponse {
    pub data: GetStopsAtLocationResponseStops,
}
