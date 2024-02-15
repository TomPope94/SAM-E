use sam_e_types::config::{Event, EventType, Lambda, Infrastructure, InfrastructureType};
use anyhow::Error;
use serde_yaml::Value;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use tracing::{debug, error, trace, warn};

/// Gets the raw CloudFormation template file from directory and returns resources specified as
/// hashmap. If multi set to true, will return all files found collated (i.e. all resources across
/// all the files).
pub fn collect_template_to_resource(
    template_name: &str,
    multi: &bool,
    current_dir: &PathBuf,
) -> anyhow::Result<HashMap<String, Value>> {
    let template_files = find_all_files(&current_dir, template_name)?;
    if template_files.is_empty() {
        return Err(Error::msg("No template files found"));
    }

    let resources = match multi {
        true => {
            if template_files.len() == 1 {
                warn!("The multi flag was set to true but only one template file found. If this is correct we recommend removing the multi flag to avoid unexpected behaviour.")
            }
            let mut resources = HashMap::new();
            for file in template_files {
                let temp_resources = build_template(&file)?;
                temp_resources.iter().for_each(|(k, v)| {
                    resources.insert(k.to_string(), v.to_owned());
                });
            }
            Ok(resources)
        }
        false => {
            if template_files.len() > 1 {
                warn!("Multiple template files found but multi flag set to false (either by default or manually), will use the first one vs collating together");
            }
            build_template(&template_files[0])
        }
    };

    resources
}

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

                let environment_vars_input: HashMap<String, String> = environment_vars
                    .iter()
                    .map(|(k, v)| {
                        let value = dialoguer::Input::<String>::new()
                            .with_prompt(format!("Found an env variable: {} for container: {}. Type to overwrite value", k, resource_name))
                            .default(v.to_string())
                            .interact()
                            .unwrap();
                        (k.to_string(), value)
                    })
                    .collect();

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
                    environment_vars_input,
                    events_vec,
                );
                lambdas.push(lambda);
            }
        }
    }

    lambdas
}

pub fn get_infrastructure_from_resources(resources: &HashMap<String, serde_yaml::Value>) -> Vec<Infrastructure> {
    let mut infrastructure = vec![];

    for (resource_name, resource) in resources.iter() {
        trace!("Resource name: {}", resource_name);
        if let Some(resource_type) = resource.get("Type") {
            if resource_type == "AWS::RDS::DBInstance" {
                trace!("Found a DB instance!");
                trace!("Now working out engine type...");

                if let Some(engine) = resource["Properties"].get("Engine") {
                    if engine.as_str().unwrap().contains("postgresql") {
                        trace!("Database engine recognized as Postgres");
                        infrastructure.push(Infrastructure::new(resource_name.to_string(), InfrastructureType::Postgres));
                    }

                    if engine.as_str().unwrap().contains("mysql") {
                        trace!("Database engine recognized as MySQL");
                        infrastructure.push(Infrastructure::new(resource_name.to_string(), InfrastructureType::Mysql));
                    }
                } else {
                    error!("No engine type found for DB instance: {}", resource_name);
                }
            }

            if resource_type == "AWS::SQS::Queue" {
                trace!("Found a queue!");
                infrastructure.push(Infrastructure::new(resource_name.to_string(), InfrastructureType::Sqs));
            }

            if resource_type == "AWS::S3::Bucket" {
                trace!("Found a bucket!");

                if let Some(bucket_name) = resource["Properties"].get("BucketName") {
                    infrastructure.push(Infrastructure::new(bucket_name.as_str().unwrap().to_string(), InfrastructureType::S3));
                } else {
                    error!("No bucket name provided for S3 bucket: {}", resource_name);
                }
            }
        }
    }

    infrastructure
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
        } else {
            return None;
        }
    } else {
        return None;
    }
}

/// Builds the template for an individual CloudFormation template returning a hashmap of just
/// the resources section. Starts by reading the file to a string before passing to serde_yaml to
/// be parsed into the HashMap. 
fn build_template(template: &PathBuf) -> anyhow::Result<HashMap<String, Value>> {
    debug!("Building template: {:?}", template);

    let template_path = template.to_str().unwrap();
    debug!("Template path: {}", template_path);

    let yaml_file = fs::read_to_string(template_path)?;
    debug!("YAML file read successfully");

    let template_value: Value = serde_yaml::from_str(&yaml_file)?;
    let resources = serde_yaml::from_value(template_value["Resources"].to_owned())?;

    Ok(resources)
}

/// Recursively goes through directories to find all files of a specific name
fn find_all_files(path: &impl AsRef<Path>, filename: &str) -> std::io::Result<Vec<PathBuf>> {
    let mut buf = vec![];

    trace!("Reading entries in {:?}", path.as_ref());
    let entries = fs::read_dir(path)?;

    for entry in entries {
        let entry = entry?;
        let meta = entry.metadata()?;
        trace!("Found entry: {:?}", entry.path());

        if meta.is_dir() {
            trace!("Entry recognized as directory, recursing...");
            let mut subdir = find_all_files(&entry.path(), filename)?;
            buf.append(&mut subdir);
        }

        if meta.is_file() && entry.file_name().to_str().unwrap() == filename {
            trace!("Entry recognized as file, adding to buffer...");
            debug!("Found file: {:?}", entry.path());
            buf.push(entry.path());
        }
    }

    Ok(buf)
}
