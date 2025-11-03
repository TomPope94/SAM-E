pub mod init_error;
pub mod invocation_error;
pub mod next;
pub mod next_host;
pub mod response;
pub mod utils;

use aws_lambda_events::encodings;
use axum::{
    body::{Body, Bytes},
    extract::State,
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    Json,
};
use futures::stream::StreamExt;
use sam_e_types::invocation::StreamChunk;
use std::str;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing::{debug, error, info, trace, warn};

use crate::{api_response::AppError, data::api::ApiState, extension};
use sam_e_types::invocation::{Extension, Invocation};

const INVOCATION_TYPE_HEADER: &str = "x-amz-invocation-type";

pub async fn invoke(
    headers: HeaderMap,
    State(api_state): State<ApiState>,
    Json(mut invocation): Json<Invocation>,
) -> Result<Response, AppError> {
    info!("Invocation requested...");
    trace!("Received invocation: {:#?}", invocation);

    let lambda_name = invocation.get_lambda_name().clone();
    debug!("Lambda name detected as {}", lambda_name);

    let request_id = invocation.get_request_id().clone();
    debug!("Request ID detected as {}", request_id);

    // Check if this is a streaming invocation
    let is_streaming = headers
        .get(INVOCATION_TYPE_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("ResponseStream"))
        .unwrap_or(false);

    // Attach registered extensions to this invocation
    let extension_store = api_state.get_extension_store();
    let ext_store_guard = extension_store.lock().await;

    // Find all extensions registered for this Lambda
    let mut extensions_for_lambda = Vec::new();
    for (ext_id, (func_name, sender, _receiver)) in ext_store_guard.iter() {
        if func_name == &lambda_name {
            debug!("Adding extension {} to invocation", ext_id);
            let extension = Extension::new(
                ext_id.clone(),
                "extension".to_string(), // Extension name
                sender.clone(),
            );
            extensions_for_lambda.push(extension);
        }
    }
    drop(ext_store_guard);

    // Add extensions to invocation
    for ext in extensions_for_lambda {
        invocation.add_extension(ext);
    }

    if !invocation.get_extensions().is_empty() {
        info!(
            "Invocation has {} registered extension(s)",
            invocation.get_extensions().len()
        );
    }

    let store = api_state.get_store();

    if is_streaming {
        info!("Detected streaming invocation");

        // Create a channel for streaming chunks
        let (tx, rx) = mpsc::unbounded_channel::<StreamChunk>();

        // Attach the sender to the invocation
        invocation.set_stream_sender(tx);

        // Write the invocation to store
        let _ = utils::write_invocation_to_store(invocation.clone(), &store)?;

        // Dispatch INVOKE event to all registered extensions
        if !invocation.get_extensions().is_empty() {
            debug!("Dispatching INVOKE event to extensions");
            if let Err(e) = extension::dispatch_invoke_event(&invocation).await {
                warn!("Failed to dispatch INVOKE event to extensions: {}", e);
            }
        }

        // Create a streaming response
        let stream = UnboundedReceiverStream::new(rx);
        let body_stream = stream.map(|chunk| match chunk {
            StreamChunk::Data(data) => Ok::<Bytes, std::io::Error>(Bytes::from(data)),
            StreamChunk::Complete => {
                debug!("Stream complete signal received");
                Ok(Bytes::new())
            }
            StreamChunk::Error(err) => {
                error!("Stream error received: {}", err);
                Err(std::io::Error::new(std::io::ErrorKind::Other, err))
            }
        });

        let body = Body::from_stream(body_stream);
        let response = Response::builder()
            .status(StatusCode::OK)
            .header(
                "content-type",
                "application/vnd.awslambda.http-integration-response",
            )
            .header("transfer-encoding", "chunked")
            .body(body)
            .unwrap();

        info!("Returning streaming response");
        return Ok(response);
    }

    // Non-streaming path (existing behavior)
    let _ = utils::write_invocation_to_store(invocation.clone(), &store)?;

    // Dispatch INVOKE event to all registered extensions
    if !invocation.get_extensions().is_empty() {
        debug!("Dispatching INVOKE event to extensions");
        if let Err(e) = extension::dispatch_invoke_event(&invocation).await {
            warn!("Failed to dispatch INVOKE event to extensions: {}", e);
        }
    }

    let processed_invocation =
        utils::read_invocation_from_store(&store, &lambda_name, request_id).await?;

    let res_headers = processed_invocation.get_response_headers();
    let res_body = processed_invocation.get_response();

    let mut header_map = HeaderMap::new();

    // Add headers from response_headers (added by Lambda adapter)
    res_headers.iter().for_each(|(key, value)| {
        header_map.insert(
            HeaderName::try_from(key.as_str())
                .unwrap_or_else(|_| HeaderName::from_static("unknown")),
            HeaderValue::try_from(value.as_str()).unwrap_or_else(|_| {
                HeaderValue::from_static("Unable to convert header to string.")
            }),
        );
    });

    // Add headers from ApiGatewayProxyResponse.headers
    res_body.headers.iter().for_each(|(key, value)| {
        if let Ok(header_name) = HeaderName::try_from(key.as_str()) {
            // value is already a HeaderValue, clone it
            header_map.insert(header_name, value.clone());
        }
    });

    // Add headers from ApiGatewayProxyResponse.multi_value_headers
    res_body.multi_value_headers.iter().for_each(|(key, value)| {
        if let Ok(header_name) = HeaderName::try_from(key.as_str()) {
            // value is already a HeaderValue, clone it
            header_map.insert(header_name, value.clone());
        }
    });

    let status_code = StatusCode::from_u16(res_body.status_code.try_into().unwrap_or(500))
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    trace!("Returning response with status code: {:?}", status_code);
    info!("Returning response with all merged headers: {:?}", header_map);
    debug!("Headers from res_body.headers: {:?}", &res_body.headers);
    debug!("Headers from res_body.multi_value_headers: {:?}", &res_body.multi_value_headers);

    let response = if let Some(response_body) = &res_body.body {
        trace!("Returning response with body: {:?}", response_body);
        match response_body {
            encodings::Body::Text(text) => {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(text) {
                    debug!("Detected JSON response body. Returning JSON response.");
                    trace!("Parsed JSON response: {:#?}", parsed);

                    // Remove content-length header as Axum will recalculate it
                    header_map.remove("content-length");

                    let response = (status_code, header_map, Json(parsed)).into_response();
                    info!("Built JSON response with status {}", status_code);
                    response
                } else {
                    if let Some(content_type) = header_map.get("content-type") {
                        let value_string: &str =
                            str::from_utf8(content_type.as_bytes()).unwrap_or("unknown");
                        if value_string.contains("text/html") {
                            debug!("Detected HTML response body. Returning HTML response.");
                            (status_code, header_map, Html(text.clone())).into_response()
                        } else if value_string.contains("text/event-stream") {
                            debug!("Detected SSE response body. Returning text/event-stream response.");
                            // Remove content-length for SSE streaming
                            header_map.remove("content-length");
                            (status_code, header_map, text.clone()).into_response()
                        } else {
                            debug!("Defaulting to text response with content-type: {}", value_string);
                            (status_code, header_map, text.clone()).into_response()
                        }
                    } else {
                        warn!("No content type found from invocation response. Defaulting to HTML response");
                        (status_code, header_map, Html(text.clone())).into_response()
                    }
                }
            }
            encodings::Body::Binary(binary) => {
                (status_code, header_map, binary.clone()).into_response()
            }
            encodings::Body::Empty => {
                (status_code, header_map, "No Response body found").into_response()
            }
        }
    } else {
        error!("No response body found. Returning empty response.");
        "No Response body found".into_response()
    };

    Ok(response)
}
