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
    credentials_location: String,
    docker_registry: Option<String>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self {
            templates: vec![], // Default to empty
            use_api_source: false,
            use_queue_source: false,
            use_s3_source: false,
            credentials_location: String::from(""),
            docker_registry: None,
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

    pub fn get_credentials_location(&self) -> &String {
        &self.credentials_location
    }

    pub fn get_docker_registry(&self) -> anyhow::Result<&String> {
        self.docker_registry
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Docker registry is not set"))
    }
}

pub struct RuntimeBuilder {
    templates: Vec<Template>,
    use_api_source: bool,
    use_queue_source: bool,
    use_s3_source: bool,
    credentials_location: Option<String>,
    docker_registry: Option<String>,
}

impl RuntimeBuilder {
    pub fn new() -> Self {
        Self {
            templates: vec![],
            use_api_source: false,
            use_queue_source: false,
            use_s3_source: false,
            credentials_location: None,
            docker_registry: None,
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

    pub fn with_credentials_location(mut self, credentials_location: String) -> Self {
        self.credentials_location = Some(credentials_location);
        self
    }

    pub fn with_docker_registry(mut self, docker_registry: String) -> Self {
        self.docker_registry = Some(docker_registry);
        self
    }

    pub fn build(self) -> Runtime {
        let Some(credentials_location) = self.credentials_location else {
            panic!("Credentials location must be set");
        };

        Runtime {
            templates: self.templates,
            use_api_source: self.use_api_source,
            use_queue_source: self.use_queue_source,
            use_s3_source: self.use_s3_source,
            credentials_location,
            docker_registry: self.docker_registry,
        }
    }
}
