use axum::{routing::get, Router};

use crate::types::app_state::AppState;

mod get_audio;
mod get_transit_arrival_times;
mod get_transit_routes;
mod get_transit_stops_at_location;
mod get_transit_stops_for_route;

pub fn apply_routes(app: Router<AppState>) -> Router<AppState> {
    app.route(
        "/transit-arrival-times",
        get(get_transit_arrival_times::get_transit_arrival_times),
    )
    .route(
        "/transit-routes",
        get(get_transit_routes::get_transit_routes),
    )
    .route(
        "/transit-stops-for-route",
        get(get_transit_stops_for_route::get_transit_stops_for_route),
    )
    .route(
        "/transit-stops-at-location",
        get(get_transit_stops_at_location::get_transit_stops_at_location),
    )
    .route("/audio", get(get_audio::get_audio))
}
