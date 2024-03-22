use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

#[derive(Debug, Deserialize, Serialize)]
pub struct DockerCompose {
    pub version: String,
    pub networks: Option<HashMap<String, Value>>,
    pub volumes: Option<HashMap<String, Value>>,
    pub services: HashMap<String, Value>,
}
