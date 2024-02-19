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
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use aws_config::{
    profile::ProfileFileCredentialsProvider, 
    BehaviorVersion,
};
use aws_sdk_sqs::{config::Region, Client};

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

async fn add_message(Path((queue_name, name)): Path<(String, String)>) -> Json<Value> {
    info!("Creating AWS SQS client");
    let region = Region::new("eu-west-2");

    let profile_provider = ProfileFileCredentialsProvider::builder()
        .profile_name("staging-mfa")
        .build();

    let config = aws_config::defaults(BehaviorVersion::v2023_11_09())
        .region(region)
        .credentials_provider(profile_provider)
        .endpoint_url("http://sqs-local:9324")
        .load()
        .await;

    let client = Client::new(&config);
    
    let queue_url = client.get_queue_url().queue_name(queue_name).send().await;
    if let Ok(queue_url_resp) = queue_url {
        if let Some(url) = queue_url_resp.queue_url() {
            let send_msg = client
                .send_message()
                .queue_url(url)
                .message_body(format!("Hello {}", name))
                .send()
                .await;
            if let Ok(send_msg_resp) = send_msg {
                info!("Message sent: {:?}", send_msg_resp);
                return Json(json!({ "msg": "Message added to queue" }));
            } else {
                error!("Failed to send message: {:?}", send_msg);
                return Json(json!({ "msg": "Failed to send message" }));
            }
        } else {
            error!("Failed to get queue url: {:?}", queue_url_resp);
            return Json(json!({ "msg": "Failed to get queue url successfully" }));
        }
    }
    error!("Failed to get queue url: {:?}", queue_url);
    return Json(json!({ "msg": "Failed to get queue url completely" }));
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
        .with_env_filter(EnvFilter::from_default_env())
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
        .route("/Prod/add/:queue_name/:name", post(add_message))
        .route_layer(axum::middleware::from_fn(context_mw));

    run(app).await
}
