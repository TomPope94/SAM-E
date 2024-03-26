use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};

use crate::cloudformation::Resource;

// NOTE: all template keys must be parsed so that when we update the template via serde, we don't lose any
// from the original, manually constructed template
#[derive(Deserialize, Debug, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Template {
    // parameters: Option<HashMap<String, Parameter>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transform: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "AWSTemplateFormatVersion")]
    format_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    conditions: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    globals: Option<serde_yaml::Value>,
    pub resources: HashMap<String, Resource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    outputs: Option<serde_yaml::Value>,
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
            CloudFormationValue::Other(value) => {
                write!(f, "{}", value.as_str().expect("VALUE INCORRECT"))
            }
        }
    }
}
