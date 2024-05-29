use crate::scripts::utils::{check_init, get_config, get_sam_e_directory_path};
use std::fs;
use tracing::{debug, error, info};

/// Add one or more functions to a previously created function group
pub async fn add_function() -> anyhow::Result<()> {
    debug!("Creating function group");

    check_init()?;
    let mut config = get_config()?;

    let lambda_groups = config.get_lambda_groups();
    let group_names = lambda_groups.keys().collect::<Vec<_>>();
    if group_names.is_empty() {
        error!("No groups found. Please create a group first using `sam-e function group create`");
        return Ok(());
    }

    let chosen_group_name = dialoguer::Select::new()
        .with_prompt("Which group do you want to add functions to?")
        .items(&group_names)
        .interact()?;
    let chosen_group_name = group_names[chosen_group_name].clone();
    let mut chosen_group_lambdas = lambda_groups[&chosen_group_name].clone();

    let lambdas = config.get_lambdas();
    let lambda_names = lambdas
        .iter()
        .map(|lambda| lambda.get_name())
        .collect::<Vec<_>>();
    if lambda_names.is_empty() {
        error!("No lambdas found. Please create a lambda first using `sam-e function add`");
        return Ok(());
    }

    let lambdas_not_in_group = lambda_names
        .into_iter()
        .filter(|lambda_name| !chosen_group_lambdas.contains(&lambda_name.to_string()))
        .collect::<Vec<_>>();
    if lambdas_not_in_group.is_empty() {
        error!("No lambdas found that are not already in the group. Exiting...");
        return Ok(());
    }

    let lambda_choices = dialoguer::MultiSelect::new()
        .with_prompt("Select the lambdas you want to add")
        .items(&lambdas_not_in_group)
        .interact()?;

    let mut new_lambdas: Vec<String> = vec![];
    for index in lambda_choices {
        let lambda_name = &lambdas_not_in_group[index];
        new_lambdas.push(lambda_name.to_string());
    }
    chosen_group_lambdas.append(&mut new_lambdas);

    let mut new_lambda_groups = lambda_groups.clone();
    new_lambda_groups.remove(&chosen_group_name);
    new_lambda_groups.insert(chosen_group_name.clone(), chosen_group_lambdas);

    config.set_lambda_groups(new_lambda_groups);

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
