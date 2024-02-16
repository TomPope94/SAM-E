use serde::{Deserialize, Serialize};
use crate::config::{Infrastructure, lambda::{Lambda, EventType, EventProperties}};

/// The overall config construct for the SAM-E environment
/// Will be used to drive the local runtime and the deployment process
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    lambdas: Vec<Lambda>,
    runtime: Runtime,
    infrastructure: Vec<Infrastructure>,
}

impl Config {
    pub fn new(
        lambdas: Vec<Lambda>,
        runtime: Runtime,
        infrastructure: Vec<Infrastructure>,
    ) -> Self {
        Self {
            lambdas,
            runtime,
            infrastructure,
        }
    }

    pub fn set_lambdas(&mut self, lambdas: Vec<Lambda>) {
        // For each lambda, set the infrastructure trigger (if not api event)
        // This makes the invoker more efficient so we don't have to check all lambdas for each event
        for lambda in lambdas.iter() {
            for event in lambda.get_events() {
                match event.get_event_type() {
                    EventType::Sqs => {
                        let queue_name = match event.get_properties() {
                            Some(EventProperties::Sqs(sqs_properties)) => sqs_properties.get_queue().clone(),
                            _ => String::new(),
                        };
                        for infrastructure in self.infrastructure.iter_mut() {
                            if infrastructure.get_name() == queue_name {
                                infrastructure.add_lambda_trigger(lambda.get_name().to_string());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        self.lambdas = lambdas;
    }

    pub fn get_lambdas(&self) -> &Vec<Lambda> {
        &self.lambdas
    }

    pub fn get_infrastructure(&self) -> &Vec<Infrastructure> {
        &self.infrastructure
    }

    pub fn set_infrastructure(&mut self, infrastructure: Vec<Infrastructure>) {
        self.infrastructure = infrastructure;
    }
}

/// Configuration for the local runtime
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Runtime {
    port: u16,
}

impl Default for Runtime {
    fn default() -> Self {
        Self { port: 3000 }
    }
}

impl Runtime {
    pub fn new(port: u16) -> Self {
        Self { port }
    }
}

