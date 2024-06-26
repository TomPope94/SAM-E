use crate::scripts::environment::build::ResourceWithTemplate;
use anyhow::Result;
use sam_e_types::{
    cloudformation::resource::{
        self,
        function::event::{ApiEvent, Event as LambdaEvent, EventType, SqsEvent},
        Function, ResourceType,
    },
    config::lambda::{docker::DockerBuildBuilder, event::Event, Lambda, PackageType},
};
use std::collections::HashMap;
use tracing::{debug, error, trace, warn};

/// Takes a hashmap of the resources within CloudFormation template and returns each of the Lambdas
/// specified in a vector.
pub fn get_lambdas_from_resources(
    resources: &HashMap<String, ResourceWithTemplate>,
) -> Result<Vec<Lambda>> {
    let mut lambdas: Vec<Lambda> = vec![];

    for (resource_name, resource) in resources.iter() {
        trace!("Resource name: {}", resource_name);
        trace!(
            "Resource Type: {:?}",
            resource.get_resources().resource_type
        );
        match &resource.get_resources().resource_type {
            ResourceType::Function => {
                trace!("Found a function!");

                let properties = resource.get_resources().properties.clone();
                let function = parse_function(
                    resource_name,
                    resource.get_template_name(),
                    resources,
                    properties,
                );

                match function {
                    Ok(f) => {
                        lambdas.push(f);
                    }
                    Err(e) => {
                        error!("Error parsing function: {}", e);
                        warn!("Unable to parse function: {}. Skipping", resource_name);
                        continue;
                    }
                }
            }
            _ => {
                trace!("Resource is not a function, skipping...");
                continue;
            }
        }
    }

    Ok(lambdas)
}

fn parse_function(
    function_name: &str,
    template_name: &str,
    resources: &HashMap<String, ResourceWithTemplate>,
    resource_properties: serde_yaml::Value,
) -> Result<Lambda> {
    debug!("Parsing function: {}", function_name);

    let properties = serde_yaml::from_value::<Function>(resource_properties)?;
    debug!("Properties: {:?}", properties);

    let image_uri = if let Some(image) = properties.get_image_uri() {
        if let Some(image) = image.as_str() {
            image
        } else {
            return Err(anyhow::anyhow!(
                "Image URI found but unable to parse: {}.",
                function_name
            ));
        }
    } else {
        return Err(anyhow::anyhow!("Image URI not found for container: {}. Note, only images are supported currently with SAM-E", function_name));
    };

    let events = properties.get_events();
    debug!("Events: {:?}", events);

    let events_vec = parse_events(function_name, resources, &events);

    let env_vars: HashMap<String, String> = if let Some(function_env) = properties.get_environment()
    {
        function_env
            .get_environment_vars()
            .iter()
            .map(|(k, v)| (k.to_string(), v.as_str().unwrap_or_default().to_string()))
            .collect()
    } else {
        HashMap::new()
    };

    let Some(_package_type) = properties.get_package_type() else {
        warn!("Package type not found for container: {}. Note, only images are supported currently with SAM-E", function_name);
        return Err(anyhow::anyhow!("Package type not found for container: {}. Note, only images are supported currently with SAM-E", function_name));
    };

    let lambda = Lambda::new(
        function_name.to_string(),
        image_uri.to_string(),
        env_vars,
        events_vec,
        template_name,
        PackageType::Image,
        None,
    );

    Ok(lambda)
}

