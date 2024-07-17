use axum::{routing::get, Router};

use crate::types::app_state::AppState;

mod get_transit_routes;
mod get_transit_stop;

pub fn apply_routes(app: Router<AppState>) -> Router<AppState> {
    app.route(
        "/transit-stops/:id",
        get(get_transit_stop::get_transit_stop),
    )
    .route(
        "/transit-routes",
        get(get_transit_routes::get_transit_routes),
    )
}
