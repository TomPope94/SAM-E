use std::process::Command;
use tracing::info;

pub fn stop() -> anyhow::Result<()> {
    info!("Stopping any running containers in the SAM-E environment...");

    let cmd = "docker ps --filter name=sam-e -q | xargs -r docker stop";

    Command::new("sh").arg("-c").arg(cmd).output()?;

    Ok(())
}
