use anyhow::anyhow;
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::Response,
};
use sam_e_types::invocation::ExtensionEvent;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, info};
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

/// Handle POST /:function_name/2020-01-01/extension/register
///
/// Extensions call this endpoint when they start to register with the Lambda runtime.
/// The invoker assigns a unique ID and creates an event channel for the extension.
pub async fn handle(
    Path(function_name): Path<String>,
    headers: HeaderMap,
    State(api_state): State<ApiState>,
) -> Result<Response, AppError> {
    info!(
        "Extension registration requested for function: {}",
        function_name
    );

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
