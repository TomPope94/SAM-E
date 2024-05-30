use super::triggers::Triggers;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct S3Infrastructure {
    pub name: String,
    pub template_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub triggers: Option<Triggers>,
}

pub struct S3Builder {
    name: Option<String>,
    template_name: Option<String>,
    triggers: Option<Triggers>,
}

impl S3Builder {
    pub fn new() -> Self {
        Self {
            name: None,
            template_name: None,
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

    pub fn triggers(mut self, triggers: Triggers) -> Self {
        self.triggers = Some(triggers);
        self
    }

    pub fn build(self) -> Result<S3Infrastructure> {
        let name = self.name.ok_or_else(|| anyhow!("Name is required"))?;
        let template_name = self
            .template_name
            .ok_or_else(|| anyhow!("Template name is required"))?;

        Ok(S3Infrastructure {
            name,
            template_name,
            triggers: self.triggers,
        })
    }
}
