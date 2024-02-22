use sam_e_types::config::Config;
use crate::scripts::build::infrastructure::create_infrastructure_files;

use std::{env, fs};
use tracing::{error, info, debug};

const SAM_E_DIRECTORY: &str = ".sam-e";

pub fn rebuild() -> anyhow::Result<()> {
    info!("Rebuilding SAM-E environment");

    let current_dir = env::current_dir()?;
    debug!("Detected current directory as: {:?}", current_dir);

    let sam_e_directory_path = format!("{}/{}", current_dir.to_str().unwrap(), SAM_E_DIRECTORY);

    // Checks to see if init has been run prior to build
    if fs::metadata(&sam_e_directory_path).is_err() {
        error!("SAM-E directory not found, please run 'sam-e init' and/or 'sam-e build' before rebuilding.");

        return Ok(());
    }

    // Reads the current config file
    let current_config_raw =
        fs::read_to_string(format!("{}/sam-e-config.yaml", sam_e_directory_path))?;
    let config: Config = serde_yaml::from_str(&current_config_raw)?;

    // Creates infrastructure files based on config (i.e. dockerfiles, docker-compose, configs etc)
    create_infrastructure_files(&config)
}
