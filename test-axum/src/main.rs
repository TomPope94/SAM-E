//! This is an example function that leverages the Lambda Rust runtime HTTP support
//! and the [axum](https://docs.rs/axum/latest/axum/index.html) web framework.  The
//! runtime HTTP support is backed by the [tower::Service](https://docs.rs/tower-service/0.3.2/tower_service/trait.Service.html)
//! trait.  Axum's applications are also backed by the `tower::Service` trait.  That means
//! that it is fairly easy to build an Axum application and pass the resulting `Service`
//! implementation to the Lambda runtime to run as a Lambda function.  By using Axum instead
//! of a basic `tower::Service` you get web framework niceties like routing, request component
//! extraction, validation, etc.
use axum::http::StatusCode;
use axum::{
    extract::Path,
    response::Json,
    routing::{get, post},
    Router,
};
use lambda_http::{
    run, 
    Error,
};
use serde_json::{json, Value};
use std::env::set_var;
use tracing::info;

async fn root() -> Json<Value> {
    info!("root() called");
    Json(json!({ "msg": "I am GET /" }))
}

async fn get_foo() -> Json<Value> {
    info!("get_foo() called");
    Json(json!({ "msg": "I am GET /foo" }))
}

async fn post_foo() -> Json<Value> {
    Json(json!({ "msg": "I am POST /foo" }))
}

async fn post_foo_name(Path(name): Path<String>) -> Json<Value> {
    Json(json!({ "msg": format!("I am POST /foo/:name, name={name}") }))
}

/// Example on how to return status codes and data from an Axum function
async fn health_check() -> (StatusCode, String) {
    let health = true;
    match health {
        true => (StatusCode::OK, "Healthy!".to_string()),
        false => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Not healthy!".to_string(),
        ),
    }
}

// Sample middleware that logs the request id
async fn context_mw(req: axum::extract::Request, next: axum::middleware::Next) -> impl axum::response::IntoResponse {
    let context = req.extensions().get::<lambda_http::request::RequestContext>();
    // if let Some(ApiGatewayV1(ctx)) = context {
    //     tracing::info!("RequestId = {:?}", ctx.request_id);
    // }
    info!("Context = {:#?}", context);
    next.run(req).await
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        // disable printing the name of the module in every log line.
        // .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    info!("Starting Lambda");

    let app = Router::new()
        .route("/Prod/", get(root))
        .route("/Prod/foo", get(get_foo).post(post_foo))
        .route("/Prod/foo/:name", post(post_foo_name))
        .route("/Prod/health/", get(health_check))
        .route_layer(axum::middleware::from_fn(context_mw));

    run(app).await
}
