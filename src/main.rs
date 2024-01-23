pub mod data;

use std::env;

use axum::{response::Html, routing::get, Router};
use tracing::{info, debug};
use tracing_subscriber::EnvFilter;

use data::{store, sam::utils};

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

    let sam_routes = utils::create_sam_routes(&sam_template);

    let store = store::Store::new();

    info!("Setting up invocation endpoints for Lambda runtime API");

    // build our application with a route
    let app = Router::new().route("/", get(handler));

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}
