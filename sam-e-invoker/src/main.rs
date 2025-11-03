mod api_response;
mod data;
mod extension;
mod invocation;
mod middleware;

use axum::{
    routing::{get, post},
    Router,
};
use std::env;
use std::net::SocketAddr;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

use data::api::ApiState;
use invocation::{init_error, invocation_error, invoke, response};

use sam_e_types::config::Config;

use crate::invocation::next_host;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(false)
        .without_time()
        .init();

    info!("Starting the SAM-E environment...");

    debug!("Reading the current configuration");
    let config_env_string = env::var("CONFIG").expect("CONFIG env variable not found");
    let config: Config = serde_yaml::from_str(&config_env_string)?;

    let api_state = ApiState::new(&config).await;

    debug!("Setting up invocation endpoints for Lambda runtime API");
    let invocation_routes = Router::new()
        .route("/next", get(next_host::request_handler))
        .route("/:request_id/response", post(response::response_handler))
        .route(
            "/:request_id/error",
            post(invocation_error::response_handler),
        )
        .route_layer(axum::middleware::from_fn(middleware::headers_mw));

    debug!("Setting up extension endpoints for Lambda Extension API");
    let extension_routes = Router::new()
        .route("/register", post(extension::register_host::handle))
        .route("/event/next", get(extension::event::handle))
        .route_layer(axum::middleware::from_fn(middleware::headers_mw));

    let app = Router::new()
        // Routes with function name in path (legacy format for sources)
        .nest("/2018-06-01/runtime/invocation", invocation_routes)
        .nest("/2020-01-01/extension", extension_routes)
        .route(
            "/:container_name/2018-06-01/runtime/init/error",
            post(init_error::response_handler),
        )
        .route(
            "/2018-06-01/runtime/init/error",
            post(init_error::response_handler),
        )
        .route("/invoke", post(invoke))
        .fallback(middleware::not_found_handler)
        .layer(middleware::cors_layer())
        .with_state(api_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3030").await.unwrap();
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();

    Ok(())
}
