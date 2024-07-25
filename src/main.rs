mod routes;
mod types;
mod utils;
use app::AppConfig;
use std::env;
use tracing::info;
mod app;
mod middlewares;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("Starting app...");

    // get all routes: https://bustime.mta.info/api/where/routes-for-agency/MTA%20NYCT.json?key={KEY}
    // get all stops: https://bustime.mta.info/api/where/stops-for-route/MTA%20NYCT_{BUS}+.json?key={KEY}&includePolylines=false&version=2
    // get all buses at stop: https://bustime.mta.info/api/siri/stop-monitoring.json?key={KEY}&MonitoringRef={STOP_REF}
    // let resp = reqwest::get("https://bustime.mta.info/api/siri/stop-monitoring.json?key={KEY}&MonitoringRef={STOP_REF}").await?.error_for_status();
    let app = app::gen_app(AppConfig {
        mta_host: "https://bustime.mta.info".to_string(),
        mta_key: env::var("MTA_KEY").expect("MTA API key is expected"),
        tomtom_key: env::var("TOMTOM_KEY").expect("TOMTOM key is expected"),
        tomtom_host: "https://api.tomtom.com".to_string(),
        auth_key: match &env::var("AUTH_KEY") {
            Ok(auth_key) => Some(auth_key.to_string()),
            Err(_) => None,
        },
    });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
