use std::{env, fs};
use sam_e_types::config::Config;
use tracing::{debug, trace};

const SAM_E_DIRECTORY: &str = ".sam-e";

pub fn check_init() -> anyhow::Result<()> {
    debug!("Checking for SAM-E directory");
    let sam_e_directory_path = get_sam_e_directory_path()?;

    // Checks to see if init has been run prior to build
    if fs::metadata(&sam_e_directory_path).is_err() {
        return Err(anyhow::anyhow!("SAM-E directory not found, please run 'sam-e init' before rebuilding."));
    }

    Ok(())
}

pub fn get_config() -> anyhow::Result<Config> {
    let sam_e_directory_path = get_sam_e_directory_path()?;
    let current_config_raw =
        fs::read_to_string(format!("{}/sam-e-config.yaml", sam_e_directory_path))?;
    let config: Config = serde_yaml::from_str(&current_config_raw)?;

    Ok(config)
}

pub fn write_config(config: &Config) -> anyhow::Result<()> {
    debug!("Writing config to file");
    let sam_e_directory_path = get_sam_e_directory_path()?;
    let config_string = serde_yaml::to_string(&config)?;

    fs::write(format!("{}/sam-e-config.yaml", sam_e_directory_path), config_string)?;

    debug!("Written config to file");
    Ok(())
}

pub fn get_sam_e_directory_path() -> anyhow::Result<String> {
    debug!("Getting SAM-E directory path");

    let current_dir = env::current_dir()?;
    trace!("Detected current directory as: {:?}", current_dir);

    let sam_e_directory_path = format!("{}/{}", current_dir.to_str().unwrap(), SAM_E_DIRECTORY);
    trace!("SAM-E directory path: {}", sam_e_directory_path);

    Ok(sam_e_directory_path)
}
