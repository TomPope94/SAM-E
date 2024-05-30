use serde::{Deserialize, Serialize};
use serde_yaml::Value;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct EventRule {
    pub description: Option<Value>,
    pub event_bus_name: Option<Value>,
    pub event_pattern: EventPattern,
    pub name: Option<Value>,
    pub role_arn: Option<Value>,
    pub schedule_expression: Option<Value>,
    pub state: Option<Value>,
    pub targets: Vec<Target>,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Target {
    pub arn: Value,
    pub id: Value,
    pub input: Option<Value>,
    pub input_path: Option<Value>,
    pub role_arn: Option<Value>,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct EventPattern {
    pub account: Option<Vec<Value>>,
    pub detail: Option<Value>,
    pub detail_type: Option<Vec<Value>>,
    pub id: Option<Vec<Value>>,
    pub region: Option<Vec<Value>>,
    pub resources: Option<Vec<Value>>,
    pub source: Option<Vec<Value>>,
    pub time: Option<Vec<Value>>,
    pub version: Option<Vec<Value>>,
}
