use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Triggers {
    #[serde(skip_serializing_if = "Option::is_none")]
    lambdas: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    queues: Option<Vec<String>>,
}

impl Triggers {
    pub fn new(lambdas: Option<Vec<String>>, queues: Option<Vec<String>>) -> Self {
        Self { lambdas, queues }
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
