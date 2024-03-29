pub mod infrastructure;
pub mod lambda;
pub mod template;

use crate::scripts::{
    environment::build::{
        infrastructure::{create_infrastructure_files, get_infrastructure_from_resources},
        lambda::{get_lambdas_from_resources, select_lambdas, specify_environment_vars},
        template::parse_templates_into_resources,
    },
    utils::{check_init, get_config, write_config},
};

use sam_e_types::cloudformation::Resource; 

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

pub fn build() -> anyhow::Result<()> {
    info!("Now building the SAM-E environment...");

    check_init()?;
    let mut config = get_config()?;

    let template_locations = config.get_runtime().get_templates();

    let resources = parse_templates_into_resources(template_locations)?;
    debug!("Resources: {:#?}", resources);

    info!("Collected template resources successfully, now building resources...");

    // Extracts the lambdas ready to be added to the config
    // TODO: Currently overwrites, should merge based on user input
    let lambdas = get_lambdas_from_resources(&resources)?;
    let chosen_lambdas = select_lambdas(lambdas);
    debug!("Lambdas: {:#?}", chosen_lambdas);
    let lambdas_with_env_vars = specify_environment_vars(chosen_lambdas);

    // Extracts the infrastructure ready to be added to the config
    let infrastructure = get_infrastructure_from_resources(&resources)?;
    debug!("Infrastructure: {:#?}", infrastructure);

    config.set_infrastructure(infrastructure);
    config.set_lambdas(lambdas_with_env_vars);
    debug!("Config post build: {:#?}", config);

    write_config(&config)?;
    debug!("Now creating infrastructure files...");

    // Creates infrastructure files based on config (i.e. dockerfiles, docker-compose, configs etc)
    create_infrastructure_files(&config)
}
