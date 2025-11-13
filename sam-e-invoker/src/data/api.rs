use crate::data::store::Store;

use anyhow::{anyhow, Result};
use bollard::{query_parameters::InspectNetworkOptions, Docker};
use sam_e_types::config::{infrastructure::Infrastructure, lambda::Lambda, Config};
use sam_e_types::invocation::ExtensionEvent;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, info, trace, warn};

// Store extension metadata: (function_name, sender, receiver)
pub type ExtensionStore = Arc<
    Mutex<
        HashMap<
            String,
            (
                String,                                  // function_name
                mpsc::UnboundedSender<ExtensionEvent>,   // sender
                mpsc::UnboundedReceiver<ExtensionEvent>, // receiver
            ),
        >,
    >,
>;

// Store IP address to function name mapping
pub type IpToFunctionMap = Arc<Mutex<HashMap<String, String>>>;

#[derive(Debug, Clone)]
pub struct ApiState {
    pub invocation_store: Store,
    pub lambdas: Vec<Lambda>,
    pub infrastructure: Vec<Infrastructure>,
    pub extension_store: ExtensionStore,
    pub docker_client: Docker,
    pub ip_to_function: IpToFunctionMap,
}

impl ApiState {
    pub async fn new(config: &Config) -> Self {
        debug!("Creating new API state");

        let docker_client = Docker::connect_with_local_defaults().unwrap_or_else(|e| {
            panic!(
                "Failed to create Docker client: {}. \
                Make sure Docker is running and the Docker socket is mounted \
                (add '-v /var/run/docker.sock:/var/run/docker.sock' to docker run, \
                or add it as a volume in docker-compose.yml)",
                e
            )
        });

        let lambdas = config.get_lambdas();
        trace!("Lambdas: {:?}", lambdas);

        let infrastructure = config.get_infrastructure();
        trace!("Infrastructure: {:?}", infrastructure);

        Self {
            invocation_store: Store::new(lambdas).await,
            lambdas: lambdas.to_owned(),
            infrastructure: infrastructure.to_owned(),
            extension_store: Arc::new(Mutex::new(HashMap::new())),
            docker_client,
            ip_to_function: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_api_lambdas(&self) -> Vec<&Lambda> {
        self.lambdas
            .iter()
            .filter(|l| {
                l.get_events()
                    .into_iter()
                    .any(|e| e.get_api_properties().is_some())
            })
            .collect()
    }

    pub fn get_store(&self) -> &Store {
        &self.invocation_store
    }

    pub fn get_infrastructure(&self) -> &Vec<Infrastructure> {
        &self.infrastructure
    }

    pub fn get_extension_store(&self) -> &ExtensionStore {
        &self.extension_store
    }

    pub fn get_ip_to_function(&self) -> &IpToFunctionMap {
        &self.ip_to_function
    }

    pub async fn register_ip_to_function(&self, source_ip: String) -> Result<String> {
        debug!("Attempting to register IP {} to function", source_ip);
        let docker_client = bollard::Docker::connect_with_local_defaults()?;
        let docker_network = docker_client
            .inspect_network("sam-e-development", None::<InspectNetworkOptions>)
            .await?;

        let containers = if let Some(containers) = docker_network.containers {
            containers
        } else {
            warn!("No containers found in Docker network 'sam-e-development'");
            HashMap::new()
        };
        debug!("Found {} containers in Docker network", containers.len());

        // Find the function name for this IP
        let mut function_name: Option<String> = None;
        for container in containers.values() {
            if let Some(container_ip) = &container.ipv4_address {
                let ip_only = container_ip.split('/').next().unwrap_or("");
                trace!(
                    "Checking container {:?} with IP {}",
                    container.name,
                    ip_only
                );
                if ip_only == source_ip {
                    trace!("IP {} matches container {:?}", source_ip, container.name);
                    // Match found - get container name and map to function name
                    if let Some(container_name) = &container.name {
                        debug!("Found container {} with IP {}", container_name, source_ip);

                        // Verify this function exists in our config
                        let lambda_exists =
                            self.lambdas.iter().any(|l| l.get_name() == container_name);

                        if lambda_exists {
                            trace!("Container {} found in config", container_name);
                            function_name = Some(container_name.to_string());

                            // Store the IP to function mapping
                            let mut map = self.ip_to_function.lock().await;
                            debug!(
                                "Registering IP {} to function {}",
                                source_ip,
                                container_name.to_string()
                            );
                            map.insert(source_ip.clone(), container_name.to_string());
                            info!("Registered IP {} to function {}", ip_only, container_name);
                            break;
                        } else {
                            warn!("Container {} not found in config", container_name,);
                        }
                    }
                }
            }
        }
        if let Some(name) = function_name {
            Ok(name)
        } else {
            warn!("No matching function found for IP {}", source_ip);
            Err(anyhow!("No matching function found for IP {}", source_ip))
        }
    }

    pub async fn get_function_name_from_ip(&self, source_ip: &str) -> Result<String> {
        let map = self.ip_to_function.lock().await;
        let container_name = map.get(source_ip).cloned();
        if let Some(name) = &container_name {
            debug!("Found function name {} for IP {}", name, source_ip);
            Ok(name.clone())
        } else {
            drop(map);
            debug!("No function name found for IP {}", source_ip);
            self.register_ip_to_function(source_ip.to_string()).await
        }
    }
}
