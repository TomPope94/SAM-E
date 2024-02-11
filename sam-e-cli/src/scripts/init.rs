use sam_e_types::config::{Config, Runtime};

use std::{env, fs};
use tracing::{debug, info};

const SAM_E_DIRECTORY: &str = ".sam-e";

pub fn init() -> anyhow::Result<()> {
    info!("Now initialising the SAM-E environment...");

    let current_directory = env::current_dir()?;
    debug!("Detected current directory as: {:?}", current_directory);

    let sam_e_directory_path = format!(
        "{}/{}",
        current_directory.to_str().unwrap(),
        SAM_E_DIRECTORY
    );

    // Check if folder already exists
    if fs::metadata(&sam_e_directory_path).is_ok() {
        info!("SAM-E directory already exists, cancelling initialisation...");
        return Ok(());
    }

    info!("Creating SAM-E directory at: {:?}", sam_e_directory_path);
    fs::create_dir(&sam_e_directory_path)?;
    debug!("SAM-E directory created successfully");

    let sam_e_config_path = format!("{}/sam-e-config.yaml", sam_e_directory_path);
    info!("Creating SAM-E config file at: {:?}", sam_e_config_path);

    let empty_config = Config::new(vec![], Runtime::default(), vec![]);
    let config_string = serde_yaml::to_string(&empty_config)?;

    fs::write(&sam_e_config_path, config_string)?;
    debug!("SAM-E config file created successfully");

    Ok(())
}
