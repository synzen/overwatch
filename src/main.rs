mod routes;
mod types;
mod utils;
use std::env;
use tracing::info;
mod app;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("Starting app...");

    // get all routes: https://bustime.mta.info/api/where/routes-for-agency/MTA%20NYCT.json?key={KEY}
    // get all stops: https://bustime.mta.info/api/where/stops-for-route/MTA%20NYCT_{BUS}+.json?key={KEY}&includePolylines=false&version=2
    // get all buses at stop: https://bustime.mta.info/api/siri/stop-monitoring.json?key={KEY}&MonitoringRef={STOP_REF}
    // let resp = reqwest::get("https://bustime.mta.info/api/siri/stop-monitoring.json?key={KEY}&MonitoringRef={STOP_REF}").await?.error_for_status();
    let app = app::gen_app(env::var("MTA_API_KEY").unwrap().as_str());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
