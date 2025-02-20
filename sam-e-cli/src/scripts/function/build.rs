use std::process::Command;
use tracing::{debug, info};

use crate::scripts::utils::{check_init, get_config, get_sam_e_directory_path};

pub fn build() -> anyhow::Result<()> {
    info!("Building function(s)");

    check_init()?;
    let config = get_config()?;

    let build_type_selection = dialoguer::Select::new()
        .with_prompt("Build all or just one specific function?")
        .items(&["All", "Specific Function"])
        .default(0)
        .interact()?;

    let functions = config.get_lambdas();
    if build_type_selection == 0 {
        debug!("Building all functions one by one");

        for function in functions {
            debug!("Building function: {}", function.get_name());

            Command::new("sh")
                .arg("-c")
                .arg(format!("docker compose build {}", function.get_name()))
                .current_dir(get_sam_e_directory_path()?)
                .status()?;

            debug!("Function image built successfully");
        }
        return Ok(());
    }

    debug!("Building a specific function");
    let function_choice = dialoguer::Select::new()
        .with_prompt("Select the function you want to build")
        .items(
            &functions
                .iter()
                .map(|function| function.get_name())
                .collect::<Vec<_>>(),
        )
        .interact()?;

    let function = &functions[function_choice];

    debug!("Building function: {}", function.get_name());

    Command::new("sh")
        .arg("-c")
        .arg(format!("docker compose build {}", function.get_name()))
        .current_dir(get_sam_e_directory_path()?)
        .status()?;

    debug!("Function image built successfully");

    Ok(())
}
