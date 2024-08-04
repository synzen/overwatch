use crate::{
    middlewares::auth::auth_middleware,
    routes::apply_routes,
    services::maps_client::maps_service::{MapsService, MapsServiceConfig},
    types::app_state::AppState,
    utils::mta_client::{MtaClient, MtaClientConfig},
};
use axum::{middleware, routing::get, Router};
use tower_http::cors::CorsLayer;

pub struct AppConfig {
    pub mta_host: String,
    pub mta_key: String,
    pub google_maps_host: String,
    pub google_maps_key: String,
    pub auth_key: Option<String>,
}

pub fn gen_app(
    AppConfig {
        auth_key,
        mta_host,
        google_maps_host,
        mta_key,
        google_maps_key,
    }: AppConfig,
) -> Router {
    let cors_middleware = CorsLayer::new();
    let state = AppState {
        mta_client: MtaClient::new(MtaClientConfig {
            host: mta_host,
            api_key: mta_key,
        }),
        maps_service: MapsService::new(MapsServiceConfig {
            host: google_maps_host,
            api_key: google_maps_key,
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

#[cfg(test)]
pub struct MockApp {
    pub mta_server: mockito::ServerGuard,
    pub google_server: mockito::ServerGuard,
    pub app: Router,
}

#[cfg(test)]
pub async fn gen_mock_app() -> MockApp {
    let mock_mta_server = mockito::Server::new_async().await;
    let mock_google_server = mockito::Server::new_async().await;

    let app_config = AppConfig {
        mta_host: mock_mta_server.url(),
        mta_key: "key".to_string(),
        google_maps_host: mock_google_server.url(),
        google_maps_key: "key".to_string(),
        auth_key: None,
    };
    let cors_middleware = CorsLayer::new();
    let state = AppState {
        mta_client: MtaClient::new(MtaClientConfig {
            host: app_config.mta_host,
            api_key: app_config.mta_key,
        }),
        maps_service: MapsService::new(MapsServiceConfig {
            host: app_config.google_maps_host,
            api_key: app_config.google_maps_key,
        }),
        auth_key: app_config.auth_key,
    };

    let router = apply_routes(Router::new())
        .route("/", get(root))
        .layer(cors_middleware)
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state);

    MockApp {
        mta_server: mock_mta_server,
        google_server: mock_google_server,
        app: router,
    }
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
        let mock_app = gen_mock_app().await;

        let response = mock_app
            .app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
