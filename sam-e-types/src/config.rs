pub mod infrastructure;
pub mod lambda;
pub mod runtime;

pub use infrastructure::Infrastructure;
pub use lambda::Lambda;

use serde::{Deserialize, Serialize};
use infrastructure::Triggers;
use lambda::EventProperties;

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
<<<<<<< HEAD
=======

/// Configuration for the local runtime
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Runtime {
    template_locations: Vec<String>,
    separate_infrastructure: bool,
}

impl Default for Runtime {
    fn default() -> Self {
        Self {
            template_locations: vec![], // Default to empty
            separate_infrastructure: true,
        }
    }
}

impl Runtime {
    pub fn get_template_locations(&self) -> &Vec<String> {
        &self.template_locations
    }

    pub fn get_separate_infrastructure(&self) -> bool {
        self.separate_infrastructure
    }
}

pub struct RuntimeBuilder {
    template_locations: Vec<String>,
    separate_infrastructure: bool,
}

impl RuntimeBuilder {
    pub fn new() -> Self {
        Self {
            template_locations: vec![],
            separate_infrastructure: true,
        }
    }

    pub fn with_template_locations(mut self, template_locations: Vec<String>) -> Self {
        self.template_locations = template_locations;
        self
    }

    pub fn with_separate_infrastructure(mut self, separate_infrastructure: bool) -> Self {
        self.separate_infrastructure = separate_infrastructure;
        self
    }

    pub fn build(self) -> Runtime {
        Runtime {
            template_locations: self.template_locations,
            separate_infrastructure: self.separate_infrastructure,
        }
    }
}

>>>>>>> main
