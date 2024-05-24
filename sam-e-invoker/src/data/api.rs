use crate::data::store::Store;

use sam_e_types::config::{infrastructure::Infrastructure, lambda::Lambda, Config};

use tracing::{debug, trace};

#[derive(Debug, Clone)]
pub struct ApiState {
    pub invocation_store: Store,
    pub lambdas: Vec<Lambda>,
    pub infrastructure: Vec<Infrastructure>,
}

impl ApiState {
    pub async fn new(config: &Config) -> Self {
        debug!("Creating new API state");

        let lambdas = config.get_lambdas();
        trace!("Lambdas: {:?}", lambdas);

        let infrastructure = config.get_infrastructure();
        trace!("Infrastructure: {:?}", infrastructure);

        Self {
            invocation_store: Store::new(lambdas).await,
            lambdas: lambdas.to_owned(),
            infrastructure: infrastructure.to_owned(),
        }
    }

    pub fn get_api_lambdas(&self) -> Vec<&Lambda> {
        self.lambdas
            .iter()
            .filter(|l| {
                l.get_events()
                    .into_iter()
                    .any(|e| e.get_api_properties().is_some())
            })
            .collect()
    }

    pub fn get_store(&self) -> &Store {
        &self.invocation_store
    }

    pub fn get_infrastructure(&self) -> &Vec<Infrastructure> {
        &self.infrastructure
    }
}