/// Map through the raw events from cloud formation and create a new config Event for each one
fn parse_events(
    function_name: &str,
    resources: &HashMap<String, ResourceWithTemplate>,
    events: &HashMap<String, LambdaEvent>,
) -> Vec<Event> {
    let events_vec: Vec<Event> = events
        .iter()
        .filter_map(|(_, event_data)| {
            let event = match event_data.event_type {
                EventType::Api => {
                    let event_data =
                        serde_yaml::from_value::<ApiEvent>(event_data.properties.clone());
                    let event_props = match event_data {
                        Ok(event_props) => event_props,
                        Err(e) => {
                            error!("Error parsing API Gateway event properties: {}", e);
                            warn!(
                                "Unable to parse API Gateway event properties for: {}. Skipping",
                                function_name
                            );
                            return None;
                        }
                    };

                    let base_path = if let Some(api_id) = event_props.get_rest_api_id() {
                        if let Some(api_id) = api_id.as_str() {
                            get_base_path(&api_id, &resources)
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let mut event = Event::new(None);
                    event.set_api_properties(
                        event_props.get_path().as_str().unwrap().to_string(),
                        base_path,
                        event_props.get_method().as_str().unwrap().to_string(),
                    );

                    event
                }
                EventType::Sqs => {
                    let event_data =
                        serde_yaml::from_value::<SqsEvent>(event_data.properties.clone());
                    let event_props = match event_data {
                        Ok(event_props) => event_props,
                        Err(e) => {
                            error!("Error parsing SQS event properties: {}", e);
                            warn!(
                                "Unable to parse SQS event properties for: {}. Skipping",
                                function_name
                            );
                            return None;
                        }
                    };

                    let queue = event_props
                        .get_queue()
                        .as_str()
                        .unwrap_or_default()
                        .to_string();

                    let mut event = Event::new(None);
                    event.set_sqs_properties(queue);

                    event
                }
                _ => {
                    warn!(
                        "Unsupported event type found for: {}. Skipping",
                        function_name
                    );
                    return None;
                }
            };

            Some(event)
        })
        .collect();

    events_vec
}

pub fn select_lambdas(lambdas: Vec<Lambda>) -> Vec<Lambda> {
    let lambdas_select = dialoguer::MultiSelect::new()
        .with_prompt("Select which new lambdas you would like to spin up in your environment:")
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

pub fn add_build_settings(lambdas: Vec<Lambda>) -> Vec<Lambda> {
    debug!("Adding build settings to lambdas...");
    let mut lambdas = lambdas;

    for lambda in lambdas.iter_mut() {
        let package_type = lambda.get_package_type();

        match package_type {
            PackageType::Image => {
                let local_build = dialoguer::Confirm::new()
                    .with_prompt(format!(
                        "Is the lambda ({}) built as part of this project (i.e. not pulled from DockerHub)?",
                        lambda.get_name()
                    ))
                    .default(true)
                    .interact()
                    .unwrap();

                if local_build {
                    let context = dialoguer::Input::<String>::new()
                        .with_prompt(format!(
                            "Enter the Docker build context for container: {}",
                            lambda.get_name()
                        ))
                        .default(".".to_string())
                        .interact()
                        .unwrap();

                    let dockerfile = dialoguer::Input::<String>::new()
                        .with_prompt(format!(
                            "Enter the Dockerfile (path from context) for container: {}",
                            lambda.get_name()
                        ))
                        .default("Dockerfile".to_string())
                        .interact()
                        .unwrap();

                    let use_ssh = dialoguer::Confirm::new()
                        .with_prompt(format!(
                            "Does the Docker build require SSH for container: {}",
                            lambda.get_name()
                        ))
                        .default(false)
                        .interact()
                        .unwrap();

                    lambda.set_docker_build(
                        DockerBuildBuilder::new()
                            .with_context(context)
                            .with_dockerfile(dockerfile)
                            .with_use_ssh(use_ssh)
                            .build(),
                    );
                }
            }
        }
    }

    lambdas
}

/// If a Lambda is linked to an API gateway with a base path, this will be returned as an Option.
fn get_base_path(
    api_id: &str,
    sam_resources: &HashMap<String, ResourceWithTemplate>,
) -> Option<String> {
    let base_api_resource = sam_resources.iter().find(|(resource_name, sub_resource)| {
        match sub_resource.get_resources().resource_type {
            ResourceType::BasePathMapping => {
                let Ok(properties) = serde_yaml::from_value::<resource::BasePathMapping>(
                    sub_resource.get_resources().properties.clone(),
                ) else {
                    warn!(
                        "Unable to parse base path mapping properties for: {}. Skipping",
                        resource_name
                    );
                    return false;
                };
                let rest_api_id = properties.get_rest_api_id();

                if let Some(rest_api_id) = rest_api_id.as_str() {
                    if rest_api_id == api_id {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    });

    // This seems like a reduntant check in match. Probably a better way to return the value from
    // iteration above.
    if let Some((resource_name, resource)) = base_api_resource {
        match &resource.get_resources().resource_type {
            ResourceType::BasePathMapping => {
                let Ok(properties) = serde_yaml::from_value::<resource::BasePathMapping>(
                    resource.get_resources().properties.clone(),
                ) else {
                    warn!(
                        "Unable to parse base path mapping properties for: {}. Skipping",
                        resource_name
                    );
                    return None;
                };
                let base_path = properties.get_base_path();
                if let Some(base_path) = base_path.as_str() {
                    return Some(base_path.to_string());
                }
            }
            _ => {
                return None;
            }
        }
    }

    None
}
