use axum::{
    extract::Path,
    response::Json,
    routing::post,
    Router,
};
use lambda_http::{
    run, 
    Error,
};
use serde_json::{json, Value};
use tracing::{debug, error, info};
use tracing_subscriber::EnvFilter;

use aws_config::BehaviorVersion;
use aws_sdk_sqs::{config::Region, Client};

async fn add_message(Path((queue_name, name)): Path<(String, String)>) -> Json<Value> {
    debug!("Creating AWS SQS client");
    let region = Region::new("eu-west-1");

    // let config = aws_config::defaults(BehaviorVersion::v2023_11_09())
    let config = aws_config::from_env()
        .region(region)
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
        .route("/Prod/queue/add/:queue_name/:name", post(add_message));

    run(app).await
}
