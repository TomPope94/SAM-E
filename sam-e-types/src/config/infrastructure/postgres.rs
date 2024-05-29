use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct PostgresInfrastructure {
    pub name: String,
    pub template_name: String,
}

pub struct PostgresBuilder {
    name: Option<String>,
    template_name: Option<String>,
}

impl PostgresBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            template_name: None,
        }
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn template_name(mut self, template_name: String) -> Self {
        self.template_name = Some(template_name);
        self
    }

    pub fn build(self) -> Result<PostgresInfrastructure> {
        let Some(name) = self.name else {
            return Err(anyhow!("Name is required"));
        };

        let Some(template_name) = self.template_name else {
            return Err(anyhow!("Template name is required"));
        };

        Ok(PostgresInfrastructure {
            name,
            template_name,
        })
    }
}
