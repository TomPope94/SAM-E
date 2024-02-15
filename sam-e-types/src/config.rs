use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
                            Some(EventProperties::Sqs(sqs_properties)) => sqs_properties.queue.clone(),
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

/// Non-triggered infrastructure (i.e. databases, queues, s3 etc.)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Infrastructure {
    name: String,
    infrastructure_type: InfrastructureType,
    lambda_triggers: Vec<String>,
    queue_url: Option<String>, // TODO this should be a separate type for SQS
}

impl Infrastructure {
    pub fn new(name: String, infrastructure_type: InfrastructureType) -> Self {
        Self {
            name,
            infrastructure_type,
            lambda_triggers: Vec::new(),
            queue_url: None,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_infrastructure_type(&self) -> &InfrastructureType {
        &self.infrastructure_type
    }

    pub fn get_lambda_triggers(&self) -> &Vec<String> {
        &self.lambda_triggers
    }

    pub fn set_lambda_triggers(&mut self, lambda_triggers: Vec<String>) {
        self.lambda_triggers = lambda_triggers;
    }

    pub fn add_lambda_trigger(&mut self, lambda_trigger: String) {
        self.lambda_triggers.push(lambda_trigger);
    }

    pub fn set_queue_url(&mut self, queue_url: String) {
        self.queue_url = Some(queue_url);
    }

    pub fn get_queue_url(&self) -> Option<&String> {
        self.queue_url.as_ref()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
pub enum InfrastructureType {
    Sqs,
    Postgres,
    Mysql,
    S3
}

/// The types of events that can trigger a Lambda
#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
pub enum EventType {
    Api,
    Sqs,
}

/// Properties for an API event
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EventApiProperties {
    path: String,
    base_path: Option<String>,
    method: String,
}

impl EventApiProperties {
    pub fn get_base_path(&self) -> Option<&String> {
        self.base_path.as_ref()
    }

    pub fn get_path(&self) -> &String {
        &self.path
    }

    pub fn get_method(&self) -> &String {
        &self.method
    }
}

/// Properties for an SQS event
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EventSqsProperties {
    queue: String,
}

/// Properties for an event - abstracted to allow for different event types
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum EventProperties {
    Api(EventApiProperties),
    Sqs(EventSqsProperties),
}

/// A Lambda function event as specified in the SAM template
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Event {
    event_type: EventType,
    properties: Option<EventProperties>,
}

impl Event {
    pub fn new(event_type: EventType) -> Self {
        match event_type {
            EventType::Api => Self {
                event_type,
                properties: Some(EventProperties::Api(EventApiProperties {
                    base_path: None,
                    path: String::new(),
                    method: String::new(),
                })),
            },
            EventType::Sqs => Self {
                event_type,
                properties: Some(EventProperties::Sqs(EventSqsProperties {
                    queue: String::new(),
                })),
            },
        }
    }

    pub fn set_api_properties(&mut self, path: String, base_path: Option<String>, method: String) {
        match &mut self.properties {
            Some(EventProperties::Api(api_properties)) => {
                api_properties.base_path = base_path;
                api_properties.path = path;
                api_properties.method = method;
            }
            _ => {}
        }
    }

    pub fn set_sqs_properties(&mut self, queue: String) {
        match &mut self.properties {
            Some(EventProperties::Sqs(sqs_properties)) => {
                sqs_properties.queue = queue;
            }
            _ => {}
        }
    }

    pub fn get_event_type(&self) -> &EventType {
        &self.event_type
    }

    pub fn get_event_type_str(&self) -> &str {
        match self.event_type {
            EventType::Api => "Api",
            EventType::Sqs => "Sqs",
        }
    }

    pub fn get_properties(&self) -> Option<&EventProperties> {
        self.properties.as_ref()
    }
}

/// A Lambda function as specified in the SAM template - will be created as a separate container
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Lambda {
    name: String,
    image: String,
    environment_vars: HashMap<String, String>,
    events: Vec<Event>,
}

impl Lambda {
    pub fn new(
        name: String,
        image: String,
        environment_vars: HashMap<String, String>,
        events: Vec<Event>,
    ) -> Self {
        Self {
            name,
            image,
            environment_vars,
            events,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_image(&self) -> &str {
        &self.image
    }

    pub fn set_environment_vars(&mut self, environment_vars: HashMap<String, String>) {
        self.environment_vars = environment_vars;
    }

    pub fn add_event(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn set_events(&mut self, events: Vec<Event>) {
        self.events = events;
    }

    pub fn get_events(&self) -> &Vec<Event> {
        &self.events
    }
}

