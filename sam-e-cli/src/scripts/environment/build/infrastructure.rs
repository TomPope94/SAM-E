use sam_e_types::{
    cloudformation::resource::{Bucket, DbInstance, ResourceType},
    config::{
        infrastructure::{Infrastructure, InfrastructureType},
        Config,
    },
};

use anyhow::{Error, Result};
use rust_embed::RustEmbed;
use std::{collections::HashMap, fs};
use tera::{Context, Tera};
use tracing::{debug, error, info, trace, warn};

use crate::scripts::environment::build::ResourceWithTemplate;

const SAM_E_DIRECTORY: &str = ".sam-e";

#[derive(RustEmbed)]
#[folder = "assets/templates/"]
struct Asset;

/// Takes a hashmap of resources and returns a vector of Infrastructure. This is done by iterating
/// over the resources and checking the type of each one. If it is a recognized infrastructure type
/// it is added to the vector.
/// TODO: Should have a more generic approach to this, currently only supports RDS, S3, and SQS
pub fn get_infrastructure_from_resources(
    resources: &HashMap<String, ResourceWithTemplate>,
) -> Result<Vec<Infrastructure>> {
    let mut infrastructure = vec![];

    for (resource_name, resource) in resources.iter() {
        trace!("Resource name: {}", resource_name);
        match resource.get_resources().resource_type {
            ResourceType::DbInstance => {
                trace!("Found a DB instance!");
                trace!("Now working out engine type...");

                let Ok(db_props) = serde_yaml::from_value::<DbInstance>(
                    resource.get_resources().properties.clone(),
                ) else {
                    warn!(
                        "Unable to parse DB instance properties for: {}. Skipping",
                        resource_name
                    );
                    continue;
                };
                debug!("Properties: {:?}", db_props);

                if let Some(engine) = db_props.get_engine().as_str() {
                    if engine.contains("postgresql") {
                        trace!("Database engine recognized as Postgres");
                        infrastructure.push(Infrastructure::new(
                            resource_name.to_string(),
                            InfrastructureType::Postgres,
                            resource.get_template_name(),
                        ));
                    } else if engine.contains("mysql") {
                        trace!("Database engine recognized as MySQL");
                        infrastructure.push(Infrastructure::new(
                            resource_name.to_string(),
                            InfrastructureType::Mysql,
                            resource.get_template_name(),
                        ));
                    } else {
                        warn!("Not able to auto infer engine of DB instance: {}. Defaulting to Postgres", resource_name);
                        infrastructure.push(Infrastructure::new(
                            resource_name.to_string(),
                            InfrastructureType::Postgres,
                            resource.get_template_name(),
                        ));
                    }
                } else {
                    warn!("No engine type found for DB instance or unable to parse into string: {}. Defaulting to Postgres", resource_name);
                    infrastructure.push(Infrastructure::new(
                        resource_name.to_string(),
                        InfrastructureType::Postgres,
                        resource.get_template_name(),
                    ));
                }
            }
            ResourceType::Queue => {
                trace!("Found a queue!");
                infrastructure.push(Infrastructure::new(
                    resource_name.to_string(),
                    InfrastructureType::Sqs,
                    resource.get_template_name(),
                ));
            }
            ResourceType::Bucket => {
                trace!("Found a bucket!");

                let Ok(s3_data) =
                    serde_yaml::from_value::<Bucket>(resource.get_resources().properties.clone())
                else {
                    warn!(
                        "Unable to parse S3 properties for: {}. Skipping",
                        resource_name
                    );
                    continue;
                };

                infrastructure.push(create_infrastructure_from_s3_resource(
                    &s3_data,
                    resource_name,
                    resource.get_template_name(),
                )?);
            }
            _ => {
                trace!("Resource not recognized as infrastructure");
            }
        }
    }

    Ok(infrastructure)
}

/// Creates an infrastructure object from the S3 resource. This is done by checking the properties
/// and adding the bucket name to the infrastructure object. If there are any queue configurations
/// they are also added to the triggers of the infrastructure object. Will return an error if the
/// template yaml is not formatted correctly.
fn create_infrastructure_from_s3_resource(
    resource: &Bucket,
    resource_name: &str,
    template_name: &str,
) -> Result<Infrastructure> {
    debug!("Creating infrastructure from S3 resource");
    debug!("Properties: {:?}", resource);
    let bucket_name = if let Some(name) = resource.get_bucket_name().as_str() {
        if name.is_empty() {
            warn!(
                "Unable to parse bucket name for S3 resource: {}. Defaulting to resource name",
                resource_name
            );
            resource_name.to_lowercase() // makes lowercase because s3 buckets are lowercase
        } else {
            name.to_string()
        }
    } else {
        warn!(
            "Unable to find bucket name for S3 resource: {}. Defaulting to resource name",
            resource_name
        );
        resource_name.to_lowercase() // makes lowercase because s3 buckets are lowercase
    };

    let mut new_infrastructure = Infrastructure::new(
        bucket_name.to_string(),
        InfrastructureType::S3,
        template_name,
    );

    if let Some(notification_configuration) = resource.get_notification_configuration() {
        if let Some(queue_config) = notification_configuration.get_queue_configurations() {
            for queue in queue_config {
                let queue_val = queue.get_queue().as_str();
                if let Some(queue_name) = queue_val {
                    new_infrastructure.add_queue_to_triggers(queue_name.to_string());
                } else {
                    warn!(
                        "Unable to parse queue name for S3 bucket: {}. Skipping",
                        bucket_name
                    );
                }
            }
        }
    } else {
        warn!(
            "No notification configurations found for S3 bucket: {}",
            bucket_name
        );
    }

    Ok(new_infrastructure)
}

