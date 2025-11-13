use crate::data::store::InvocationQueue;
use sam_e_types::invocation::{EventRequest, Status, StreamChunk};

use aws_lambda_events::apigw::ApiGatewayProxyResponse;
use axum::{
    body::Bytes,
    debug_handler,
    extract::{ConnectInfo, Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use std::{collections::HashMap, str};
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

use crate::data::api::ApiState;

const STREAM_COMPLETE_HEADER: &str = "x-lambda-stream-complete";

#[debug_handler]
pub async fn response_handler(
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    Path(request_id): Path<Uuid>,
    State(api_state): State<ApiState>,
    body: Bytes,
) -> impl IntoResponse {
    let source_ip = addr.ip().to_string();

    debug!(
        "Received /response from IP: {}, headers: {:?}",
        source_ip, headers
    );

    // Look up function name from IP address
    let container_name = match api_state.get_function_name_from_ip(&source_ip).await {
        Ok(function_name) => {
            info!(
                "Mapped IP {} to function name: {}",
                source_ip, function_name
            );
            function_name
        }
        Err(_) => {
            // Fallback: try X-Lambda-Function-Name header
            if let Some(func_name) = headers
                .get("x-lambda-function-name")
                .and_then(|h| h.to_str().ok())
            {
                warn!(
                    "IP {} not registered, using X-Lambda-Function-Name header: {}",
                    source_ip, func_name
                );
                func_name.to_string()
            } else {
                warn!(
                    "Could not determine function name from IP {} or headers - extension may not be registered yet",
                    source_ip
                );
                "unknown".to_string()
            }
        }
    };

    info!("Processing response for function: {}", container_name);
    let store = api_state.get_store();

    let mut headers_hashmap: HashMap<String, String> = headers
        .iter()
        .map(|(key, value)| {
            let value_string: &str = str::from_utf8(value.as_bytes()).unwrap_or("unknown");
            (key.to_string(), value_string.to_owned())
        })
        .collect();

    // Write the response record
    let write_store = store.clone();
    let write_queue = InvocationQueue::new();
    let write_container_name = container_name.clone();

    let mut store_queues = write_store.queues.write();

    match store_queues
        .entry(write_container_name)
        .or_insert(write_queue)
        .get_invocations_mut()
        .iter_mut()
        .find(|invocation| invocation.get_request_id() == &request_id)
    {
        Some(invocation) => {
            debug!("Found the invocation to complete processing");
            debug!("Raw lambda response headers: {:?}", headers_hashmap);
            trace!("Raw lambda response body: {:?}", body);

            // Check if this is a streaming invocation
            if invocation.get_stream_sender().is_some() {
                debug!("Detected streaming invocation");

                // Check if this is the final chunk
                let is_complete = headers
                    .get(STREAM_COMPLETE_HEADER)
                    .and_then(|v| v.to_str().ok())
                    .map(|v| v.eq_ignore_ascii_case("true"))
                    .unwrap_or(false);

                // Auto-complete if this is the first response from a non-streaming Lambda
                // (Lambda doesn't support streaming protocol, but client requested streaming mode)
                let current_status = invocation.get_status();
                let is_first_response =
                    matches!(current_status, Status::Pending | Status::Processing);
                let should_auto_complete = !is_complete && is_first_response;

                if is_complete || should_auto_complete {
                    if should_auto_complete {
                        info!(
                            "Auto-completing streaming invocation (non-streaming Lambda detected)"
                        );
                    } else {
                        info!("Received final chunk for streaming invocation");
                    }

                    // Send the final data chunk if body is not empty
                    if !body.is_empty() {
                        if let Err(e) = invocation.send_chunk(StreamChunk::Data(body.to_vec())) {
                            error!("Failed to send final data chunk: {}", e);
                            if let Err(e) = invocation.send_chunk(StreamChunk::Error(format!(
                                "Failed to send final chunk: {}",
                                e
                            ))) {
                                warn!("Failed to send error chunk: {}", e);
                            }
                            return StatusCode::INTERNAL_SERVER_ERROR;
                        }
                    }

                    // Send completion signal
                    if let Err(e) = invocation.send_chunk(StreamChunk::Complete) {
                        error!("Failed to send completion chunk: {}", e);
                        return StatusCode::INTERNAL_SERVER_ERROR;
                    }

                    invocation.set_status(Status::Processed);
                    invocation.set_response_headers(headers_hashmap);

                    debug!("Streaming invocation completed successfully");
                } else {
                    debug!("Received intermediate chunk for streaming invocation");

                    // Send the data chunk
                    if let Err(e) = invocation.send_chunk(StreamChunk::Data(body.to_vec())) {
                        error!("Failed to send data chunk: {}", e);
                        if let Err(e) = invocation
                            .send_chunk(StreamChunk::Error(format!("Failed to send chunk: {}", e)))
                        {
                            warn!("Failed to send error chunk: {}", e);
                        }
                        return StatusCode::INTERNAL_SERVER_ERROR;
                    }

                    // Keep status as Streaming
                    invocation.set_status(Status::Streaming);
                }

                return StatusCode::OK;
            }

            // Non-streaming path (existing behavior)
            invocation.set_status(Status::Processed);

            let request = invocation.get_request();
            match request {
                EventRequest::Api(_) => {
                    debug!("Detected event source as API");
                    let response_data: ApiGatewayProxyResponse =
                        serde_json::from_slice(&body).unwrap();

                    for (key, value) in response_data.headers.iter() {
                        let value_string: &str =
                            str::from_utf8(value.as_bytes()).unwrap_or("unknown");
                        headers_hashmap.insert(key.to_string(), value_string.to_owned());
                    }

                    invocation.set_response(response_data); // Note: response data is currently APIGW
                    invocation.set_response_headers(headers_hashmap);
                }
                EventRequest::Sqs(_) => {
                    debug!("Detected event source as SQS");
                    invocation.set_response_headers(headers_hashmap);
                    // TODO: delete the message on success: currently happens on invocation
                }
            }

            trace!("New invocation... {:?}", invocation);

            return StatusCode::OK;
        }
        None => {
            error!("No invocation found to complete processing");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }
}
