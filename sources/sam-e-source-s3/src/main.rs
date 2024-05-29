mod data;
mod response;
mod webhook;

use axum::{routing::post, Router};

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

    info!("Starting the SAM-E s3 webhook handler...");

    debug!("Reading the current configuration");
    let config_env_string = std::env::var("CONFIG").expect("CONFIG env variable not found");
    let config: Config = serde_yaml::from_str(&config_env_string)?;
    debug!("Configuration read successfully");

    debug!("Creating the API state");
    let api_state = data::ApiState::from_config(&config).await;

    debug!("Setting up the webhook route");
    let app = Router::new()
        .route("/", post(webhook::handler))
        .with_state(api_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
