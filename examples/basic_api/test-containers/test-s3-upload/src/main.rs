use aws_config::{profile::ProfileFileCredentialsProvider, BehaviorVersion};
use aws_sdk_s3::{config::Region, Client};
use aws_lambda_events::event::sqs::SqsEvent;

use lambda_runtime::{run, service_fn, Error, LambdaEvent};

use tracing::info;
use tracing_subscriber::EnvFilter;


#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .with_ansi(false)
        .without_time()
        .init();

    info!("Starting S3 Lambda...");
    let region = Region::new("eu-west-2");

    let profile_provider = ProfileFileCredentialsProvider::builder()
        .profile_name("staging-mfa")
        .build();

    let config = aws_config::defaults(BehaviorVersion::v2023_11_09())
        .region(region)
        .credentials_provider(profile_provider)
        .endpoint_url("http://s3-local:9000")
        .load()
        .await;

    let _client = Client::new(&config);

    run(service_fn(function_handler)).await
}

async fn function_handler(event: LambdaEvent<SqsEvent>) -> Result<(), Error> {
    info!("Received event: {:?}", event);

    Ok(())
}
