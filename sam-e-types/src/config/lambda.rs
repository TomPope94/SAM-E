pub mod docker;
pub mod event;

use docker::DockerBuild;
use event::Event;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum PackageType {
    Image,
}

/// A Lambda function as specified in the SAM template - will be created as a separate container
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Lambda {
    name: String,
    image: String,
    environment_vars: HashMap<String, String>,
    events: Vec<Event>,
    template_name: String,
    package_type: PackageType,
    docker_build: Option<DockerBuild>,
}

impl Lambda {
    pub fn new(
        name: String,
        image: String,
        environment_vars: HashMap<String, String>,
        events: Vec<Event>,
        template_name: &str,
        package_type: PackageType,
        docker_build: Option<DockerBuild>,
    ) -> Self {
        Self {
            name,
            image,
            environment_vars,
            events,
            template_name: template_name.to_string(),
            package_type,
            docker_build,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_image(&self) -> &str {
        &self.image
    }

    pub fn set_environment_vars(&mut self, environment_vars: HashMap<String, String>) {
        self.environment_vars = environment_vars;
    }

    pub fn get_environment_vars(&self) -> &HashMap<String, String> {
        &self.environment_vars
    }

    pub fn get_environment_vars_as_value(&self) -> HashMap<String, serde_yaml::Value> {
        self.environment_vars
            .iter()
            .map(|(key, value)| (key.clone(), serde_yaml::Value::String(value.clone())))
            .collect()
    }

    pub fn add_environment_var(&mut self, key: String, value: String) {
        self.environment_vars.insert(key, value);
    }

    pub fn remove_environment_var(&mut self, key: &str) {
        self.environment_vars.remove(key);
    }

    pub fn add_event(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn set_events(&mut self, events: Vec<Event>) {
        self.events = events;
    }

    pub fn get_events(&self) -> &Vec<Event> {
        &self.events
    }

    pub fn get_template_name(&self) -> &str {
        &self.template_name
    }

    pub fn get_package_type(&self) -> &PackageType {
        &self.package_type
    }

    pub fn get_docker_build(&self) -> Option<&DockerBuild> {
        self.docker_build.as_ref()
    }
    pub fn set_docker_build(&mut self, docker_build: DockerBuild) {
        self.docker_build = Some(docker_build);
    }
}