/// Creates the infrastructure files required for the local environment. This includes the
/// Dockerfile and entrypoint.sh for S3 and the custom.conf for SQS. This is done by using Tera to
/// render the templates with the context provided by the config. The files are then written to the
/// .sam-e directory. All files are embedded in the binary so no need to worry about them being lost.
pub fn create_infrastructure_files(config: &Config) -> anyhow::Result<()> {
    let infrastructure = config.get_infrastructure();

    let mut tera = Tera::default();
    let mut context = Context::new();

    context.insert("lambdas", config.get_lambdas());
    context.insert("infrastructure", config.get_infrastructure());

    if let Some(s3_dockerfile) = Asset::get("local-s3/entrypoint.sh") {
        let raw_data = s3_dockerfile.data;
        tera.add_raw_template("s3-dockerfile", &String::from_utf8_lossy(&raw_data))?;
    } else {
        error!("Failed to find S3 Dockerfile template");
        return Err(Error::msg("Failed to find S3 Dockerfile template"));
    };

    if let Some(queue_config) = Asset::get("local-queue/custom.conf") {
        let raw_data = queue_config.data;
        tera.add_raw_template("queue-config", &String::from_utf8_lossy(&raw_data))?;
    } else {
        error!("Failed to find queue config template");
        return Err(Error::msg("Failed to find queue config template"));
    };

    if let Some(docker_template) = Asset::get("docker-compose.yaml") {
        let raw_data = docker_template.data;
        tera.add_raw_template("docker-compose", &String::from_utf8_lossy(&raw_data))?;
    } else {
        error!("Failed to find docker-compose template");
        return Err(Error::msg("Failed to find docker-compose template"));
    };

    if let true = has_infrastructure_type(infrastructure, InfrastructureType::S3) {
        info!("Detected S3 infrastructure. Creating required files within .sam-e directory");
        fs::create_dir_all(format!("{}/local-s3", SAM_E_DIRECTORY))?;
        create_s3_dockerfile(&tera, &context)?;
    } else {
        debug!("No S3 infrastructure detected. Skipping creation of S3 Dockerfile");
    }

    if let true = has_infrastructure_type(infrastructure, InfrastructureType::Sqs) {
        info!("Detected SQS infrastructure. Creating required files within .sam-e directory");
        fs::create_dir_all(format!("{}/local-queue", SAM_E_DIRECTORY))?;
        create_queue_config(&tera, &context)?;
    } else {
        debug!("No SQS infrastructure detected. Skipping creation of queue config");
    }

    create_docker_compose(&tera, &context)?;
    Ok(())
}

/// Checks if the infrastructure given matches the type being checked. Returns Boolean based on
/// this match
fn has_infrastructure_type(
    infrastructure: &Vec<Infrastructure>,
    infrastructure_type: InfrastructureType,
) -> bool {
    infrastructure
        .iter()
        .any(|i| i.get_infrastructure_type() == &infrastructure_type)
}

/// Actually writes the docker-compose file to the .sam-e directory after rendering via tera
/// template with the context provided
fn create_docker_compose(tera: &Tera, context: &Context) -> anyhow::Result<()> {
    let result = tera.render("docker-compose", &context)?;

    fs::write(format!("{}/docker-compose.yaml", SAM_E_DIRECTORY), result)?;

    Ok(())
}

/// Actually writes the S3 Dockerfile to the .sam-e directory after rendering via tera template with
/// the context provided. Also, writes the entrypoint.sh file to the same directory which is
/// required for S3 triggers to be created in the local environment correctly.
fn create_s3_dockerfile(tera: &Tera, context: &Context) -> anyhow::Result<()> {
    let result = tera.render("s3-dockerfile", &context)?;

    fs::write(
        format!("{}/local-s3/entrypoint.sh", SAM_E_DIRECTORY),
        result,
    )?;

    if let Some(s3_dockerfile) = Asset::get("local-s3/Dockerfile") {
        let raw_data = s3_dockerfile.data;
        fs::write(format!("{}/local-s3/Dockerfile", SAM_E_DIRECTORY), raw_data)?;
    } else {
        error!("Failed to find S3 Dockerfile template");
        return Err(Error::msg("Failed to find S3 Dockerfile template"));
    };

    Ok(())
}

/// Actually writes the queue config file to the .sam-e directory after rendering via Tera
/// template with the context provided. NOTE: this doesn't work with VMs becuase the file is written
/// to the host machine so can't be passed as volume in dockerfile...
fn create_queue_config(tera: &Tera, context: &Context) -> anyhow::Result<()> {
    let result = tera.render("queue-config", &context)?;

    fs::write(
        format!("{}/local-queue/custom.conf", SAM_E_DIRECTORY),
        result,
    )?;

    Ok(())
}
