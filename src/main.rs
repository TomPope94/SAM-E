pub mod data;

use std::env;

use axum::{
    response::Html,
    routing::{get, post},
    Router,
};
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

use data::api::ApiState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .with_ansi(false)
        .without_time()
        .init();

    info!("Now parsing the SAM template");
    let sam_template = env::var("SAM_TEMPLATE").expect("No SAM template found");
    debug!("SAM template: {:?}", sam_template);

    let api_state = ApiState::new(&sam_template);

    info!("Setting up invocation endpoints for Lambda runtime API");

    let invocation_routes = Router::new()
        .route("/next", get(handler))
        .route("/:request_id/response", post(|| async { "OK" }))
        .route("/:request_id/error", post(|| async { "OK" }));

    // build our application with a route
    let app = Router::new()
        .nest("/2018-06-01/runtime/invocation", invocation_routes)
        .route("/2018-06-01/runtime/init/error", post(|| async { "OK" }))
        .route("/*path", get(|| async { "OK" }))
        .with_state(api_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}
