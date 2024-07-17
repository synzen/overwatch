use crate::{routes::apply_routes, types::app_state::AppState, utils::mta_client::MtaClient};
use axum::{routing::get, Router};
use tower_http::cors::CorsLayer;

pub fn gen_app(mta_host: &str, mta_key: &str) -> Router {
    let cors_middleware = CorsLayer::new();
    let state = AppState {
        mta_client: MtaClient::new(mta_host.to_string(), mta_key.to_string()),
    };

    apply_routes(Router::new())
        .route("/", get(root))
        .layer(cors_middleware)
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
        let app = gen_app("host", "key");

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
