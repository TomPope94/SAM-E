use serde::Deserialize;
use std::collections::HashMap;

use crate::cloudformation::Resource;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Template {
    // parameters: Option<HashMap<String, Parameter>>,
    pub resources: HashMap<String, Resource>,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum CloudFormationValue {
    String(String),
    Number(i64),
    #[serde(rename_all = "PascalCase")]
    Ref {
        #[serde(rename = "Ref")]
        ref_: String,
    },
    GetAtt {
        #[serde(rename = "Fn::GetAtt")]
        get_att: String,
    },
    Join {
        #[serde(rename = "Fn::Join")]
        join: (String, Vec<CloudFormationValue>),
    },
    Sub {
        #[serde(rename = "Fn::Sub")]
        sub: String,
    },
    #[serde(untagged)]
    Other(serde_yaml::Value),
}

impl CloudFormationValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            CloudFormationValue::String(s) => Some(s.as_str()),
            CloudFormationValue::Ref { ref_ } => Some(ref_.as_str()),
            CloudFormationValue::GetAtt { get_att } => Some(get_att.as_str()),
                // Some(get_att.join(".").as_str()),
            //     Some(get_att[0].as_str())
            // }
            _ => None,
        }
    }

    pub fn to_string(&self) -> Option<String> {
        let as_str_ref = self.as_str();
        match as_str_ref {
            Some(s) => Some(s.to_string()),
            None => None,
        }
    }

    /// Cleans the string for local environment. This means removing ".Arn" from the end of the string for now but further logic can be added.
    pub fn as_local_string(&self) -> Option<String> {
        match self {
            CloudFormationValue::String(s) => Some(s.to_string()),
            CloudFormationValue::Ref { ref_ } => {
                let mut ref_string = ref_.to_string();
                if ref_string.ends_with(".Arn") {
                    ref_string = ref_string.replace(".Arn", "");
                }
                Some(ref_string)
            },
            CloudFormationValue::GetAtt { get_att } => {
                let mut get_att_string = get_att.to_string();
                if get_att_string.ends_with(".Arn") {
                    get_att_string = get_att_string.replace(".Arn", "");
                }
                Some(get_att_string)
            },
            _ => None,
        }
    }
}
