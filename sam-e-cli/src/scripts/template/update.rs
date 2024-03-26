use sam_e_types::config::Config;

use std::{env, fs};
use tracing::{debug, error, info};

use crate::scripts::template::utils::{
    get_env_var_additions, get_env_var_removals, get_template_lambda, update_template_file,
};

const SAM_E_DIRECTORY: &str = ".sam-e";

pub fn update() -> anyhow::Result<()> {
    info!("Updating the SAM-E template.yaml file");
    let current_dir = env::current_dir()?;
    debug!("Detected current directory as: {:?}", current_dir);

    let sam_e_directory_path = format!("{}/{}", current_dir.to_str().unwrap(), SAM_E_DIRECTORY);

    // Checks to see if init has been run prior to build
    if fs::metadata(&sam_e_directory_path).is_err() {
        error!("SAM-E directory not found, please run 'sam-e init' and/or 'sam-e build' before rebuilding.");

        return Ok(());
    }

    // Reads the current config file
    let current_config_raw =
        fs::read_to_string(format!("{}/sam-e-config.yaml", sam_e_directory_path))?;
    let config: Config = serde_yaml::from_str(&current_config_raw)?;

    let lambdas = config.get_lambdas();
    let templates = config.get_runtime().get_templates();

    for lambda in lambdas {
        let mut template_lambda = get_template_lambda(&lambda, &templates)?;

        let local_additions = get_env_var_additions(lambda, &template_lambda);
        let local_removals = get_env_var_removals(lambda, &template_lambda);

        if let Some(local_additions) = local_additions {
            debug!("Local additions: {:?}", local_additions);
            for (key, value) in local_additions.iter() {
                template_lambda.add_environment_var(key.to_string(), value.to_string());
            }
        }

        if let Some(local_removals) = local_removals {
            debug!("Local removals: {:?}", local_removals);
            for key in local_removals {
                template_lambda.remove_environment_var(&key);
            }
        }

        update_template_file(&template_lambda, templates)?;
    }

    Ok(())
}
