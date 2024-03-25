pub mod template;

use template::{Template, TemplateBuilder};
use serde::{Deserialize, Serialize};

/// Configuration for the local runtime
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Runtime {
    templates: Vec<Template>,
    separate_infrastructure: bool,
}

impl Default for Runtime {
    fn default() -> Self {
        Self {
            templates: vec![], // Default to empty
            separate_infrastructure: true,
        }
    }
}

impl Runtime {
    pub fn get_templates(&self) -> &Vec<Template> {
        &self.templates
    }

    pub fn get_separate_infrastructure(&self) -> bool {
        self.separate_infrastructure
    }
}

pub struct RuntimeBuilder {
    templates: Vec<Template>,
    separate_infrastructure: bool,
}

impl RuntimeBuilder {
    pub fn new() -> Self {
        Self {
            templates: vec![],
            separate_infrastructure: true,
        }
    }

    pub fn with_templates(mut self, template_locations: Vec<String>) -> Self {
        for location in template_locations {
            self.templates.push(TemplateBuilder::new().with_location(location).build());
        }

        self
    }

    pub fn with_separate_infrastructure(mut self, separate_infrastructure: bool) -> Self {
        self.separate_infrastructure = separate_infrastructure;
        self
    }

    pub fn build(self) -> Runtime {
        Runtime {
            templates: self.templates,
            separate_infrastructure: self.separate_infrastructure,
        }
    }
}


