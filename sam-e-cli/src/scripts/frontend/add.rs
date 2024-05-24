use std::{collections::HashMap, fs};

use sam_e_types::config::{frontend::FrontendBuilder, lambda::docker::DockerBuildBuilder};
use tracing::{debug, info, warn};

use crate::scripts::utils::{get_config, get_sam_e_directory_path};

pub fn add_frontend() -> anyhow::Result<()> {
    info!("Adding a new frontend to the local environment");
    warn!(
        "This is separate from the SAM templates... please be careful to update those accordingly"
    );

    let mut config = get_config()?;
    let current_frontend = config.get_frontend();
    if current_frontend.is_some() {
        warn!("There is already a frontend in the configuration. Please run `sam-e frontend remove` to remove it first.");
        return Err(anyhow::anyhow!("Frontend already exists"));
    }

    let name = dialoguer::Input::<String>::new()
        .with_prompt("What is the name of the frontend?")
        .interact()?;

    let port = dialoguer::Input::<u16>::new()
        .with_prompt("What port should the frontend run on?")
        .default(5173)
        .interact()?;

    let docker_context = dialoguer::Input::<String>::new()
        .with_prompt("What is the Docker context for the frontend?")
        .interact()?;
    let docker_file = dialoguer::Input::<String>::new()
        .with_prompt("What is the Dockerfile for the frontend (path from context)?")
        .interact()?;
    let docker_build = DockerBuildBuilder::new()
        .with_context(docker_context)
        .with_dockerfile(docker_file)
        .build();

    // TODO: env_vars are overwritten not appended?
    let mut env_vars: HashMap<String, String> = HashMap::new();
    let another_input = true;
    while another_input {
        let key = dialoguer::Input::<String>::new()
            .with_prompt("What is the key for the environment variable?")
            .interact()?;
        let value = dialoguer::Input::<String>::new()
            .with_prompt("What is the value for the environment variable?")
            .interact()?;
        env_vars.insert(key, value);

        let another = dialoguer::Confirm::new()
            .with_prompt("Do you want to add another environment variable?")
            .interact()?;
        if !another {
            break;
        }
    }

    let frontend = FrontendBuilder::new()
        .with_name(name)
        .with_port(port)
        .with_docker_build(docker_build)
        .with_env_vars(env_vars)
        .build();

    config.set_frontend(frontend);
    let config_string = serde_yaml::to_string(&config)?;

    let sam_e_directory_path = get_sam_e_directory_path()?;
    let sam_e_config_path = format!("{}/sam-e-config.yaml", sam_e_directory_path);
    info!("Updating SAM-E config file at: {:?}", sam_e_config_path);

    fs::write(&sam_e_config_path, config_string)?;
    debug!("SAM-E config file updated successfully");

    Ok(())
}
