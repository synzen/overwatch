use crate::{
    middlewares::auth::auth_middleware,
    routes::apply_routes,
    types::app_state::AppState,
    utils::mta_client::{MtaClient, MtaClientConfig},
};
use axum::{middleware, routing::get, Router};
use tower_http::cors::CorsLayer;

pub struct AppConfig {
    pub mta_host: String,
    pub mta_key: String,
    pub auth_key: Option<String>,
}

pub fn gen_app(
    AppConfig {
        auth_key,
        mta_host,
        mta_key,
    }: AppConfig,
) -> Router {
    let cors_middleware = CorsLayer::new();
    let state = AppState {
        mta_client: MtaClient::new(MtaClientConfig {
            host: mta_host,
            api_key: mta_key,
        }),
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
        let app = gen_app(AppConfig {
            mta_host: "host".to_string(),
            mta_key: "key".to_string(),
            auth_key: None,
        });

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
