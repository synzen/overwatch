#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct GetStopsForRouteResponseDataReferencesStop {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Serialize)]
pub struct GetStopsForRouteResponseDataReferences {
    pub stops: Vec<GetStopsForRouteResponseDataReferencesStop>,
}

#[derive(Deserialize, Serialize)]
pub struct GetStopsForRouteResponseDataEntryStopGroupingStopGroupName {
    pub name: String,
}

#[derive(Deserialize, Serialize)]
pub struct GetStopsForRouteResponseDataEntryStopGroupingStopGroup {
    pub id: String,
    pub name: GetStopsForRouteResponseDataEntryStopGroupingStopGroupName,
    pub stopIds: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub enum GetStopsForRouteResponseDataEntryStopGroupingType {
    #[serde(rename = "direction")]
    Direction,
}

#[derive(Deserialize, Serialize)]
pub struct GetStopsForRouteResponseDataEntryStopGrouping {
    pub r#type: GetStopsForRouteResponseDataEntryStopGroupingType,
    pub stopGroups: Vec<GetStopsForRouteResponseDataEntryStopGroupingStopGroup>,
}

#[derive(Deserialize, Serialize)]
pub struct GetStopsForRouteResponseDataEntry {
    pub stopGroupings: Vec<GetStopsForRouteResponseDataEntryStopGrouping>,
}

#[derive(Deserialize, Serialize)]
pub struct GetStopsForRouteResponseData {
    pub references: GetStopsForRouteResponseDataReferences,
    pub entry: GetStopsForRouteResponseDataEntry,
}

#[derive(Deserialize, Serialize)]
pub struct GetStopsForRouteResponse {
    pub data: GetStopsForRouteResponseData,
}
