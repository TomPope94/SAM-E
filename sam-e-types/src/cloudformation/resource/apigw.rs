use serde::Deserialize;
// use crate::cloudformation::template::CloudFormationValue as Value;
use serde_yaml::Value;

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct ApiGateway {
    name: Value,
    description: Option<Value>,
    stage_name: Option<Value>,
}

impl ApiGateway {
    pub fn get_name(&self) -> &Value {
        &self.name
    }

    pub fn get_description(&self) -> &Option<Value> {
        &self.description
    }

    pub fn get_stage_name(&self) -> &Option<Value> {
        &self.stage_name
    }
}

