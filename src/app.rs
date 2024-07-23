use crate::{
    middlewares::auth::auth_middleware, routes::apply_routes, types::app_state::AppState,
    utils::mta_client::MtaClient,
};
use axum::{middleware, routing::get, Router};
use tower_http::cors::CorsLayer;

pub fn gen_app(mta_host: &str, mta_key: &str, auth_key: Option<String>) -> Router {
    let cors_middleware = CorsLayer::new();
    let state = AppState {
        mta_client: MtaClient::new(mta_host.to_string(), mta_key.to_string()),
        auth_key,
    };

    apply_routes(Router::new())
        .route("/", get(root))
        .layer(cors_middleware)
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state)
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request, http::StatusCode};
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn hello_world() {
        let app = gen_app("host", "key", None);

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
