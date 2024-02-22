use sam_e_types::config::lambda::{Event, EventType, Lambda};
use serde_yaml::Value;
use std::collections::HashMap;
use tracing::{debug, trace, warn};

/// Takes a hashmap of the resources within CloudFormation template and returns each of the Lambdas
/// specified in a vector.
pub fn get_lambdas_from_resources(resources: &HashMap<String, serde_yaml::Value>) -> Vec<Lambda> {
    let mut lambdas = vec![];

    for (resource_name, resource) in resources.iter() {
        trace!("Resource name: {}", resource_name);
        if let Some(resource_type) = resource.get("Type") {
            if resource_type == "AWS::Serverless::Function" {
                trace!("Found a function!");

                let image_uri = resource["Properties"]["ImageUri"]
                    .as_str()
                    .unwrap_or_else(|| "CHANGE ME")
                    .to_string();

                let environment_vars: HashMap<String, String> = serde_yaml::from_value(
                    resource["Properties"]["Environment"]["Variables"].to_owned(),
                )
                .unwrap_or(HashMap::new());

                let events: HashMap<String, Value> =
                    serde_yaml::from_value(resource["Properties"]["Events"].to_owned())
                        .unwrap_or(HashMap::new());
                debug!("Events: {:?}", events);

                let events_vec: Vec<Event> = events
                    .iter()
                    .map(|(_, event_data)| {
                        if event_data["Type"].as_str().unwrap_or("") == "Api" {
                            let base_path =
                                if let Some(api_id) = event_data["Properties"]["RestApiId"].as_str() {
                                    get_base_path(api_id, &resources)
                                } else {
                                    None
                                };

                            let path = event_data["Properties"]["Path"].as_str();
                            if path.is_none() {
                                warn!("Path not found for event despite type being API, skipping...");
                                warn!("Container: {}", resource_name);
                                return Event::new(EventType::Api);
                            }

                            let method =
                                if let Some(method) = event_data["Properties"]["Method"].as_str() {
                                    method.to_string()
                                } else {
                                    warn!(
                                        "Method was not parsed correctly for container: {}",
                                        resource_name
                                    );
                                    warn!("Defaulting to ANY");
                                    "ANY".to_string()
                                };

                            let mut event = Event::new(EventType::Api);
                            event.set_api_properties(path.unwrap().to_string(), base_path, method);

                            return event;
                        }

                        if event_data["Type"].as_str().unwrap_or("") == "SQS" {
                            let queue = event_data["Properties"]["Queue"].as_str();
                            if queue.is_none() {
                                warn!("Queue not found for event despite type being SQS, skipping...");
                                warn!("Container: {}", resource_name);
                                return Event::new(EventType::Sqs);
                            }

                            if let Some(queue) = queue {
                                let queue_cleaned = queue.replace("!GetaAtt ", "").replace(".Arn", "");

                                let mut event = Event::new(EventType::Sqs);
                                event.set_sqs_properties(queue_cleaned.to_string());

                                return event;
                            }
                        }

                        warn!("Event type not recognized, setting as default API but will not be usable...");
                        warn!("Container: {}", resource_name);
                        Event::new(EventType::Api)
                    })
                    .collect();

                let lambda = Lambda::new(
                    resource_name.to_string(),
                    image_uri,
                    environment_vars,
                    events_vec,
                );
                lambdas.push(lambda);
            }
        }
    }

    lambdas
}

// let environment_vars_input: HashMap<String, String> = environment_vars
//     .iter()
//     .map(|(k, v)| {
//         let value = dialoguer::Input::<String>::new()
//             .with_prompt(format!("Found an env variable: {} for container: {}. Type to overwrite value", k, resource_name))
//             .default(v.to_string())
//             .interact()
//             .unwrap();
//         (k.to_string(), value)
//     })
//     .collect();

pub fn select_lambdas(lambdas: Vec<Lambda>) -> Vec<Lambda> {
    let lambdas_select = dialoguer::MultiSelect::new()
        .with_prompt("Select which lambdas you would like to spin up in your environment:")
        .items_checked(
            &lambdas
                .iter()
                .map(|l| (l.get_name(), true))
                .collect::<Vec<(&str, bool)>>(),
        )
        .interact()
        .unwrap();

    lambdas_select
        .iter()
        .map(|i| lambdas[*i].clone())
        .collect::<Vec<_>>()
}

pub fn specify_environment_vars(lambdas: Vec<Lambda>) -> Vec<Lambda> {
    let mut lambdas = lambdas;

    for lambda in lambdas.iter_mut() {
        let environment_vars = lambda.get_environment_vars();
        let environment_vars_input: HashMap<String, String> = environment_vars
            .iter()
            .map(|(k, v)| {
                let value = dialoguer::Input::<String>::new()
                    .with_prompt(format!(
                        "Found an env variable: {} for container: {}. Type to overwrite value",
                        k,
                        lambda.get_name()
                    ))
                    .default(v.to_string())
                    .interact()
                    .unwrap();
                (k.to_string(), value)
            })
            .collect();

        lambda.set_environment_vars(environment_vars_input);
    }

    lambdas
}

/// If a Lambda is linked to an API gateway with a base path, this will be returned as an Option.
fn get_base_path(
    api_id: &str,
    sam_resources: &HashMap<String, serde_yaml::Value>,
) -> Option<String> {
    let base_path_mapping = sam_resources.iter().find(|(_, sub_resource)| {
        sub_resource["Type"] == "AWS::ApiGateway::BasePathMapping"
            && sub_resource["Properties"]["RestApiId"].is_string()
            && sub_resource["Properties"]["RestApiId"]
                .as_str()
                .unwrap_or("")
                == api_id
    });

    if let Some((_, base_path_mapping)) = base_path_mapping {
        let base_path = base_path_mapping["Properties"]["BasePath"].as_str();
        if let Some(base_path) = base_path {
            return Some(base_path.to_string());
        }
    }

    None
}
