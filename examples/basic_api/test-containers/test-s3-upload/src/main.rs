use aws_config::BehaviorVersion;
use aws_sdk_s3::{Config, config::Region, Client};
use aws_lambda_events::event::sqs::SqsEvent;

use lambda_runtime::{run, service_fn, Error, LambdaEvent};

use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

use std::fs;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .with_ansi(false)
        .without_time()
        .init();
    info!("Starting S3 Lambda...");

    run(service_fn(function_handler)).await
}

async fn function_handler(event: LambdaEvent<SqsEvent>) -> Result<(), Error> {
    info!("Received event: {:?}", event);
    debug!("Processing {} records", event.payload.records.len());
    debug!("Creating S3 client...");

    let region = Region::new("eu-west-2");

    let raw_config = Config::builder()
        .behavior_version(BehaviorVersion::v2023_11_09())
        .region(region)
        .force_path_style(true)
        .endpoint_url("http://s3-local:9000")
        .build();

    let client = Client::from_conf(raw_config);
    debug!("Client created successfully");

    let messages = event.payload.records;
    for message in messages {
        if let (Some(body), Some(id)) = (message.body, message.message_id) {
            debug!("Writing file inside container: {}", id);
            let file_name = format!("{}.txt", id);
            let file_path = format!("/tmp/{}", file_name);
            fs::write(&file_path, body.as_bytes()).expect("Unable to write file");

            let file_bytes = fs::read(&file_path).expect("Unable to read file");

            debug!("Uploading file to S3: {}", file_name);
            let _response = client
                .put_object()
                .bucket("test-bucket-002")
                .key(&file_name)
                .body(file_bytes.into())
                .send()
                .await?;
        }
    }

    Ok(())
}
