use serde::{Deserialize, Serialize};

/// Non-triggered infrastructure (i.e. databases, queues, s3 etc.)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Infrastructure {
    name: String,
    infrastructure_type: InfrastructureType,
    #[serde(skip_serializing_if = "Option::is_none")]
    triggers: Option<Triggers>,
    #[serde(skip_serializing_if = "Option::is_none")]
    queue_url: Option<String>,
}

impl Infrastructure {
    pub fn new(name: String, infrastructure_type: InfrastructureType) -> Self {
        Self {
            name,
            infrastructure_type,
            triggers: None,
            queue_url: None,
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

    pub fn get_lambda_triggers(&self) -> &Option<Vec<String>> {
        if let Some(triggers) = &self.triggers {
            triggers.get_lambdas()
        } else {
            &None
        }
    }

    pub fn add_lambda_to_triggers(&mut self, lambda: String) {
        if let Some(triggers) = &mut self.triggers {
            triggers.add_lambda(lambda);
        } else {
            let mut new_triggers = Triggers::new();
            new_triggers.add_lambda(lambda);
            self.triggers = Some(new_triggers);
        }
    }

    pub fn add_queue_to_triggers(&mut self, queue: String) {
        if let Some(triggers) = &mut self.triggers {
            triggers.add_queue(queue);
        } else {
            let mut new_triggers = Triggers::new();
            new_triggers.add_queue(queue);
            self.triggers = Some(new_triggers);
        }
    }

    pub fn get_queue_url(&self) -> &Option<String> {
        &self.queue_url
    }

    pub fn set_queue_url(&mut self, url: String) {
        self.queue_url = Some(url);
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
    #[serde(skip_serializing_if = "Option::is_none")]
    lambdas: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
