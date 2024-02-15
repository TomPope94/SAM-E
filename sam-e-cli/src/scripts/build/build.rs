use crate::{
    data::cli::BuildArgs,
    scripts::{
        build::utils::{collect_template_to_resource, get_lambdas_from_resources, get_infrastructure_from_resources},
        init,
    },
};

use sam_e_types::config::{Config, Infrastructure, InfrastructureType};

use anyhow::Error;
use rust_embed::RustEmbed;
use std::{env, fs};
use tera::{Context, Tera};
use tracing::{debug, error, info};

const DEFAULT_TEMPLATE: &str = "template.yaml";
const SAM_E_DIRECTORY: &str = ".sam-e";

#[derive(RustEmbed)]
#[folder = "assets/templates/"]
struct Asset;

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

        let infrastructure = get_infrastructure_from_resources(&resources);
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
        debug!("Now creating docker compose");

        create_infrastructure_files(&config)
    } else {
        error!("Please check at least one file exists with your template file name and is .yaml before trying again");
        Err(Error::msg("Failed to parse resources"))
    }
}

fn create_infrastructure_files(config: &Config) -> anyhow::Result<()> {
    let infrastructure = config.get_infrastructure();

    if let true = has_infrastructure_type(infrastructure, InfrastructureType::S3) { 
        info!("Detected S3 infrastructure. Creating required files within .sam-e directory");
        fs::create_dir_all(format!("{}/local-s3", SAM_E_DIRECTORY))?;
    }

    if let true = has_infrastructure_type(infrastructure, InfrastructureType::Sqs) {
        info!("Detected SQS infrastructure. Creating required files within .sam-e directory");
        fs::create_dir_all(format!("{}/local-queue", SAM_E_DIRECTORY))?;
    }

    let mut tera = Tera::default();
    let mut context = Context::new();

    context.insert("lambdas", config.get_lambdas());
    context.insert("infrastructure", config.get_infrastructure());

    if let Some(s3_dockerfile) = Asset::get("local-s3/Dockerfile") {
        let raw_data = s3_dockerfile.data;
        tera.add_raw_template("s3-dockerfile", &String::from_utf8_lossy(&raw_data))?;
    } else {
        error!("Failed to find S3 Dockerfile template");
        return Err(Error::msg("Failed to find S3 Dockerfile template"))
    };

    if let Some(queue_config) = Asset::get("local-queue/custom.conf") {
        let raw_data = queue_config.data;
        tera.add_raw_template("queue-config", &String::from_utf8_lossy(&raw_data))?;
    } else {
        error!("Failed to find queue config template");
        return Err(Error::msg("Failed to find queue config template"))
    };

    if let Some(docker_template) = Asset::get("docker-compose.yaml") {
        let raw_data = docker_template.data;
        tera.add_raw_template("docker-compose", &String::from_utf8_lossy(&raw_data))?;
    } else {
        error!("Failed to find docker-compose template");
        return Err(Error::msg("Failed to find docker-compose template"))
    };

    create_s3_dockerfile(&tera, &context)?;
    create_queue_config(&tera, &context)?;
    create_docker_compose(&tera, &context)?;
    Ok(())
}

fn has_infrastructure_type(infrastructure: &Vec<Infrastructure> , infrastructure_type: InfrastructureType) -> bool {
    infrastructure.iter().any(|i| i.get_infrastructure_type() == &infrastructure_type)
} 

fn create_docker_compose(tera: &Tera, context: &Context) -> anyhow::Result<()> {
        let result = tera.render("docker-compose", &context)?;

        fs::write(format!("{}/docker-compose.yaml", SAM_E_DIRECTORY), result)?;

        Ok(())
}

fn create_s3_dockerfile(tera: &Tera, context: &Context) -> anyhow::Result<()> {
    let result = tera.render("s3-dockerfile", &context)?;

    fs::write(format!("{}/local-s3/Dockerfile", SAM_E_DIRECTORY), result)?;

    Ok(())
}

fn create_queue_config(tera: &Tera, context: &Context) -> anyhow::Result<()> {
    // NOTE: this doesn't work with VMs becuase the file is written to the host machine so can't be
    // passed as volume in dockerfile...
    let result = tera.render("queue-config", &context)?;

    fs::write(format!("{}/local-queue/custom.conf", SAM_E_DIRECTORY), result)?;

    Ok(())
}
