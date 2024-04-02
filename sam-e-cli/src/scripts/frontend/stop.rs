use tracing::{info, warn};

pub fn stop_frontend() -> anyhow::Result<()> {
    info!("Stopping the frontend in the local environment");
    warn!("WARNING: NOT YET IMPLEMENTED");
    Ok(())
}
