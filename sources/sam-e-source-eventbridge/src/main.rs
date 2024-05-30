pub mod data;
pub mod middleware;
pub mod request;
pub mod response;

use axum::{routing::post, Router};

use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

use sam_e_types::config::Config;

use aws_sdk_sqs::{config::Region, Client};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(false)
        .without_time()
        .init();

    info!("Starting the SAM-E local eventbridge...");

    debug!("Reading the current configuration");
    let config_env_string = std::env::var("CONFIG").expect("CONFIG env variable not found");
    let config: Config = serde_yaml::from_str(&config_env_string)?;
    debug!("Configuration read successfully");

    debug!("Setting up the event store");
    let event_store = data::store::EventStore::from_config(&config);
    debug!("Event store setup successfully");

    debug!("Creating AWS SQS client");
    let region = Region::new("eu-west-1");

    // let config = aws_config::defaults(BehaviorVersion::v2023_11_09())
    let config = aws_config::from_env()
        .region(region)
        .endpoint_url("http://sqs-local:9324")
        .load()
        .await;


    let event_store_for_listening = event_store.clone();
    tokio::spawn(async move {
        debug!("Listening for events");
        let event_bus_names = event_store_for_listening.get_event_bus_names();

        let client = Client::new(&config);

        for event_bus_name in event_bus_names {
            event_store_for_listening.listen_for_events(event_bus_name, client.clone()).await;
        }
    });

    debug!("Setting up the eventbridge API routes");
    let app = Router::new()
        .route("/", post(request::handler))
        .with_state(event_store);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3002").await.unwrap();
    info!("Listening on: {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
