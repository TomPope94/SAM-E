use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Template {
    name: String,
    location: String,
}

impl Template {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_location(&self) -> &str {
        &self.location
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TemplateBuilder {
    name: String,
    location: String,
}

impl TemplateBuilder {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            location: String::new(),
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn with_location(mut self, location: String) -> Self {
        self.location = location;
        self
    }

    pub fn build(mut self) -> Template {
        if self.name.is_empty() {
            self.name = Uuid::new_v4().to_string();
        }

        Template {
            name: self.name,
            location: self.location,
        }
    }
}
