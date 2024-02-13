use tracing::{debug, error, info};
use std::{env, fs, process::Command};
use sam_e_types::config::Config;

const SAM_E_DIRECTORY: &str = ".sam-e";

pub async fn start() -> anyhow::Result<()> {
    debug!("Starting the SAM-E environment...");

    let current_dir = env::current_dir()?;
    debug!("Detected current directory as: {:?}", current_dir);

    let sam_e_directory_path = format!("{}/{}", current_dir.to_str().unwrap(), SAM_E_DIRECTORY);

    // Checks to see if init has been run prior to build
    if fs::metadata(&sam_e_directory_path).is_err() {
        error!("SAM-E directory not found, cancelling start...");
        return Err(anyhow::Error::msg(
            "Please run `sam-e init` and `sam-e build` before starting the environment",
        ));
    }

    info!("Reading the current configuration");

    let config_location = format!("{}/sam-e-config.yaml", &sam_e_directory_path);
    let docker_compose_location = format!("{}/docker-compose.yaml", &sam_e_directory_path);

    // Reads the current config file
    let current_config_raw =
        fs::read_to_string(config_location)?;
    let config: Config = serde_yaml::from_str(&current_config_raw)?;
    let config_string = serde_yaml::to_string(&config)?;

    let mut sh = Command::new("sh");
    
    // let docker_cmd = format!("docker compose --compatibility -f {} up --remove-orphans --build", &docker_compose_location);
    let docker_cmd = "docker compose --compatibility up --remove-orphans --build";

    sh.arg("-c")
        // .arg(config_arg)
        .env("CONFIG", config_string)
        .current_dir(sam_e_directory_path)
        .arg(docker_cmd)
        .status()?;

    Ok(())
}
