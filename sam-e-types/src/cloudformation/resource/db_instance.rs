use serde::Deserialize;
use crate::cloudformation::template::CloudFormationValue as Value;
// use serde_yaml::Value;

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct DbInstance {
    engine: Value,
}

impl DbInstance {
    pub fn get_engine(&self) -> &Value {
        &self.engine
    }
}
