use std::collections::HashMap;

use crate::config::lambda::docker::DockerBuild;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Frontend {
    name: String,
    docker_build: Option<DockerBuild>,
    port: u16,
    env_vars: HashMap<String, String>,
}

impl Frontend {
    pub fn get_name(&self) -> &String {
        &self.name
    }
}

pub struct FrontendBuilder {
    name: String,
    docker_build: Option<DockerBuild>,
    port: u16,
    env_vars: HashMap<String, String>,
}

impl FrontendBuilder {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            docker_build: None,
            port: 0,
            env_vars: HashMap::new(),
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }
    pub fn with_docker_build(mut self, docker_build: DockerBuild) -> Self {
        self.docker_build = Some(docker_build);
        self
    }
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
    pub fn with_env_vars(mut self, env_vars: HashMap<String, String>) -> Self {
        self.env_vars = env_vars;
        self
    }

    pub fn build(self) -> Frontend {
        Frontend {
            name: self.name,
            docker_build: self.docker_build,
            port: self.port,
            env_vars: self.env_vars,
        }
    }
}
