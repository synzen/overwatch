mod routes;
mod types;
mod utils;
use axum::{routing::get, Router};
use std::env;
use utils::mta_client::MtaClient;

#[derive(Clone)]
struct AppState {
    mta_client: MtaClient,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        mta_client: MtaClient::new(env::var("MTA_KEY").expect("MTA_KEY must be set!")),
    };
    // get all routes: https://bustime.mta.info/api/where/routes-for-agency/MTA%20NYCT.json?key={KEY}
    // get all stops: https://bustime.mta.info/api/where/stops-for-route/MTA%20NYCT_{BUS}+.json?key={KEY}&includePolylines=false&version=2
    // get all buses at stop: https://bustime.mta.info/api/siri/stop-monitoring.json?key={KEY}&MonitoringRef={STOP_REF}
    // let resp = reqwest::get("https://bustime.mta.info/api/siri/stop-monitoring.json?key={KEY}&MonitoringRef={STOP_REF}").await?.error_for_status();
    let app = Router::new()
        .route("/", get(root))
        .route("/stops/:id", get(routes::get_stop))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}
