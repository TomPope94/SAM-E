use serde::Deserialize;
// use crate::cloudformation::template::CloudFormationValue as Value;
use serde_yaml::Value;

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Queue {
    queue_name: Option<Value>,
    redrive_policy: Option<Value>,
    visibility_timeout: Option<Value>,
    fifo_queue: Option<Value>,
}

impl Queue {
    pub fn get_queue_name(&self) -> &Option<Value> {
        &self.queue_name
    }
}
