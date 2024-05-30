use super::triggers::Triggers;
use crate::cloudformation::resource::event_rule::EventPattern as CloudFormationEventPattern;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use tracing::debug;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct EventRuleInfrastructure {
    pub name: String,
    pub template_name: String,
    pub triggers: Option<Triggers>,
    pub event_pattern: EventPattern,
}

pub struct EventRuleBuilder {
    name: Option<String>,
    template_name: Option<String>,
    triggers: Option<Triggers>,
    event_pattern: Option<EventPattern>,
}

impl EventRuleBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            template_name: None,
            triggers: None,
            event_pattern: None,
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

    pub fn event_pattern(mut self, event_pattern: EventPattern) -> Self {
        self.event_pattern = Some(event_pattern);
        self
    }

    pub fn build(self) -> Result<EventRuleInfrastructure> {
        let name = self.name.ok_or_else(|| anyhow!("Name is required"))?;
        let template_name = self
            .template_name
            .ok_or_else(|| anyhow!("Template name is required"))?;

        let event_pattern = self
            .event_pattern
            .ok_or_else(|| anyhow!("Event pattern is required"))?;

        Ok(EventRuleInfrastructure {
            name,
            template_name,
            triggers: self.triggers,
            event_pattern,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct EventPattern {
    pub detail: Option<Value>,
    pub detail_type: Option<Vec<String>>,
    pub source: Option<Vec<String>>,
}

pub struct EventPatternBuilder {
    source: Option<Vec<String>>,
    detail_type: Option<Vec<String>>,
    detail: Option<Value>,
}

impl EventPatternBuilder {
    pub fn new() -> Self {
        Self {
            source: None,
            detail_type: None,
            detail: None,
        }
    }

    pub fn from_cloud_formation(event_pattern: CloudFormationEventPattern) -> Self {
        debug!("Converting CloudFormation EventPattern to Infrastructure EventPattern");
        let source = if let Some(source) = event_pattern.source {
            debug!("Detected source");
            let sources_as_strings: Vec<String> = source
                .into_iter()
                .map(|s| {
                    let source_as_str = s.as_str();
                    if let Some(source_as_str) = source_as_str {
                        source_as_str.to_string()
                    } else {
                        "".to_string()
                    }
                })
                .collect();
            debug!("Sources as strings: {:?}", sources_as_strings);
            Some(sources_as_strings)
        } else {
            None
        };

        let detail_type = if let Some(detail_type) = event_pattern.detail_type {
            debug!("Detected detail type");
            let detail_types_as_strings: Vec<String> = detail_type
                .into_iter()
                .map(|s| {
                    let detail_type_as_str = s.as_str();
                    if let Some(detail_type_as_str) = detail_type_as_str {
                        detail_type_as_str.to_string()
                    } else {
                        "".to_string()
                    }
                })
                .collect();
            debug!("Detail types as strings: {:?}", detail_types_as_strings);
            Some(detail_types_as_strings)
        } else {
            None
        };

        debug!("Parsed cloudformation successfully, now constructing EventPatternBuilder");
        Self {
            source,
            detail_type,
            detail: event_pattern.detail,
        }
    }

    pub fn source(mut self, source: Vec<String>) -> Self {
        self.source = Some(source);
        self
    }

    pub fn detail_type(mut self, detail_type: Vec<String>) -> Self {
        self.detail_type = Some(detail_type);
        self
    }

    pub fn detail(mut self, detail: Value) -> Self {
        self.detail = Some(detail);
        self
    }

    pub fn build(self) -> Result<EventPattern> {
        if let (None, None, None) = (&self.source, &self.detail_type, &self.detail) {
            return Err(anyhow!("At least one of source, detail_type, or detail must be set in your event pattern"));
        }

        debug!("Detected required fields, building EventPattern");
        Ok(EventPattern {
            source: self.source,
            detail_type: self.detail_type,
            detail: self.detail,
        })
    }
}

