use anyhow::anyhow;
use axum::{
    extract::{ConnectInfo, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
    Json,
};
use sam_e_types::invocation::ExtensionEvent;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing::{debug, info, warn};

use crate::{api_response::AppError, data::api::ApiState};

const EXTENSION_IDENTIFIER_HEADER: &str = "lambda-extension-identifier";

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "eventType")]
enum EventResponse {
    #[serde(rename = "INVOKE")]
    Invoke {
        #[serde(rename = "deadlineMs")]
        deadline_ms: u64,
        #[serde(rename = "requestId")]
        request_id: String,
        #[serde(rename = "invokedFunctionArn")]
        invoked_function_arn: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        tracing: Option<TracingInfo>,
    },
    #[serde(rename = "SHUTDOWN")]
    Shutdown {
        #[serde(rename = "shutdownReason")]
        shutdown_reason: String,
        #[serde(rename = "deadlineMs")]
        deadline_ms: u64,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct TracingInfo {
    #[serde(rename = "type")]
    tracing_type: String,
    value: String,
}

/// Handle GET /2020-01-01/extension/event/next (without function name prefix)
/// Function name extracted from extension identifier (which was mapped during registration)
pub async fn handle(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    State(api_state): State<ApiState>,
) -> Result<Response, AppError> {
    let source_ip = addr.ip().to_string();

    debug!("Extension event poll from IP: {}, headers: {:?}", source_ip, headers);

    // Get function name from IP (registered during extension registration)
    let _function_name = api_state
        .get_function_name_from_ip(&source_ip)
        .await
        .unwrap_or_else(|| {
            warn!("Extension event poll from unregistered IP: {}", source_ip);
            "unknown".to_string()
        });

    // Extract extension identifier from header
    let extension_id = headers
        .get(EXTENSION_IDENTIFIER_HEADER)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| anyhow!("Missing required header: {}", EXTENSION_IDENTIFIER_HEADER))?;

    debug!("Extension {} polling for events", extension_id);

    // Get the event receiver for this extension
    let extension_store = api_state.get_extension_store();
    let mut ext_store_guard = extension_store.lock().await;

    let (_function_name, _sender, receiver) =
        ext_store_guard
            .get_mut(extension_id)
            .ok_or_else(|| anyhow!("Extension not found: {}", extension_id))?;

    // Long-poll: wait for an event on the channel
    // This will block until an event is sent or the channel is closed
    debug!("Extension {} waiting for event...", extension_id);

    let event = receiver.recv().await.ok_or_else(|| {
        warn!("Extension {} channel closed", extension_id);
        anyhow!("Extension event channel closed")
    })?;

    drop(ext_store_guard); // Release lock

    info!("Sending event to extension {}: {:?}", extension_id, event);

    // Convert internal event to API response format
    let response_event = match event {
        ExtensionEvent::Invoke {
            request_id,
            deadline_ms,
            invoked_function_arn,
        } => EventResponse::Invoke {
            deadline_ms,
            request_id,
            invoked_function_arn,
            tracing: None, // Could add tracing support later
        },
        ExtensionEvent::Shutdown {
            reason,
            deadline_ms,
        } => EventResponse::Shutdown {
            shutdown_reason: reason,
            deadline_ms,
        },
    };

    // Return the event as JSON
    Ok(Json(response_event).into_response())
}
