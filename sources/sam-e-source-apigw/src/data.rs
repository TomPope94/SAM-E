use reqwest::Client;
use sam_e_types::config::{lambda::Lambda, Config};

#[derive(Debug, Clone)]
pub struct ApiState {
    pub lambdas: Vec<Lambda>,
    pub client: Client,
}

impl ApiState {
    pub fn new(lambdas: Vec<Lambda>) -> Self {
        let client = Client::new();
        Self { lambdas, client }
    }

    pub fn from_config(config: &Config) -> Self {
        let client = Client::new();
        Self {
            lambdas: config.get_lambdas().to_owned(),
            client,
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

    pub fn get_client(&self) -> &Client {
        &self.client
    }
}
