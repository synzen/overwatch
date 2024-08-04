use crate::{services::maps_client::maps_service::MapsService, utils::mta_client::MtaClient};

#[derive(Clone)]
pub struct AppState {
    pub mta_client: MtaClient,
    pub maps_service: MapsService,
    pub auth_key: Option<String>,
}
