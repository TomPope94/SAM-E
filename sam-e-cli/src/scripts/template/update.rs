use sam_e_types::{
    cloudformation::{
        resource::{Function, ResourceType},
        Template,
    },
    config::{lambda::Lambda, Config},
};

use std::{env, fs, path::Path};
use tracing::{debug, error, info};

use crate::scripts::build::{lambda::get_lambdas_from_resources, template::build_template};

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

    // Go through each of the lambdas and check environment variable keys match what's in the
    // template
    for lambda in lambdas {
        let env_vars = lambda.get_environment_vars();
        let template = lambda.get_template_name();

        let matched_template = templates
            .iter()
            .find(|t| t.get_name() == template)
            .expect("Template not found");

        let resources_from_template = build_template(matched_template)?;
        let mut lambdas_from_template = get_lambdas_from_resources(&resources_from_template)?;

        let matching_lambda: &mut Lambda = lambdas_from_template
            .iter_mut()
            .find(|l| l.get_name() == lambda.get_name())
            .expect("Lambda not found in template");
        let mut matching_lambda_env_vars = matching_lambda.get_environment_vars().clone();

        for (key, value) in env_vars {
            if !matching_lambda_env_vars.contains_key(key) {
                info!(
                    "Environment variable key '{}' not found in template for lambda '{}'",
                    key,
                    lambda.get_name()
                );
                debug!("Inserting key '{}' with value '{}'", key, value);

                matching_lambda_env_vars.insert(key.to_string(), value.to_string());
            }
        }

        matching_lambda.set_environment_vars(matching_lambda_env_vars);

        let template_path = Path::new(matched_template.get_location());
        let path_as_str = template_path.to_str().unwrap();
        let yaml_file = fs::read_to_string(path_as_str)?;
        let mut template_yaml: Template = serde_yaml::from_str(&yaml_file)?;

        for (key, value) in template_yaml.resources.iter_mut() {
            if value.resource_type == ResourceType::Function && matching_lambda.get_name() == key {
                let mut function: Function = serde_yaml::from_value(value.properties.clone()).unwrap();
                function
                    .get_environment_mut()
                    .as_mut()
                    .unwrap()
                    .set_environment_vars(matching_lambda.get_environment_vars_as_value());

                value.properties = serde_yaml::to_value(function).unwrap();
            };
        }

        let updated_yaml = serde_yaml::to_string(&template_yaml)?;
        fs::write(path_as_str, updated_yaml)?;
    }

    Ok(())
}
