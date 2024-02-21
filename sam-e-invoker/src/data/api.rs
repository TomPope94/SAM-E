use crate::data::store::Store;

use sam_e_types::config::{
    lambda::EventType, Config, Infrastructure, Lambda
};

use fancy_regex::Regex;

#[derive(Debug, Clone)]
pub struct ApiState {
    pub invocation_store: Store,
    pub lambdas: Vec<Lambda>,
    pub infrastructure: Vec<Infrastructure>,
}

impl ApiState {
    pub async fn new(config: &Config) -> Self {
        let lambdas = config.get_lambdas();
        let infrastructure = config.get_infrastructure();

        Self {
            invocation_store: Store::new(lambdas).await,
            lambdas: lambdas.to_owned(),
            infrastructure: infrastructure.to_owned(),
        }
    }

    pub fn get_api_lambdas(&self) -> Vec<&Lambda> {
        self.lambdas.iter().filter(|l| {
            l.get_events()
                .into_iter()
                .any(|e| e.get_event_type() == &EventType::Api)
        }).collect()
    }

    pub fn get_store(&self) -> &Store {
        &self.invocation_store
    }
}

#[derive(Debug, Clone)]
pub struct Route {
    pub route: String,
    pub method: String,
    pub container_name: String,
    pub route_regex: Regex,
    pub route_base_path: Option<String>,
}

impl Route {
    pub fn create(
        route: String,
        method: String,
        container_name: String,
        route_regex: Regex,
        route_base_path: Option<String>,
    ) -> Self {
        Route {
            route,
            method,
            container_name,
            route_regex,
            route_base_path,
        }
    }
    pub fn get_route_base_path(&self) -> Option<&str> {
        self.route_base_path.as_deref()
    }
}
