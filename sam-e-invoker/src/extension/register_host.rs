use anyhow::anyhow;
use axum::{
    extract::{ConnectInfo, State},
    http::HeaderMap,
    response::Response,
};
use bollard::query_parameters::InspectNetworkOptions;
use sam_e_types::invocation::ExtensionEvent;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::{api_response::AppError, data::api::ApiState};

const EXTENSION_NAME_HEADER: &str = "lambda-extension-name";
const EXTENSION_IDENTIFIER_HEADER: &str = "lambda-extension-identifier";

#[derive(Debug, Serialize, Deserialize)]
struct RegisterResponse {
    #[serde(rename = "functionName")]
    function_name: String,
    #[serde(rename = "functionVersion")]
    function_version: String,
    handler: String,
}

/// Handle POST /2020-01-01/extension/register (without function name prefix)
/// Extracts function name from connecting IP address
pub async fn handle(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    State(api_state): State<ApiState>,
) -> Result<Response, AppError> {
    let source_ip = addr.ip().to_string();

    info!(
        "Extension registration from IP: {}, headers: {:?}",
        source_ip, headers
    );

    // At the point an extension is registered, we check all the docker IP addresses and map their
    // container names to the function names in the API state.
    let docker_network = api_state
        .docker_client
        .inspect_network("sam-e-development", None::<InspectNetworkOptions>)
        .await?;
    let containers = if let Some(containers) = docker_network.containers {
        containers
    } else {
        warn!("No containers found in Docker network 'sam-e-development'");
        HashMap::new()
    };

    // Find the function name for this IP
    let mut function_name: Option<String> = None;

    for container in containers.values() {
        if let Some(container_ip) = &container.ipv4_address {
            let ip_only = container_ip.split('/').next().unwrap_or("");
            if ip_only == source_ip {
                // Match found - get container name and map to function name
                if let Some(container_name) = &container.name {
                    debug!("Found container {} with IP {}", container_name, source_ip);

                    // Verify this function exists in our config
                    let lambda_exists = api_state
                        .lambdas
                        .iter()
                        .any(|l| l.get_name() == container_name);

                    if lambda_exists {
                        function_name = Some(container_name.to_string());
                        // Store the IP to function mapping
                        api_state
                            .register_ip_to_function(
                                ip_only.to_string(),
                                container_name.to_string(),
                            )
                            .await;
                        info!("Registered IP {} to function {}", ip_only, container_name);
                        break;
                    } else {
                        warn!("Container {} not found in config", container_name,);
                    }
                }
            }
        }
    }

    let function_name = function_name
        .ok_or_else(|| anyhow!("Could not determine function name from IP {}", source_ip))?;

    // Extract extension name from header
    let extension_name = headers
        .get(EXTENSION_NAME_HEADER)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| anyhow!("Missing required header: {}", EXTENSION_NAME_HEADER))?;

    debug!("Registering extension: {}", extension_name);

    // Generate unique extension ID
    let extension_id = Uuid::new_v4().to_string();

    // Create event channel for this extension
    let (tx, rx) = mpsc::unbounded_channel::<ExtensionEvent>();

    // Store the receiver in the extension store so event.rs can access it
    let extension_store = api_state.get_extension_store();
    let mut ext_store_guard = extension_store.lock().await;
    ext_store_guard.insert(extension_id.clone(), (function_name.clone(), tx, rx));
    drop(ext_store_guard);

    info!(
        "Extension registered successfully: {} (ID: {}) for function: {}",
        extension_name, extension_id, function_name
    );

    // Prepare response
    let response_body = RegisterResponse {
        function_name: function_name.clone(),
        function_version: "$LATEST".to_string(),
        handler: "handler".to_string(), // Mock handler name
    };

    let response = Response::builder()
        .status(200)
        .header(EXTENSION_IDENTIFIER_HEADER, extension_id)
        .header("content-type", "application/json")
        .body(axum::body::Body::from(serde_json::to_string(
            &response_body,
        )?))
        .map_err(|e| anyhow!("Failed to build response: {}", e))?;

    Ok(response)
}
