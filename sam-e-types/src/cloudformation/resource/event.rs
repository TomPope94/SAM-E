use serde::{Deserialize, Serialize};
// use crate::cloudformation::template::CloudFormationValue as Value;
use serde_yaml::Value;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub enum EventType {
    #[serde(rename = "Api")]
    Api,
    #[serde(rename = "Sqs")]
    Sqs,
    #[serde(untagged)]
    Other(serde_yaml::Value),
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Event {
    #[serde(rename = "Type")]
    pub event_type: EventType,
    pub properties: Value,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct ApiEvent {
    path: Value,
    method: Value,
    rest_api_id: Option<Value>,
    stage_name: Option<Value>,
}

impl ApiEvent {
    pub fn get_path(&self) -> &Value {
        &self.path
    }

    pub fn get_method(&self) -> &Value {
        &self.method
    }

    pub fn get_rest_api_id(&self) -> &Option<Value> {
        &self.rest_api_id
    }

    pub fn get_stage_name(&self) -> &Option<Value> {
        &self.stage_name
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct SqsEvent {
    queue: Value,
    batch_size: Option<Value>,
}

impl SqsEvent {
    pub fn get_queue(&self) -> &Value {
        &self.queue
    }

    pub fn get_batch_size(&self) -> &Option<Value> {
        &self.batch_size
    }
}
