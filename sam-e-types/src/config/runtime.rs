use serde::{Deserialize, Serialize};

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


