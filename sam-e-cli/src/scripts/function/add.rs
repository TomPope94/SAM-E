use crate::scripts::{
    environment::build::{
        lambda::{add_build_settings, get_lambdas_from_resources, specify_environment_vars},
        template::parse_templates_into_resources,
    },
    utils::{check_init, get_config, get_sam_e_directory_path},
};

use sam_e_types::config::{Config, Lambda};
use std::fs;
use tracing::{debug, info};

pub fn add() -> anyhow::Result<()> {
    info!("Adding a new lambda to the SAM-E environment");

    check_init()?;
    let config = get_config()?;

    let template_locations = config.get_runtime().get_templates();
    let resources = parse_templates_into_resources(template_locations)?;

    let mut current_lambdas = config.get_lambdas().clone();
    let current_lambda_names = current_lambdas
        .iter()
        .map(|lambda| lambda.get_name())
        .collect::<Vec<_>>();
    let non_used_lambdas = get_lambdas_from_resources(&resources)?
        .into_iter()
        .filter(|lambda| !current_lambda_names.contains(&lambda.get_name()))
        .collect::<Vec<_>>();

    let lambda_choices = non_used_lambdas
        .iter()
        .map(|lambda| lambda.get_name())
        .collect::<Vec<_>>();

    let selection = dialoguer::MultiSelect::new()
        .with_prompt("Select the lambdas you want to add")
        .items(&lambda_choices)
        .interact()?;

    let mut new_lambdas: Vec<Lambda> = vec![];
    for index in selection {
        let lambda = &non_used_lambdas[index];
        new_lambdas.push(lambda.clone());
    }
    let lambdas_with_env = specify_environment_vars(new_lambdas);
    let mut lambdas_with_builds = add_build_settings(lambdas_with_env);
    current_lambdas.append(&mut lambdas_with_builds);

    let new_config = Config::new(
        current_lambdas,
        config.get_runtime().clone(),
        config.get_infrastructure().clone(),
        None,
    );
    let config_string = serde_yaml::to_string(&new_config)?;

    let sam_e_directory_path = get_sam_e_directory_path()?;
    let sam_e_config_path = format!("{}/sam-e-config.yaml", sam_e_directory_path);
    info!("Updating SAM-E config file at: {:?}", sam_e_config_path);

    fs::write(&sam_e_config_path, config_string)?;
    debug!("SAM-E config file updated successfully");

    Ok(())
}
