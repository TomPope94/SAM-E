use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

impl EventSqsProperties {
    pub fn get_queue(&self) -> &String {
        &self.queue
    }
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
