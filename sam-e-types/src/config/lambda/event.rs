use fancy_regex::Regex;
use serde::{Deserialize, Serialize};

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
    pub fn new(event_properties: Option<EventProperties>) -> Self {
        Self {
            properties: event_properties,
        }
    }

    pub fn set_api_properties(&mut self, path: String, base_path: Option<String>, method: String) {
        let replaced_path = replaced_regex_path(&path, &base_path);
        let route_regex = Regex::new(&replaced_path).expect("invalid regex").to_string();
        let api_props = EventApiProperties {
            path,
            base_path,
            method,
            route_regex,
        };

        self.properties = Some(EventProperties::Api(api_props));
    }

    pub fn get_api_properties(&self) -> Option<&EventApiProperties> {
        match &self.properties {
            Some(EventProperties::Api(api_properties)) => Some(api_properties),
            _ => None,
        }
    }

    pub fn set_sqs_properties(&mut self, queue: String) {
        let sqs_props = EventSqsProperties { queue };
        self.properties = Some(EventProperties::Sqs(sqs_props));
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

