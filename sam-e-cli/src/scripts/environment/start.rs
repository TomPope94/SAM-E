use sam_e_types::config::infrastructure::Infrastructure;
use std::process::Command;
use tracing::{debug, error, info, warn};

use crate::{
    data::cli::StartArgs,
    scripts::utils::{check_init, get_config, get_sam_e_directory_path},
};

pub async fn start(args: StartArgs) -> anyhow::Result<()> {
    debug!("Starting the SAM-E environment...");

    check_init()?;

    info!("Reading the current configuration");

    let config = get_config()?;

    let env_selection = dialoguer::Select::new()
        .with_prompt("Which environment would you like to start?")
        .items(&["Local", "Dev"])
        .default(0)
        .interact()?;

    let selection = dialoguer::Select::new()
        .with_prompt("Which part of the environment would you like to start?")
        .items(&[
            "Infrastructure",
            "All Functions",
            "Function Group",
            "Frontend",
            "All",
        ])
        .default(0)
        .interact()?;

    let mut docker_cmd = match selection {
        0 => {
            info!("Starting the infrastructure...");
            let infrastructure = config.get_infrastructure();

            let mut cmd_str =
                "docker compose --compatibility up --remove-orphans sam-e-invoker ".to_string();

            let mut use_s3 = false;
            let mut use_postgres = false;
            let mut use_sqs = false;
            for service in infrastructure {
                match service {
                    Infrastructure::S3(_) => {
                        if !use_s3 {
                            cmd_str.push_str("s3-local ");
                        }
                        use_s3 = true;
                    }
                    Infrastructure::Postgres(_) => {
                        if !use_postgres {
                            cmd_str.push_str("postgres-local ");
                        }
                        use_postgres = true;
                    }
                    Infrastructure::Sqs(_) => {
                        if !use_sqs {
                            cmd_str.push_str("sqs-local ");
                        }
                        use_sqs = true;
                    }
                    _ => {
                        warn!("Unsupported infrastructure type detected. Skipping...");
                        continue;
                    }
                }
            }

            cmd_str
        }
        1 => {
            info!("Starting the functions...");
            let functions = config.get_lambdas();

            let mut cmd_str = "docker compose --compatibility up --remove-orphans ".to_string();
            for function in functions {
                cmd_str.push_str(function.get_name());
                cmd_str.push(' ');
            }

            cmd_str
        }
        2 => {
            let function_groups = config.get_lambda_groups();
            let function_group_names = function_groups.keys().collect::<Vec<_>>();

            if function_group_names.is_empty() {
                error!("No function groups found. Please create a function group first using `sam-e function group create`");
                return Err(anyhow::Error::msg("No function groups found"));
            }

            let selection = dialoguer::Select::new()
                .with_prompt("Which function group would you like to start?")
                .items(&function_group_names)
                .default(0)
                .interact()?;
            let function_group_name = function_group_names[selection].clone();
            let chosen_group = function_groups.get(&function_group_name).unwrap();

            if chosen_group.is_empty() {
                error!("No functions found in the group. Exiting...");
                return Err(anyhow::Error::msg("No functions found in the group"));
            }

            info!("Starting all functions for group: {}", function_group_name);

            let mut cmd_str = "docker compose --compatibility up --remove-orphans ".to_string();
            for function_name in chosen_group {
                cmd_str.push_str(function_name);
                cmd_str.push(' ');
            }

            cmd_str
        }
        3 => {
            info!("Starting the frontend...");
            let frontend = config.get_frontend();

            if let Some(frontend) = frontend {
                let mut cmd_str =
                    "docker compose --compatibility up --remove-orphans frontend_".to_string();
                cmd_str.push_str(frontend.get_name());
                cmd_str.push(' ');

                cmd_str
            } else {
                warn!("No frontend found in the configuration. Please run `sam-e frontend add` to add one.");
                return Err(anyhow::Error::msg("No frontend found in the configuration"));
            }
        }
        4 => {
            info!("Starting all parts of the environment...");
            let cmd_str = "docker compose --compatibility up --remove-orphans ".to_string();
            cmd_str
        }
        _ => {
            error!("Invalid selection, cancelling start...");
            return Err(anyhow::Error::msg("Invalid selection"));
        }
    };

    if args.build {
        docker_cmd.push_str(" --build");
    }

    if args.detached {
        docker_cmd.push_str(" -d");
    }

    debug!("Running the docker command: {}", docker_cmd);

    let mut sh = Command::new("sh");

    let config_string = serde_yaml::to_string(&config)?;
    sh.arg("-c")
        .env("COMPOSE_DOCKER_CLI_BUILD", "1") // Allows individual docker ignore files
        .env("CONFIG", config_string);

    let runtime = config.get_runtime();
    let docker_registry = runtime.get_docker_registry();
    if env_selection == 1 {
        if let Ok(registry) = docker_registry {
            sh.env("REGISTRY", registry);
        }
    }

    if env_selection == 1 {
        sh.current_dir(format!("{}/dev", get_sam_e_directory_path()?));
    } else {
        sh.current_dir(format!("{}/local", get_sam_e_directory_path()?));
    }

    sh.arg(docker_cmd).status()?;

    Ok(())
}
