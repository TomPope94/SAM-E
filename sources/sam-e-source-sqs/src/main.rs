mod data;
mod queues;

use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

use sam_e_types::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(false)
        .without_time()
        .init();

    info!("Starting the SAM-E SQS source handler...");

    debug!("Reading the current configuration");
    let config_env_string = std::env::var("CONFIG").expect("CONFIG env variable not found");
    let config: Config = serde_yaml::from_str(&config_env_string)?;
    debug!("Configuration read successfully");

    if config.get_runtime().get_use_queue_source() {
        debug!("Using the SQS source as specified in config...");
    } else {
        debug!("SQS source not specified in config, exiting...");
        return Ok(());
    }

    let queue_state = data::QueueState::from_config(&config).await;
    debug!("Queue state created");

    info!("Starting to poll queues for messages");
    queues::listen_to_queues(config, queue_state).await;

    Ok(())
}
