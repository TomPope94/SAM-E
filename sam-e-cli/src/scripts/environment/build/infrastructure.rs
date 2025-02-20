use sam_e_types::{
    cloudformation::resource::{Bucket, DbInstance, EventBus, EventRule, ResourceType},
    config::{
        infrastructure::{
            event_rule::{EventPatternBuilder, EventRuleBuilder},
            triggers::Triggers,
            EventBusBuilder, Infrastructure, MysqlBuilder, PostgresBuilder, QueueBuilder,
            ResourceContainer, S3Builder,
        },
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
                        let postgres_infra = PostgresBuilder::new()
                            .name(resource_name.to_string())
                            .template_name(resource.get_template_name().to_string())
                            .build()?;
                        infrastructure.push(Infrastructure::Postgres(ResourceContainer::new(
                            postgres_infra,
                        )));
                    } else if engine.contains("mysql") {
                        trace!("Database engine recognized as MySQL");
                        let mysql_infra = MysqlBuilder::new()
                            .name(resource_name.to_string())
                            .template_name(resource.get_template_name().to_string())
                            .build()?;
                        infrastructure
                            .push(Infrastructure::Mysql(ResourceContainer::new(mysql_infra)));
                    } else {
                        warn!("Not able to auto infer engine of DB instance: {}. Defaulting to Postgres", resource_name);
                        let postgres_infra = PostgresBuilder::new()
                            .name(resource_name.to_string())
                            .template_name(resource.get_template_name().to_string())
                            .build()?;
                        infrastructure.push(Infrastructure::Postgres(ResourceContainer::new(
                            postgres_infra,
                        )));
                    }
                } else {
                    warn!("No engine type found for DB instance or unable to parse into string: {}. Defaulting to Postgres", resource_name);
                    let postgres_infra = PostgresBuilder::new()
                        .name(resource_name.to_string())
                        .template_name(resource.get_template_name().to_string())
                        .build()?;
                    infrastructure.push(Infrastructure::Postgres(ResourceContainer::new(
                        postgres_infra,
                    )));
                }
            }
            ResourceType::Queue => {
                trace!("Found a queue!");
                let sqs_infra = QueueBuilder::new()
                    .name(resource_name.to_string())
                    .template_name(resource.get_template_name().to_string())
                    .build()?;
                infrastructure.push(Infrastructure::Sqs(ResourceContainer::new(sqs_infra)));
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
            ResourceType::EventBus => {
                debug!("Found an event bus!");
                let Ok(event_bus) =
                    serde_yaml::from_value::<EventBus>(resource.get_resources().properties.clone())
                else {
                    warn!(
                        "Unable to parse event bus properties for: {}. Skipping",
                        resource_name
                    );
                    continue;
                };

                debug!("Properties: {:?}", event_bus);

                let event_bus_name = if let Some(name) = event_bus.name {
                    if let Some(name_str) = name.as_str() {
                        name_str.to_string()
                    } else {
                        warn!(
                            "Unable to parse event bus name (despite existing) for: {}. Defaulting to resource name",
                            resource_name
                        );
                        resource_name.to_string()
                    }
                } else {
                    warn!(
                        "Unable to parse event bus name for: {}. Defaulting to resource name",
                        resource_name
                    );
                    resource_name.to_string()
                };

                let event_bus_infra = EventBusBuilder::new()
                    .name(event_bus_name)
                    .template_name(resource.get_template_name().to_string())
                    .build()?;
                infrastructure.push(Infrastructure::EventBus(ResourceContainer::new(
                    event_bus_infra,
                )));
            }
            ResourceType::EventRule => {
                debug!("Found an event rule!");
                let Ok(event_rule) = serde_yaml::from_value::<EventRule>(
                    resource.get_resources().properties.clone(),
                ) else {
                    warn!(
                        "Unable to parse event rule properties for: {}. Skipping",
                        resource_name
                    );
                    continue;
                };

                debug!("Properties: {:?}", event_rule);

                let targets: Vec<String> = event_rule
                    .clone()
                    .targets
                    .iter()
                    .map(|t| {
                        let arn = t.arn.as_str();
                        if let Some(arn) = arn {
                            arn.to_string()
                        } else {
                            warn!("Unable to parse target ARN for event rule");
                            "".to_string()
                        }
                    })
                    .collect();

                let event_pattern =
                    EventPatternBuilder::from_cloud_formation(event_rule.event_pattern).build()?;

                let event_rule_infra = EventRuleBuilder::new()
                    .name(resource_name.to_string())
                    .template_name(resource.get_template_name().to_string())
                    .triggers(Triggers::new(None, Some(targets)))
                    .event_pattern(event_pattern)
                    .build()?;
                infrastructure.push(Infrastructure::EventRule(ResourceContainer::new(
                    event_rule_infra,
                )));
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

    let s3_infra_builder = S3Builder::new()
        .name(bucket_name.to_string())
        .template_name(template_name.to_string());

    let mut queue_triggers = vec![];
    if let Some(notification_configuration) = resource.get_notification_configuration() {
        if let Some(queue_config) = notification_configuration.get_queue_configurations() {
            for queue in queue_config {
                let queue_val = queue.get_queue().as_str();
                if let Some(queue_name) = queue_val {
                    queue_triggers.push(queue_name.to_string());
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

    let s3_infra = s3_infra_builder
        .triggers(Triggers::new(None, Some(queue_triggers)))
        .build()?;

    Ok(Infrastructure::S3(ResourceContainer::new(s3_infra)))
}

/// Creates the infrastructure files required for the local environment. This includes the
/// Dockerfile and entrypoint.sh for S3 and the custom.conf for SQS. This is done by using Tera to
/// render the templates with the context provided by the config. The files are then written to the
/// .sam-e directory. All files are embedded in the binary so no need to worry about them being lost.
pub fn create_infrastructure_files(config: &Config, dev_deploy: bool) -> anyhow::Result<()> {
    let infrastructure = config.get_infrastructure();

    let mut tera = Tera::default();
    let mut context = Context::new();

    context.insert("lambdas", config.get_lambdas());
    context.insert("infrastructure", config.get_infrastructure());
    context.insert("runtime", config.get_runtime());

    let frontend = config.get_frontend();
    if let Some(frontend) = frontend {
        context.insert("frontend", frontend);
    }

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

    if let Some(docker_dev_template) = Asset::get("docker-compose.dev.yaml") {
        let raw_data = docker_dev_template.data;
        tera.add_raw_template("docker-compose.dev", &String::from_utf8_lossy(&raw_data))?;
    } else {
        error!("Failed to find docker-compose dev template");
        return Err(Error::msg("Failed to find docker-compose dev template"));
    };

    let mut has_s3 = false;
    let mut has_queue = false;

    infrastructure.iter().for_each(|i| match i {
        Infrastructure::S3(_) => has_s3 = true,
        Infrastructure::Sqs(_) => has_queue = true,
        _ => (),
    });

    if has_s3 {
        info!("Detected S3 infrastructure. Creating required files within .sam-e directory");
        fs::create_dir_all(format!("{}/local-s3", SAM_E_DIRECTORY))?;
        create_s3_dockerfile(&tera, &context)?;
    } else {
        debug!("No S3 infrastructure detected. Skipping creation of S3 Dockerfile");
    }

    if has_queue {
        info!("Detected SQS infrastructure. Creating required files within .sam-e directory");
        fs::create_dir_all(format!("{}/local-queue", SAM_E_DIRECTORY))?;
        create_queue_config(&tera, &context)?;
    } else {
        debug!("No SQS infrastructure detected. Skipping creation of queue config");
    }

    create_docker_compose(&tera, &context, dev_deploy)?;
    Ok(())
}

/// Actually writes the docker-compose file to the .sam-e directory after rendering via tera
/// template with the context provided
fn create_docker_compose(tera: &Tera, context: &Context, dev_deploy: bool) -> anyhow::Result<()> {
    let result = if dev_deploy {
        tera.render("docker-compose.dev", &context)?
    } else {
        tera.render("docker-compose", &context)?
    };

    if dev_deploy {
        fs::write(
            format!("{}/dev/docker-compose.yaml", SAM_E_DIRECTORY),
            result,
        )?;
    } else {
        fs::write(
            format!("{}/local/docker-compose.yaml", SAM_E_DIRECTORY),
            result,
        )?;
    }

    Ok(())
}

/// Actually writes the S3 Dockerfile to the .sam-e directory after rendering via tera template with
/// the context provided. Also, writes the entrypoint.sh file to the same directory which is
/// required for S3 triggers to be created in the local environment correctly.
fn create_s3_dockerfile(tera: &Tera, context: &Context) -> anyhow::Result<()> {
    trace!("Context for tera render: {:?}", context);
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
