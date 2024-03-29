use serde::Deserialize;
// use crate::cloudformation::template::CloudFormationValue as Value;
use serde_yaml::Value;

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct BasePathMapping {
    base_path: Value,
    domain_name: Value,
    rest_api_id: Value,
    stage: Value,
}

impl BasePathMapping {
    pub fn get_base_path(&self) -> &Value {
        &self.base_path
    }

    pub fn get_domain_name(&self) -> &Value {
        &self.domain_name
    }

    pub fn get_rest_api_id(&self) -> &Value {
        &self.rest_api_id
    }

    pub fn get_stage(&self) -> &Value {
        &self.stage
    }
}
