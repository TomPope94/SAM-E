mod msg_handler;

use lambda_runtime::{run, service_fn, Error};

use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .without_time()
        .init();
    info!("Starting email sender Lambda...");

    run(service_fn(msg_handler::msg_handler)).await
}
