use crate::services::{
    maps_client::maps_service::MapsService, transit_service::transit_service::TransitService,
};

#[derive(Clone)]
pub struct AppState {
    pub transit_service: TransitService,
    pub maps_service: MapsService,
    pub auth_key: Option<String>,
}
