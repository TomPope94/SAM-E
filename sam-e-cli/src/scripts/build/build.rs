use crate::scripts::{
    build::{
        infrastructure::{create_infrastructure_files, get_infrastructure_from_resources},
        lambda::{get_lambdas_from_resources, select_lambdas, specify_environment_vars},
        template::parse_templates_into_resources,
    },
    init,
};

use sam_e_types::config::Config;

use anyhow::Error;
use std::{env, fs};
use tracing::{debug, error, info};

const SAM_E_DIRECTORY: &str = ".sam-e";

pub fn build() -> anyhow::Result<()> {
    info!("Now building the SAM-E environment...");

    let current_dir = env::current_dir()?;
    debug!("Detected current directory as: {:?}", current_dir);

    let sam_e_directory_path = format!("{}/{}", current_dir.to_str().unwrap(), SAM_E_DIRECTORY);

    // Checks to see if init has been run prior to build
    if fs::metadata(&sam_e_directory_path).is_err() {
        let init_confirm = dialoguer::Confirm::new()
            .with_prompt("SAM-E directory not found, would you like to initialise it now?")
            .interact()?;

        if init_confirm {
            init()?;
        } else {
            error!("SAM-E directory not found, cancelling build...");
            return Ok(());
        }
    }

    // Get the initiated config to grab the template locations
    let current_config_raw =
        fs::read_to_string(format!("{}/sam-e-config.yaml", sam_e_directory_path))?;
    let current_config: Config = serde_yaml::from_str(&current_config_raw)?;
    let template_locations = current_config.get_runtime().get_template_locations();

    let resources = parse_templates_into_resources(template_locations);
    debug!("Resources: {:#?}", resources);

    if let Ok(resources) = resources {
        info!("Collected template resources successfully, now building resources...");

        // Extracts the lambdas ready to be added to the config
        // TODO: Currently overwrites, should merge based on user input
        let lambdas = get_lambdas_from_resources(&resources);
        let chosen_lambdas = select_lambdas(lambdas);
        debug!("Lambdas: {:#?}", chosen_lambdas);
        let lambdas_with_env_vars = specify_environment_vars(chosen_lambdas);

        // Extracts the infrastructure ready to be added to the config
        let infrastructure = get_infrastructure_from_resources(&resources)?;
        debug!("Infrastructure: {:#?}", infrastructure);

        // Reads the current config file
        let current_config_raw =
            fs::read_to_string(format!("{}/sam-e-config.yaml", sam_e_directory_path))?;
        let mut config: Config = serde_yaml::from_str(&current_config_raw)?;

        config.set_infrastructure(infrastructure);
        config.set_lambdas(lambdas_with_env_vars);
        debug!("Config post build: {:#?}", config);

        let config_string = serde_yaml::to_string(&config)?;
        fs::write(
            format!("{}/sam-e-config.yaml", sam_e_directory_path),
            config_string,
        )?;
        debug!("Written config to file");
        debug!("Now creating infrastructure files...");

        // Creates infrastructure files based on config (i.e. dockerfiles, docker-compose, configs etc)
        create_infrastructure_files(&config)
    } else {
        error!("Please check at least one file exists with your template file name and is .yaml before trying again");
        Err(Error::msg("Failed to parse resources"))
    }
}
