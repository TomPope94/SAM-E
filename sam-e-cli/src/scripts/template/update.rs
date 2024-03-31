use tracing::{debug, info};

use crate::scripts::{
    template::utils::{
        get_env_var_additions, get_env_var_removals, get_template_lambda, update_template_file,
    },
    utils::{check_init, get_config},
};

pub fn update() -> anyhow::Result<()> {
    info!("Updating the SAM-E template.yaml file");
    check_init()?;
    let config = get_config()?;

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
