use serde::{Deserialize, Serialize};

/// Non-triggered infrastructure (i.e. databases, queues, s3 etc.)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Infrastructure {
    name: String,
    infrastructure_type: InfrastructureType,
    triggers: Option<Triggers>,
}

impl Infrastructure {
    pub fn new(name: String, infrastructure_type: InfrastructureType) -> Self {
        Self {
            name,
            infrastructure_type,
            triggers: None,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_infrastructure_type(&self) -> &InfrastructureType {
        &self.infrastructure_type
    }

    pub fn get_triggers(&self) -> &Option<Triggers> {
        &self.triggers
    }

    pub fn get_mut_triggers(&mut self) -> &mut Option<Triggers> {
        &mut self.triggers
    }

    pub fn set_triggers(&mut self, triggers: Triggers) {
        self.triggers = Some(triggers);
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
pub enum InfrastructureType {
    Sqs,
    Postgres,
    Mysql,
    S3,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Triggers {
    lambdas: Option<Vec<String>>,
    queues: Option<Vec<String>>,
}

impl Triggers {
    pub fn new() -> Self {
        Self {
            lambdas: None,
            queues: None,
        }
    }

    pub fn get_lambdas(&self) -> &Option<Vec<String>> {
        &self.lambdas
    }

    pub fn add_lambda(&mut self, lambda: String) {
        if let Some(lambdas) = &mut self.lambdas {
            lambdas.push(lambda);
        } else {
            self.lambdas = Some(vec![lambda]);
        }
    }

    pub fn set_lambdas(&mut self, lambdas: Vec<String>) {
        self.lambdas = Some(lambdas);
    }

    pub fn get_queues(&self) -> &Option<Vec<String>> {
        &self.queues
    }

    pub fn add_queue(&mut self, queue: String) {
        if let Some(queues) = &mut self.queues {
            queues.push(queue);
        } else {
            self.queues = Some(vec![queue]);
        }
    }

    pub fn set_queues(&mut self, queues: Vec<String>) {
        self.queues = Some(queues);
    }
}
