use crate::scripts::{
    template::utils::{get_env_var_additions, get_env_var_removals, get_template_lambda},
    utils::{check_init, get_config},
};

use tracing::{info, warn};

pub fn validate() -> anyhow::Result<()> {
    info!("Validating the SAM-E template.yaml file");

    check_init()?;
    let config = get_config()?;

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
