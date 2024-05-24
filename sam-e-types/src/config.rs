pub mod infrastructure;
pub mod lambda;
pub mod runtime;
pub mod frontend;

pub use infrastructure::Infrastructure;
pub use lambda::Lambda;
pub use runtime::Runtime;

use infrastructure::Triggers;
use lambda::event::EventProperties;
use frontend::Frontend;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The overall config construct for the SAM-E environment
/// Will be used to drive the local runtime and the deployment process
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    lambdas: Vec<Lambda>,
    lambda_groups: HashMap<String, Vec<String>>,
    runtime: Runtime,
    infrastructure: Vec<Infrastructure>,
    frontend: Option<Frontend>,
}

impl Config {
    pub fn new(
        lambdas: Vec<Lambda>,
        lambda_groups: HashMap<String, Vec<String>>,
        runtime: Runtime,
        infrastructure: Vec<Infrastructure>,
        frontend: Option<Frontend>,
    ) -> Self {
        Self {
            lambdas,
            lambda_groups,
            runtime,
            infrastructure,
            frontend,
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

    pub fn get_frontend(&self) -> Option<&Frontend> {
        self.frontend.as_ref()
    }

    pub fn set_frontend(&mut self, frontend: Frontend) {
        self.frontend = Some(frontend);
    }

    pub fn get_lambda_groups(&self) -> &HashMap<String, Vec<String>> {
        &self.lambda_groups
    }

    pub fn set_lambda_groups(&mut self, lambda_groups: HashMap<String, Vec<String>>) {
        self.lambda_groups = lambda_groups;
    }
}
