use crate::data::store::Store;

use bollard::Docker;
use sam_e_types::config::{infrastructure::Infrastructure, lambda::Lambda, Config};
use sam_e_types::invocation::ExtensionEvent;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, trace};

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

    pub async fn register_ip_to_function(&self, ip: String, function_name: String) {
        let mut map = self.ip_to_function.lock().await;
        debug!("Registering IP {} to function {}", ip, function_name);
        map.insert(ip, function_name);
    }

    pub async fn get_function_name_from_ip(&self, ip: &str) -> Option<String> {
        let map = self.ip_to_function.lock().await;
        map.get(ip).cloned()
    }
}
