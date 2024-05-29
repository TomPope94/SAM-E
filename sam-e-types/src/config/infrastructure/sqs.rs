use super::triggers::Triggers;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct QueueInfrastructure {
    pub name: String,
    pub template_name: String,
    pub queue_url: Option<String>,
    pub triggers: Option<Triggers>,
}

pub struct QueueBuilder {
    name: Option<String>,
    template_name: Option<String>,
    queue_url: Option<String>,
    triggers: Option<Triggers>,
}

impl QueueBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            template_name: None,
            queue_url: None,
            triggers: None,
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

    pub fn queue_url(mut self, queue_url: String) -> Self {
        self.queue_url = Some(queue_url);
        self
    }

    pub fn triggers(mut self, triggers: Triggers) -> Self {
        self.triggers = Some(triggers);
        self
    }

    pub fn build(self) -> Result<QueueInfrastructure> {
        let name = self.name.ok_or_else(|| anyhow!("Name is required"))?;
        let template_name = self
            .template_name
            .ok_or_else(|| anyhow!("Template name is required"))?;

        Ok(QueueInfrastructure {
            name,
            template_name,
            queue_url: self.queue_url,
            triggers: self.triggers,
        })
    }
}
