use serde::{Deserialize, Serialize};
use serde_yaml::Value;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct EventRule {
    pub description: Option<Value>,
    pub event_bus_name: Option<Value>,
    pub event_pattern: Option<Value>,
    pub name: Option<Value>,
    pub role_arn: Option<Value>,
    pub schedule_expression: Option<Value>,
    pub state: Option<Value>,
    pub targets: Vec<Target>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Target {
    pub arn: Option<Value>,
    pub id: Option<Value>,
    pub input: Option<Value>,
    pub input_path: Option<Value>,
    pub role_arn: Option<Value>,
}
