use crate::{
    data::cli::BuildArgs,
    scripts::{
        build::{
            infrastructure::{create_infrastructure_files, get_infrastructure_from_resources},
            lambda::get_lambdas_from_resources,
            template::collect_template_to_resource,
        },
        init,
    },
};

use sam_e_types::config::Config;

use anyhow::Error;
use std::{env, fs};
use tracing::{debug, error, info};

const DEFAULT_TEMPLATE: &str = "template.yaml";
const SAM_E_DIRECTORY: &str = ".sam-e";

pub fn build(args: BuildArgs) -> anyhow::Result<()> {
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

    // Sets default values for args if not provided by the user
    let template_name = args
        .template_name
        .unwrap_or_else(|| DEFAULT_TEMPLATE.to_string());
    let multi = args.multi.unwrap_or_else(|| false);

    debug!("Template name: {}", template_name);
    debug!("Multi: {}", multi);

    // Finds one or more template yaml files, collates them and returns resources sections as a hashmap
    let resources = collect_template_to_resource(&template_name, &multi, &current_dir);

    if let Ok(resources) = resources {
        info!("Collected template resources successfully, now building resources...");

        // Extracts the lambdas ready to be added to the config
        // TODO: Currently overwrites, should merge based on user input
        let lambdas = get_lambdas_from_resources(&resources);
        debug!("Lambdas: {:#?}", lambdas);

        // Extracts the infrastructure ready to be added to the config
        let infrastructure = get_infrastructure_from_resources(&resources)?;
        debug!("Infrastructure: {:#?}", infrastructure);

        // Reads the current config file
        let current_config_raw =
            fs::read_to_string(format!("{}/sam-e-config.yaml", sam_e_directory_path))?;
        let mut config: Config = serde_yaml::from_str(&current_config_raw)?;

        config.set_infrastructure(infrastructure);
        config.set_lambdas(lambdas);
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
