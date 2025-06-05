use crate::scripts::{
    environment::build::infrastructure::create_infrastructure_files,
    utils::{check_init, get_config},
};

use tracing::info;

pub fn rebuild() -> anyhow::Result<()> {
    info!("Rebuilding SAM-E environment");

    check_init()?;
    let config = get_config()?;

    // Creates infrastructure files based on config (i.e. dockerfiles, docker-compose, configs etc)
    create_infrastructure_files(&config)
}
