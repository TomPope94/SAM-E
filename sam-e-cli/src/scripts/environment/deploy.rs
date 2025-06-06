use tracing::{debug, info, warn};

use crate::scripts::{
    environment::build::infrastructure::create_infrastructure_files,
    template::utils::{get_env_var_additions, get_env_var_removals, get_template_lambda},
    utils::{check_init, get_config, get_sam_e_directory_path},
};
use std::process::Command;

pub fn deploy() -> anyhow::Result<()> {
    info!("Deploying SAM-E environment");
    check_init()?;
    let config = get_config()?;

    debug!("Checking for changes in local environment - NOT IMPLEMENTED YET");
    let mut change_detected = false;
    let lambdas = config.get_lambdas();
    let templates = config.get_runtime().get_templates();
    //
    // for lambda in lambdas {
    //     let template_lambda = get_template_lambda(&lambda, &templates)?;
    //
    //     let local_additions = get_env_var_additions(lambda, &template_lambda);
    //     let local_removals = get_env_var_removals(lambda, &template_lambda);
    //
    //     if let Some(local_additions) = local_additions {
    //         warn!("Local additions detected!");
    //         warn!("Lambda: {}", lambda.get_name());
    //         warn!("Additions: {:?}", local_additions);
    //
    //         change_detected = true;
    //     }
    //
    //     if let Some(local_removals) = local_removals {
    //         warn!("Local removals detected!");
    //         warn!("Lambda: {}", lambda.get_name());
    //         warn!("Removals: {:?}", local_removals);
    //
    //         change_detected = true;
    //     }
    // }
    //
    // let confirm_deploy = if change_detected {
    //     warn!("Changes detected in local environment. We strongly recommend you align your SAM templates with local environment before continuing!");
    //     dialoguer::Confirm::new()
    //         .with_prompt("Are you sure you want to continue with deployment?")
    //         .default(false)
    //         .interact()?
    // } else {
    //     true
    // };
    //
    // if !confirm_deploy {
    //     warn!("Deployment aborted by user");
    //     return Ok(());
    // }

    let env_selection = dialoguer::Select::new()
        .with_prompt("Which environment would you like to deploy?")
        .items(&["Dev", "Prod"])
        .default(0)
        .interact()?;

    match env_selection {
        0 => {
            info!("Deploying to Dev environment");
            create_infrastructure_files(&config)?;
            info!("Infrastructure files created successfully");

            info!("Pushing lambdas to private registry...");
            for lambda in lambdas {
                debug!("Building lambda: {}", lambda.get_name());
                let mut build_sh = Command::new("sh");
                build_sh
                    .arg("-c")
                    .arg(format!("docker compose build {}", lambda.get_name()))
                    .current_dir(get_sam_e_directory_path()?)
                    .status()?;

                debug!("Re-tagging lambda: {}", lambda.get_name());

                let image_with_registry =
                    if let Ok(registry) = config.get_runtime().get_docker_registry() {
                        format!("{}/{}:latest", registry, lambda.get_image())
                    } else {
                        format! {"{}:latest", lambda.get_image()}
                    };
                let mut tag_sh = Command::new("sh");
                tag_sh
                    .arg("-c")
                    .arg(format!(
                        "docker tag {} {}",
                        lambda.get_image(),
                        image_with_registry
                    ))
                    .status()?;

                debug!("Pushing lambda: {}", lambda.get_name());
                let mut push_sh = Command::new("sh");
                push_sh
                    .arg("-c")
                    .arg(format!("docker push {}", image_with_registry))
                    .status()?;
            }
            info!("Lambdas pushed to private registry successfully!");
            info!("Now git pull within the dev environment and run `sam-e environment start` to start the dev server");

            return Ok(());
        }
        1 => {
            info!("Deploying to Prod environment");
            warn!("NOT YET IMPLEMENTED");
            return Ok(());
        }
        _ => unreachable!(),
    }
}
