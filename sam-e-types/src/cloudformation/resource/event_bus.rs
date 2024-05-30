use serde::{Deserialize, Serialize};
use serde_yaml::Value;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct EventBus {
    pub dead_letter_config: Option<DeadLetterConfig>,
    pub description: Option<Value>,
    pub event_source_name: Option<Value>,
    pub kms_key_identifier: Option<Value>,
    pub name: Option<Value>,
    pub policy: Option<Value>,
    pub tags: Option<Vec<Tag>>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct DeadLetterConfig {
    pub arn: Option<Value>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Tag {
    pub key: Option<Value>,
    pub value: Option<Value>,
}
