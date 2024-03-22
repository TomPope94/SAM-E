use sam_e_types::config::config::{Config, RuntimeBuilder};

use std::{env, fs};
use tracing::{debug, info, warn};

use crate::scripts::build::template::find_all_files;

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
        warn!("SAM-E directory already exists, cancelling initialisation...");
        return Ok(());
    }

    info!("Please wait while we find all the YAML files in the current directory...");
    let found_yaml_files = find_all_files(&current_directory, "(.*?)\\.(yaml)$")?;
    let yaml_files: Vec<&str> = found_yaml_files
        .iter()
        .map(|file| file.to_str().unwrap())
        .collect();

    let selection = dialoguer::MultiSelect::new()
        .with_prompt("Please select the YAML files you would like to use as the SAM templates. Use space to select. Press enter when done.")
        .items(&yaml_files)
        .interact()?;

    let selected_as_str = selection
        .iter()
        .map(|&index| yaml_files[index].to_owned())
        .collect::<Vec<String>>();
    debug!("Selected YAML files: {:?}", selected_as_str);

    info!("Creating SAM-E directory at: {:?}", sam_e_directory_path);
    fs::create_dir(&sam_e_directory_path)?;
    debug!("SAM-E directory created successfully");

    let sam_e_config_path = format!("{}/sam-e-config.yaml", sam_e_directory_path);
    info!("Creating SAM-E config file at: {:?}", sam_e_config_path);

    let new_runtime = RuntimeBuilder::new()
        .with_template_locations(selected_as_str)
        .build();

    let new_config = Config::new(vec![], new_runtime, vec![]);
    let config_string = serde_yaml::to_string(&new_config)?;

    fs::write(&sam_e_config_path, config_string)?;
    debug!("SAM-E config file created successfully");

    Ok(())
}
