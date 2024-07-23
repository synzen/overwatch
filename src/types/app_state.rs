use crate::utils::mta_client::MtaClient;

#[derive(Clone)]
pub struct AppState {
    pub mta_client: MtaClient,
    pub auth_key: Option<String>,
}
