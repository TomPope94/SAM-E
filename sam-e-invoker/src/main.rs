mod data;
mod handlers;
mod middleware;

use axum::{
    routing::{get, post},
    Router,
};
use std::env;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

use data::api::ApiState;
use handlers::{
    client::{queues, request},
    invocation::{init_error, invocation_error, next, response},
    webhook::s3,
};

use sam_e_types::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .with_ansi(false)
        .without_time()
        .init();

    debug!("Starting the SAM-E environment...");

    info!("Reading the current configuration");

    let config_env_string = env::var("CONFIG").expect("CONFIG env variable not found");
    let config: Config = serde_yaml::from_str(&config_env_string)?;

    let api_state = ApiState::new(&config).await;
    let cloned_store = api_state.get_store().clone();

    tokio::spawn(async move {
        info!("Starting polling queues for messages");
        queues::listen_to_queues(config, cloned_store).await;
    });

    info!("Setting up invocation endpoints for Lambda runtime API");

    let invocation_routes = Router::new()
        .route("/next", get(next::request_handler))
        .route("/:request_id/response", post(response::response_handler))
        .route(
            "/:request_id/error",
            post(invocation_error::response_handler),
        )
        .route_layer(axum::middleware::from_fn(middleware::headers_mw));

    let webhook_routes = Router::new()
        .route("/s3", post(s3::handler))
        .route_layer(axum::middleware::from_fn(middleware::headers_mw));
    // build our application with a route
    let app = Router::new()
        .nest(
            "/:container_name/2018-06-01/runtime/invocation",
            invocation_routes,
        )
        .route(
            "/:container_name/2018-06-01/runtime/init/error",
            post(init_error::response_handler),
        )
        .route(
            "/2018-06-01/runtime/invocation/:request_id/response",
            post(response::response_handler),
        )
        .route(
            "/custom/*path",
            get(request::request_handler)
                .post(request::request_handler)
                .put(request::request_handler)
                .delete(request::request_handler)
                .patch(request::request_handler),
        )
        .nest("/webhook", webhook_routes)
        .layer(middleware::cors_layer())
        .with_state(api_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
