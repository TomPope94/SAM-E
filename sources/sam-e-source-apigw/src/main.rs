mod data;
mod middleware;
mod request;
mod response;
pub mod utils;

use axum::{
    routing::get,
    Router,
};

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

    info!("Starting the SAM-E API source handler...");

    debug!("Reading the current configuration");
    let config_env_string = std::env::var("CONFIG").expect("CONFIG env variable not found");
    let config: Config = serde_yaml::from_str(&config_env_string)?;
    debug!("Configuration read successfully");

    debug!("Creating the API state");
    let api_state = data::ApiState::from_config(&config);

    debug!("Setting up the API routes");
    let app = Router::new()
        .route("/", 
               get(request::handler)
               .post(request::handler)
               .patch(request::handler)
               .put(request::handler)
               .delete(request::handler)
        )
        .route("/*path", 
               get(request::handler)
               .post(request::handler)
               .patch(request::handler)
               .put(request::handler)
               .delete(request::handler)
        )
        .layer(middleware::cors_layer())
        .with_state(api_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Listening on: {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}


