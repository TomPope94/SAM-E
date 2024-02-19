use serde::{Deserialize, Serialize};

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

