use reqwest::Client;
use sam_e_types::config::{lambda::Lambda, Config};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct ApiState {
    pub lambdas: Vec<Lambda>,
    pub client: Client,
}

impl ApiState {
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

#[derive(Debug)]
pub enum ContentType {
    Json,
    Html,
    Text,
}

impl FromStr for ContentType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("text/html") {
            return Ok(ContentType::Html);
        }
        if s.contains("application/json") {
            return Ok(ContentType::Json);
        }
        if s.contains("text/plain") {
            return Ok(ContentType::Text);
        }

        Err(())
    }
}
