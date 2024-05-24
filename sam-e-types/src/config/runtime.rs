pub mod template;

use serde::{Deserialize, Serialize};
use template::{Template, TemplateBuilder};

/// Configuration for the local runtime
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Runtime {
    templates: Vec<Template>,
    use_api_source: bool,
    use_queue_source: bool,
    use_s3_source: bool,
}

impl Default for Runtime {
    fn default() -> Self {
        Self {
            templates: vec![], // Default to empty
            use_api_source: false,
            use_queue_source: false,
            use_s3_source: false,
        }
    }
}

impl Runtime {
    pub fn get_templates(&self) -> &Vec<Template> {
        &self.templates
    }
    pub fn add_template(&mut self, template: Template) {
        self.templates.push(template);
    }
    pub fn add_template_str(&mut self, location: &str) {
        self.templates.push(
            TemplateBuilder::new()
                .with_location(location.to_string())
                .build(),
        );
    }

    pub fn get_use_api_source(&self) -> bool {
        self.use_api_source
    }

    pub fn get_use_queue_source(&self) -> bool {
        self.use_queue_source
    }

    pub fn get_use_s3_source(&self) -> bool {
        self.use_s3_source
    }

    pub fn set_use_api_source(&mut self, use_api_source: bool) {
        self.use_api_source = use_api_source;
    }

    pub fn set_use_queue_source(&mut self, use_queue_source: bool) {
        self.use_queue_source = use_queue_source;
    }

    pub fn set_use_s3_source(&mut self, use_s3_source: bool) {
        self.use_s3_source = use_s3_source;
    }
}

pub struct RuntimeBuilder {
    templates: Vec<Template>,
    use_api_source: bool,
    use_queue_source: bool,
    use_s3_source: bool,
}

impl RuntimeBuilder {
    pub fn new() -> Self {
        Self {
            templates: vec![],
            use_api_source: false,
            use_queue_source: false,
            use_s3_source: false,
        }
    }

    pub fn with_templates(mut self, template_locations: Vec<String>) -> Self {
        for location in template_locations {
            self.templates
                .push(TemplateBuilder::new().with_location(location).build());
        }

        self
    }

    pub fn with_use_api_source(mut self, use_api_source: bool) -> Self {
        self.use_api_source = use_api_source;
        self
    }

    pub fn with_use_queue_source(mut self, use_queue_source: bool) -> Self {
        self.use_queue_source = use_queue_source;
        self
    }

    pub fn with_use_s3_source(mut self, use_s3_source: bool) -> Self {
        self.use_s3_source = use_s3_source;
        self
    }

    pub fn build(self) -> Runtime {
        Runtime {
            templates: self.templates,
            use_api_source: self.use_api_source,
            use_queue_source: self.use_queue_source,
            use_s3_source: self.use_s3_source,
        }
    }
}
