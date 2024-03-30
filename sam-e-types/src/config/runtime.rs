pub mod template;

use serde::{Deserialize, Serialize};
use template::{Template, TemplateBuilder};

/// Configuration for the local runtime
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Runtime {
    separate_infrastructure: bool,
    templates: Vec<Template>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self {
            separate_infrastructure: true,
            templates: vec![], // Default to empty
        }
    }
}

impl Runtime {
    pub fn get_separate_infrastructure(&self) -> bool {
        self.separate_infrastructure
    }
    pub fn get_templates(&self) -> &Vec<Template> {
        &self.templates
    }
    pub fn add_template(&mut self, template: Template) {
        self.templates.push(template);
    }
    pub fn add_template_str(&mut self, location: &str) {
        self.templates.push(TemplateBuilder::new().with_location(location.to_string()).build());
    }
}

pub struct RuntimeBuilder {
    separate_infrastructure: bool,
    templates: Vec<Template>,
}

impl RuntimeBuilder {
    pub fn new() -> Self {
        Self {
            separate_infrastructure: true,
            templates: vec![],
        }
    }

    pub fn with_templates(mut self, template_locations: Vec<String>) -> Self {
        for location in template_locations {
            self.templates
                .push(TemplateBuilder::new().with_location(location).build());
        }

        self
    }

    pub fn with_separate_infrastructure(mut self, separate_infrastructure: bool) -> Self {
        self.separate_infrastructure = separate_infrastructure;
        self
    }

    pub fn build(self) -> Runtime {
        Runtime {
            separate_infrastructure: self.separate_infrastructure,
            templates: self.templates,
        }
    }
}
