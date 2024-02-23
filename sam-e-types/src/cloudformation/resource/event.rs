use serde::Deserialize;
// use crate::cloudformation::template::CloudFormationValue as Value;
use serde_yaml::Value;

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
#[serde(tag = "Type")]
pub enum Event {
    #[serde(rename = "Api")]
    Api(EventContainer<ApiEvent>),
    #[serde(rename = "SQS")]
    Sqs(EventContainer<SqsEvent>),
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct EventContainer<T> {
    properties: T,
}

impl <T> EventContainer<T> {
    pub fn get_properties(&self) -> &T {
        &self.properties
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
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

#[derive(Deserialize, Debug, PartialEq, Eq)]
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
