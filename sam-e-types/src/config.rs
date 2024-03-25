pub mod infrastructure;
pub mod lambda;
pub mod runtime;

use infrastructure::{Infrastructure, Triggers};
use lambda::{EventProperties, Lambda};
use runtime::Runtime;

use serde::{Deserialize, Serialize};

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
                if let Some(event_properties) = event.get_properties() {
                    match event_properties {
                        EventProperties::Sqs(_) => {
                            let queue_name = match event.get_properties() {
                                Some(EventProperties::Sqs(sqs_properties)) => {
                                    sqs_properties.get_queue().clone()
                                }
                                _ => String::new(),
                            };
                            for infrastructure in self.infrastructure.iter_mut() {
                                if infrastructure.get_name() == queue_name {
                                    if let Some(triggers) = infrastructure.get_mut_triggers() {
                                        triggers.add_lambda(lambda.get_name().to_string());
                                    } else {
                                        let mut triggers = Triggers::new();
                                        triggers.add_lambda(lambda.get_name().to_string());
                                        infrastructure.set_triggers(triggers);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        self.lambdas = lambdas;
    }

    pub fn get_lambdas(&self) -> &Vec<Lambda> {
        &self.lambdas
    }

    pub fn get_runtime(&self) -> &Runtime {
        &self.runtime
    }

    pub fn get_infrastructure(&self) -> &Vec<Infrastructure> {
        &self.infrastructure
    }

    pub fn set_infrastructure(&mut self, infrastructure: Vec<Infrastructure>) {
        self.infrastructure = infrastructure;
    }

    pub fn set_runtime(&mut self, runtime: Runtime) {
        self.runtime = runtime;
    }
}
