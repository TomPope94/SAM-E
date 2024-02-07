pub mod data;
pub mod handlers;
pub mod middleware;

use std::env;

use axum::{
    routing::{get, post},
    Router,
};
use tracing::{info, trace};
use tracing_subscriber::EnvFilter;

use data::api::ApiState;
use handlers::{
    client::request,
    invocation::{init_error, invocation_error, next, response},
};

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
    trace!("SAM template: {:?}", sam_template);

    let api_state = ApiState::new(&sam_template);

    info!("Setting up invocation endpoints for Lambda runtime API");

    let invocation_routes = Router::new()
        .route("/next", get(next::request_handler))
        .route("/:request_id/response", post(response::response_handler))
        .route(
            "/:request_id/error",
            post(invocation_error::response_handler),
        )
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
        .layer(middleware::cors_layer())
        .with_state(api_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3030").await.unwrap();
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
