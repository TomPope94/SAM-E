use std::process::Command;
use tracing::{debug, info, warn};

use crate::scripts::{template::utils::{
    get_env_var_additions, get_env_var_removals, get_template_lambda,
}, utils::{check_init, get_config}};

pub fn deploy() -> anyhow::Result<()> {
    info!("Deploying SAM-E environment");
    check_init()?;
    let config = get_config()?;

    debug!("Checking for changes in local environment");
    let mut change_detected = false;
    let lambdas = config.get_lambdas();
    let templates = config.get_runtime().get_templates();

    for lambda in lambdas {
        let template_lambda = get_template_lambda(&lambda, &templates)?;

        let local_additions = get_env_var_additions(lambda, &template_lambda);
        let local_removals = get_env_var_removals(lambda, &template_lambda);

        if let Some(local_additions) = local_additions {
            warn!("Local additions detected!");
            warn!("Lambda: {}", lambda.get_name());
            warn!("Additions: {:?}", local_additions);
            
            change_detected = true;
        }

        if let Some(local_removals) = local_removals {
            warn!("Local removals detected!");
            warn!("Lambda: {}", lambda.get_name());
            warn!("Removals: {:?}", local_removals);

            change_detected = true;
        }
    }

    let confirm_deploy = if change_detected {
        warn!("Changes detected in local environment. We strongly recommend you align your SAM templates with local environment before continuing!");
        dialoguer::Confirm::new()
            .with_prompt("Are you sure you want to continue with deployment?")
            .default(false)
            .interact()?
    } else {
        true
    };

    if !confirm_deploy {
        warn!("Deployment aborted by user");
        return Ok(());
    }

    let templates = config.get_runtime().get_templates().iter().map(|t| t.get_location()).collect::<Vec<&str>>();
    let selection = dialoguer::Select::new()
        .with_prompt("Select template to deploy. If nesting, choose the root template.")
        .items(&templates)
        .default(0)
        .interact()?;

    let mut sh = Command::new("sh");
    sh.arg("-c")
        .arg("sam init")
        .arg("--location")
        .arg(templates[selection])
        .status()?;

    Ok(())
}
