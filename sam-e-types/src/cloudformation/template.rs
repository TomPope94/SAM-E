use serde::Deserialize;
use std::collections::HashMap;

use crate::cloudformation::Resource;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Template {
    // parameters: Option<HashMap<String, Parameter>>,
    pub resources: HashMap<String, Resource>,
}

// #[derive(Deserialize, Debug, PartialEq, Eq)]
// #[serde(untagged)]
// pub enum CloudFormationValue {
//     String(String),
//     Number(i64),
//     #[serde(rename_all = "PascalCase")]
//     Ref {
//         r#ref: String
//     },
//     GetAtt {
//         #[serde(rename = "Fn::GetAtt")]
//         get_att: Vec<String>,
//     },
//     Join {
//         #[serde(rename = "Fn::Join")]
//         join: (String, Vec<CloudFormationValue>),
//     },
//     Sub {
//         #[serde(rename = "Fn::Sub")]
//         sub: String,
//     },
//     Other(serde_yaml::Value),
// }

// impl CloudFormationValue {
//     pub fn as_str(&self) -> Option<&str> {
//         match self {
//             CloudFormationValue::String(s) => Some(s.as_str()),
//             _ => None,
//         }
//     }
//
//     pub fn to_string(&self) -> String {
//         self.as_str().unwrap().to_string()
//     }
// }
