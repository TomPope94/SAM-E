use fancy_regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct DockerBuild {
    pub dockerfile: String,
    pub context: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DockerBuildBuilder {
    dockerfile: String,
    context: String,
}

impl DockerBuildBuilder {
    pub fn new() -> Self {
        Self {
            dockerfile: String::new(),
            context: String::new(),
        }
    }

    pub fn with_dockerfile(mut self, dockerfile: String) -> Self {
        self.dockerfile = dockerfile;
        self
    }

    pub fn with_context(mut self, context: String) -> Self {
        self.context = context;
        self
    }

    pub fn build(self) -> DockerBuild {
        DockerBuild {
            dockerfile: self.dockerfile,
            context: self.context,
        }
    }
}

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

/// The types of events that can trigger a Lambda
#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
pub enum EventType {
    Api,
    Sqs,
}

/// Properties for an API event
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct EventApiProperties {
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    base_path: Option<String>,
    method: String,
    route_regex: String,
}

impl EventApiProperties {
    pub fn get_base_path(&self) -> Option<&String> {
        self.base_path.as_ref()
    }

    pub fn get_path(&self) -> &String {
        &self.path
    }

    pub fn get_method(&self) -> &String {
        &self.method
    }

    pub fn get_route_regex(&self) -> Regex {
        Regex::new(&self.route_regex).expect("invalid regex")
    }
}

/// Properties for an SQS event
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct EventSqsProperties {
    queue: String,
}

impl EventSqsProperties {
    pub fn get_queue(&self) -> &String {
        &self.queue
    }
}

/// Properties for an event - abstracted to allow for different event types
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum EventProperties {
    Api(EventApiProperties),
    Sqs(EventSqsProperties),
}

/// A Lambda function event as specified in the SAM template
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Event {
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<EventProperties>,
}

impl Event {
    pub fn new(event_type: EventType) -> Self {
        match event_type {
            EventType::Api => Self {
                properties: Some(EventProperties::Api(EventApiProperties {
                    base_path: None,
                    path: String::new(),
                    method: String::new(),
                    route_regex: String::new(),
                })),
            },
            EventType::Sqs => Self {
                properties: Some(EventProperties::Sqs(EventSqsProperties {
                    queue: String::new(),
                })),
            },
        }
    }

    pub fn set_api_properties(&mut self, path: String, base_path: Option<String>, method: String) {
        match &mut self.properties {
            Some(EventProperties::Api(api_properties)) => {
                let replaced_path = replaced_regex_path(&path, &base_path);
                let route_regex = Regex::new(&replaced_path).expect("invalid regex");

                api_properties.base_path = base_path;
                api_properties.path = path;
                api_properties.method = method;
                api_properties.route_regex = route_regex.to_string();
            }
            _ => {}
        }
    }

    pub fn get_api_properties(&self) -> Option<&EventApiProperties> {
        match &self.properties {
            Some(EventProperties::Api(api_properties)) => Some(api_properties),
            _ => None,
        }
    }

    pub fn set_sqs_properties(&mut self, queue: String) {
        match &mut self.properties {
            Some(EventProperties::Sqs(sqs_properties)) => {
                sqs_properties.queue = queue;
            }
            _ => {}
        }
    }

    pub fn get_sqs_properties(&self) -> Option<&EventSqsProperties> {
        match &self.properties {
            Some(EventProperties::Sqs(sqs_properties)) => Some(sqs_properties),
            _ => None,
        }
    }

    pub fn get_properties(&self) -> Option<&EventProperties> {
        self.properties.as_ref()
    }
}

fn replaced_regex_path(path: &str, base_path: &Option<String>) -> String {
    // As SAM supports parameters in url with {param} syntax we need to replace them with usable regex
    let replace_matches: Regex = Regex::new("{.*?}").expect("invalid regex");

    let replaced_sam_path =
        replace_matches
            .find_iter(path)
            .fold(path.to_string(), |mut acc, current_match| {
                if let Ok(current_match) = current_match {
                    let current_match_name: &str =
                        &current_match.as_str()[1..current_match.as_str().len() - 1];

                    if current_match_name.ends_with('+') {
                        acc = acc.replace(
                            current_match.as_str(),
                            &format!(
                                r"(?P<{}>.*)",
                                &current_match_name[0..current_match_name.len() - 1]
                            ),
                        );
                    } else {
                        acc = acc.replace(
                            current_match.as_str(),
                            &format!(r"(?P<{}>[^\/]+)", &current_match_name),
                        );
                    }
                };
                acc
            });

    if let Some(base_path) = base_path {
        format!("^/{}{}$", base_path, replaced_sam_path)
    } else {
        format!("^{}$", replaced_sam_path)
    }
}
