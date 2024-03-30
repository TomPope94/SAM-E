use sam_e_types::config::infrastructure::InfrastructureType;
use std::process::Command;
use tracing::{debug, error, info, warn};

use crate::{data::cli::StartArgs, scripts::utils::{check_init, get_config, get_sam_e_directory_path}};

pub async fn start(args: StartArgs) -> anyhow::Result<()> {
    debug!("Starting the SAM-E environment...");

    check_init()?;

    info!("Reading the current configuration");

    let config = get_config()?;

    let selection = dialoguer::Select::new()
        .with_prompt("Which part of the envioronment would you like to start?")
        .items(&["Infrastructure", "Functions", "All"])
        .default(0)
        .interact()?;

    let mut docker_cmd = match selection {
        0 => {
            info!("Starting the infrastructure...");
            let infrastructure = config.get_infrastructure();

            let mut cmd_str =
                "docker compose --compatibility up --remove-orphans --build ".to_string();

            let mut use_s3 = false;
            let mut use_postgres = false;
            let mut use_sqs = false;
            for service in infrastructure {
                let service_type = service.get_infrastructure_type();
                match service_type {
                    InfrastructureType::S3 => {
                        if !use_s3 {
                            cmd_str.push_str("s3-local ");
                        }
                        use_s3 = true;
                    }
                    InfrastructureType::Postgres => {
                        if !use_postgres {
                            cmd_str.push_str("postgres-local ");
                        }
                        use_postgres = true;
                    }
                    InfrastructureType::Sqs => {
                        if !use_sqs {
                            cmd_str.push_str("sqs-local ");
                        }
                        use_sqs = true;
                    }
                    _ => {
                        warn!("Unsupported infrastructure type: {:?}", service_type);
                        continue;
                    }
                }
            }

            cmd_str
        }
        1 => {
            info!("Starting the functions...");
            let functions = config.get_lambdas();

            let mut cmd_str =
                "docker compose --compatibility up --remove-orphans --build ".to_string();
            for function in functions {
                cmd_str.push_str(function.get_name());
                cmd_str.push(' ');
            }

            cmd_str
        }
        2 => {
            info!("Starting the entire environment...");
            "docker compose --compatibility up --remove-orphans --build".to_string()
        }
        _ => {
            error!("Invalid selection, cancelling start...");
            return Err(anyhow::Error::msg("Invalid selection"));
        }
    };

    if args.detached {
        docker_cmd.push_str(" -d");
    }

    debug!("Running the docker command: {}", docker_cmd);

    let mut sh = Command::new("sh");

    let config_string = serde_yaml::to_string(&config)?;
    sh.arg("-c")
        // .arg(config_arg)
        .env("CONFIG", config_string)
        .current_dir(get_sam_e_directory_path()?)
        .arg(docker_cmd)
        .status()?;

    Ok(())
}
