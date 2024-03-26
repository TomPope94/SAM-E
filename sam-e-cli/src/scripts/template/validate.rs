use sam_e_types::config::Config;
use crate::scripts::template::utils::{
    get_env_var_additions, get_env_var_removals, get_template_lambda,
};

use std::{env, fs};
use tracing::{debug, error, info, warn};

const SAM_E_DIRECTORY: &str = ".sam-e";

pub fn validate() -> anyhow::Result<()> {
    info!("Validating the SAM-E template.yaml file");

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
        let template_lambda = get_template_lambda(&lambda, &templates)?;

        let local_additions = get_env_var_additions(lambda, &template_lambda);
        let local_removals = get_env_var_removals(lambda, &template_lambda);

        if let Some(local_additions) = local_additions {
            warn!("Local additions detected!");
            warn!("Lambda: {}", lambda.get_name());
            warn!("Additions: {:?}", local_additions);
        }

        if let Some(local_removals) = local_removals {
            warn!("Local removals detected!");
            warn!("Lambda: {}", lambda.get_name());
            warn!("Removals: {:?}", local_removals);
        }
    }

    info!("Validation complete. Please see above if any warnings were detected.");

    Ok(())
}
