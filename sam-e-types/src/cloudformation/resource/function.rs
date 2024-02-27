use std::collections::HashMap;
use serde::Deserialize;
use crate::cloudformation::{
    resource::Event,
    template::CloudFormationValue as Value,
};
// use serde_yaml::Value;

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Function {
    package_type: Option<Value>,
    image_uri: Option<Value>,
    events: HashMap<String, Event>,
    environment: Option<Environment>,
}

impl Function {
    pub fn get_package_type(&self) -> &Option<Value> {
        &self.package_type
    }

    pub fn get_image_uri(&self) -> &Option<Value> {
        &self.image_uri
    }

    pub fn get_events(&self) -> &HashMap<String, Event> {
        &self.events
    }

    pub fn get_environment(&self) -> &Option<Environment> {
        &self.environment
    }

    pub fn get_environment_owned(self) -> Environment {
        if let Some(env) = self.environment {
            env
        } else {
            Environment::default()
        }
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Environment {
    variables: HashMap<String, Value>,
}

impl Environment {
    pub fn default() -> Environment {
        Environment {
            variables: HashMap::new(),
        }
    }

    pub fn get_environment_vars(&self) -> &HashMap<String, Value> {
        &self.variables
    }
}
