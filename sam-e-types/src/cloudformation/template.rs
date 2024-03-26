use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt
};

use crate::cloudformation::Resource;

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Template {
    // parameters: Option<HashMap<String, Parameter>>,
    pub resources: HashMap<String, Resource>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub enum CloudFormationValue {
    Ref(String),
    // #[serde(rename_all = "PascalCase")]
    // Ref {
    //     #[serde(rename = "Ref")]
    //     ref_: String,
    // },
    // GetAtt {
    //     #[serde(rename = "Fn::GetAtt")]
    //     get_att: String,
    // },
    // Join {
    //     #[serde(rename = "Fn::Join")]
    //     join: (String, Vec<CloudFormationValue>),
    // },
    // Sub {
    //     #[serde(rename = "Fn::Sub")]
    //     sub: String,
    // },
    #[serde(untagged)]
    String(String),
    #[serde(untagged)]
    Number(i64),
    #[serde(untagged)]
    Other(serde_yaml::Value),
}

impl fmt::Display for CloudFormationValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CloudFormationValue::String(val) => write!(f, "{}", val),
            CloudFormationValue::Number(val) => write!(f, "{}", val),
            CloudFormationValue::Ref(ref_val) => write!(f, "{}", ref_val.replace(".Arn", "")),
            CloudFormationValue::Other(value) => write!(f, "{}", value.as_str().expect("VALUE INCORRECT")),
        }
    }
}
