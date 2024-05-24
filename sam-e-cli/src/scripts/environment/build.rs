pub mod infrastructure;
pub mod lambda;
pub mod template;

use crate::scripts::{
    environment::build::{
        infrastructure::{create_infrastructure_files, get_infrastructure_from_resources},
        lambda::{
            add_build_settings, get_lambdas_from_resources, select_lambdas,
            specify_environment_vars,
        },
        template::parse_templates_into_resources,
    },
    utils::{check_init, get_config, write_config},
};

use sam_e_types::{cloudformation::Resource, config::{runtime::RuntimeBuilder, Lambda}};

use serde::Deserialize;
use tracing::{debug, info};

// TODO: Should probably find a better home for this...
#[derive(Debug, Deserialize)]
pub struct ResourceWithTemplate {
    template_name: String,

    #[serde(flatten)]
    resources: Resource,
}

impl ResourceWithTemplate {
    fn new(resources: Resource, template_name: &str) -> Self {
        Self {
            resources,
            template_name: template_name.to_string(),
        }
    }

    fn get_template_name(&self) -> &str {
        &self.template_name
    }

    fn get_resources(&self) -> &Resource {
        &self.resources
    }
}

// TODO: Should refactor lambdas to be methods on a struct
pub fn build() -> anyhow::Result<()> {
    info!("Now building the SAM-E environment...");

    check_init()?;
    let mut config = get_config()?;

    let template_locations = config.get_runtime().get_templates();

    let resources = parse_templates_into_resources(template_locations)?;
    debug!("Resources: {:#?}", resources);

    info!("Collected template resources successfully, now building resources...");

    let config_lambdas = config.get_lambdas().clone();
    let config_lambdas_name = config_lambdas
        .iter()
        .map(|l| l.get_name())
        .collect::<Vec<_>>();
    if !config_lambdas_name.is_empty() {
        info!("Detected lambdas in config. Note: Selection will be for lambdas not in use only.");
    }

    let resource_lambdas = get_lambdas_from_resources(&resources)?;
    let new_lambdas = resource_lambdas
        .into_iter()
        .filter(|l| !config_lambdas_name.contains(&l.get_name()))
        .collect::<Vec<_>>();

    if new_lambdas.is_empty() {
        info!("No new lambdas found in resources. Exiting...");
        return Ok(());
    }

    let chosen_lambdas = select_lambdas(new_lambdas);
    debug!("Lambdas: {:#?}", chosen_lambdas);

    let lambdas_with_env_vars = specify_environment_vars(chosen_lambdas);
    let lambdas_with_builds = add_build_settings(lambdas_with_env_vars);

    // Extracts the infrastructure ready to be added to the config
    let infrastructure = get_infrastructure_from_resources(&resources)?;
    debug!("Infrastructure: {:#?}", infrastructure);
    config.set_infrastructure(infrastructure);

    let combined_lambdas: Vec<Lambda> = config_lambdas
        .into_iter()
        .chain(lambdas_with_builds)
        .collect();

    let use_api_source = &combined_lambdas
        .iter()
        .any(|l| l.get_events().iter().any(|e| e.get_api_properties().is_some()));
    let use_queue_source = &combined_lambdas
        .iter()
        .any(|l| l.get_events().iter().any(|e| e.get_sqs_properties().is_some()));

    config.set_lambdas(combined_lambdas);
    
    // TODO: this needs a cleanup
    let runtime_clone = config.get_runtime().clone();
    let current_templates_clone = runtime_clone.get_templates().clone();
    let template_locations_as_strings = current_templates_clone
        .iter()
        .map(|t| t.get_location().to_string())
        .collect::<Vec<_>>();

    let new_runtime = RuntimeBuilder::new()
        .with_templates(template_locations_as_strings)
        .with_credentials_location(runtime_clone.get_credentials_location().to_owned())
        .with_use_api_source(*use_api_source)
        .with_use_queue_source(*use_queue_source)
        .with_use_s3_source(false)
        .build();
    
    config.set_runtime(new_runtime);

    debug!("Config post build: {:#?}", config);

    write_config(&config)?;
    debug!("Now creating infrastructure files...");

    // Creates infrastructure files based on config (i.e. dockerfiles, docker-compose, configs etc)
    create_infrastructure_files(&config)
}
