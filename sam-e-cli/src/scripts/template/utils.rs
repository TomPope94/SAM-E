use std::{collections::HashMap, fs, path::Path};

use crate::scripts::environment::build::{
    lambda::get_lambdas_from_resources, template::build_template,
};
use sam_e_types::{
    cloudformation::{
        resource::{Function, ResourceType},
        Template as AwsTemplate,
    },
    config::{lambda::Lambda, runtime::template::Template},
};

pub fn get_template_lambda(lambda: &Lambda, templates: &Vec<Template>) -> anyhow::Result<Lambda> {
    let template = lambda.get_template_name();

    let Some(matched_template) = templates.iter().find(|t| t.get_name() == template) else {
        return Err(anyhow::anyhow!("Template not found: {}", template));
    };

    let resources_from_template = build_template(matched_template)?;
    let lambdas_from_template = get_lambdas_from_resources(&resources_from_template)?;

    let Some(matching_lambda) = lambdas_from_template
        .iter()
        .find(|l| l.get_name() == lambda.get_name())
    else {
        return Err(anyhow::anyhow!(
            "Lambda not found in template: {}",
            lambda.get_name()
        ));
    };

    Ok(matching_lambda.clone())
}

pub fn get_env_var_additions(
    local_lambda: &Lambda,
    template_lambda: &Lambda,
) -> Option<HashMap<String, String>> {
    let local_env_vars = local_lambda.get_environment_vars();
    let template_env_vars = template_lambda.get_environment_vars();

    let env_vars_additions = local_env_vars
        .iter()
        .filter(|(key, _)| !template_env_vars.contains_key(*key))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect::<HashMap<_, _>>();

    if env_vars_additions.is_empty() {
        None
    } else {
        Some(env_vars_additions)
    }
}

pub fn get_env_var_removals(
    local_lambda: &Lambda,
    template_lambda: &Lambda,
) -> Option<Vec<String>> {
    let local_env_vars = local_lambda.get_environment_vars();
    let template_env_vars = template_lambda.get_environment_vars();

    let env_vars_removals = template_env_vars
        .iter()
        .filter(|(key, _)| !local_env_vars.contains_key(*key))
        .map(|(key, _)| key.clone())
        .collect::<Vec<String>>();

    if env_vars_removals.is_empty() {
        None
    } else {
        Some(env_vars_removals)
    }
}

pub fn update_template_file(
    updated_lambda: &Lambda,
    templates: &Vec<Template>,
) -> anyhow::Result<()> {
    let matched_template = templates
        .iter()
        .find(|t| t.get_name() == updated_lambda.get_template_name())
        .unwrap();

    let template_path = Path::new(matched_template.get_location());
    let path_as_str = template_path.to_str().unwrap();
    let yaml_file = fs::read_to_string(path_as_str)?;
    let template_yaml: AwsTemplate = serde_yaml::from_str(&yaml_file)?;

    let new_template = update_lambda_env_vars(updated_lambda, &template_yaml)?;
    let new_template_yaml = serde_yaml::to_string(&new_template)?;

    fs::write(path_as_str, new_template_yaml)?;

    Ok(())
}

fn update_lambda_env_vars(
    updated_lambda: &Lambda,
    template: &AwsTemplate,
) -> anyhow::Result<AwsTemplate> {
    let mut template_yaml = template.clone();

    for (key, value) in template_yaml.resources.iter_mut() {
        if value.resource_type == ResourceType::Function && updated_lambda.get_name() == key {
            let mut function: Function = serde_yaml::from_value(value.properties.clone()).unwrap();
            function
                .get_environment_mut()
                .as_mut()
                .unwrap()
                .set_environment_vars(updated_lambda.get_environment_vars_as_value());

            value.properties = serde_yaml::to_value(function).unwrap();
        };
    }

    Ok(template_yaml)
}
