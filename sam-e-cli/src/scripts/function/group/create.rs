use tracing::{debug, error, info};
use crate::scripts::utils::{check_init, get_config, get_sam_e_directory_path};
use std::fs;

pub async fn create() -> anyhow::Result<()> {
    debug!("Creating function group");

    check_init()?;
    let mut config = get_config()?;

    let group_name = dialoguer::Input::<String>::new()
        .with_prompt("Enter the name of the group")
        .interact()?;

    let mut current_groups = config.get_lambda_groups().clone();
    // check if group already exists
    if current_groups.iter().any(|group| group.0 == &group_name) {
        error!("A group with that name already exists! Exiting...");
        return Ok(());
    }

    debug!("No group with that name detected. Creating new group...");
    current_groups.insert(group_name.clone(), vec![]);
    config.set_lambda_groups(current_groups);

    info!("Group created successfully. Now saving to config file...");

    let config_string = serde_yaml::to_string(&config)?;

    let sam_e_directory_path = get_sam_e_directory_path()?;
    let sam_e_config_path = format!("{}/sam-e-config.yaml", sam_e_directory_path);
    info!("Updating SAM-E config file at: {:?}", sam_e_config_path);

    fs::write(&sam_e_config_path, config_string)?;
    debug!("SAM-E config file updated successfully");


    info!("Run `sam-e function group add` to add lambdas to the group");

    Ok(())
}
