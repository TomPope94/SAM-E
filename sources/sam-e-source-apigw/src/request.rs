use crate::{
    data::{ApiState, ContentType},
    response::AppError,
    utils::{create_api_request, find_lambda_with_base_path},
};
use sam_e_types::invocation::{EventRequest, InvocationBuilder};

use axum::{
    body::Body,
    extract::{Json, Path, Query, State},
    http::{HeaderMap, Method, StatusCode},
    response::{Html, IntoResponse, Response},
};
use futures::StreamExt;
use std::{collections::HashMap, str::FromStr};
use tracing::{debug, info, trace, warn};
use uuid::Uuid;

const INVOCATION_TYPE_HEADER: &str = "x-amz-invocation-type";

pub async fn handler(
    method: Method,
    headers: HeaderMap,
    path: Option<Path<String>>,
    Query(params): Query<HashMap<String, String>>,
    State(api_state): State<ApiState>,
    body: String,
) -> Result<Response, AppError> {
    debug!("Request received: {:#?}", method);
    let api_lambdas = api_state.get_api_lambdas();
    if api_lambdas.is_empty() {
        warn!("No API lambdas detected from SAM template.");
        return Ok("No API lambdas detected from SAM template.".into_response());
    }

    // This is required because Axum doesn't prepend the path with a slash - needed for AWS events
    // and matching to relevant lambda.
    let prepended_path = if let Some(path) = path {
        format!("/{}", path.0)
    } else {
        warn!("No base path detected in request... defaulting to '/'");
        "/".to_string()
    };

    let (matched_lambda, matched_event) =
        find_lambda_with_base_path(api_lambdas, &prepended_path, &method.to_string())?;
    let matched_api_props = matched_event.get_api_properties().unwrap().to_owned();
    trace!("Event lambda found: {:?}", &matched_lambda);

    // Check if streaming is requested (before headers are moved)
    let is_streaming = headers
        .get(INVOCATION_TYPE_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("ResponseStream"))
        .unwrap_or(false);

    debug!("Creating invocation using matched lambda and request data");
    let request_id = Uuid::new_v4();
    let body_option = if body.is_empty() { None } else { Some(body) };
    let api_data = create_api_request(
        body_option,
        headers,
        params,
        method,
        &prepended_path,
        &matched_api_props.get_base_path(),
        &request_id,
    );

    let new_invocation = InvocationBuilder::new()
        .with_request(EventRequest::Api(api_data))
        .with_request_id(request_id)
        .with_lambda_name(matched_lambda.get_name().to_string())
        .build()?;

    debug!("Now adding invocation to store");
    let client = api_state.get_client();

    // Build the request to the invoker
    let mut request_builder = client
        .post("http://0.0.0.0:3030/invoke")
        .json(&serde_json::json!(new_invocation));

    // Add streaming header if requested
    if is_streaming {
        info!("Streaming invocation requested");
        request_builder = request_builder.header(INVOCATION_TYPE_HEADER, "ResponseStream");
    }

    let response = request_builder.send().await?;

    debug!("Response from invoker - status: {}, headers: {:?}", response.status(), response.headers());
    // Note: Response body is not shown here because it's a stream that hasn't been consumed yet

    // Check if this is a streaming response
    let is_streaming_response = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("vnd.awslambda"))
        .unwrap_or(false);

    if is_streaming_response {
        info!("Detected streaming response from invoker");

        // Stream the response back to the client
        let status_code_reqwest = response.status().as_u16();
        let status_code =
            StatusCode::from_u16(status_code_reqwest).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        // Convert reqwest response into a stream of bytes
        let byte_stream = response.bytes_stream();
        let body_stream = byte_stream.map(|result| {
            result
                .map(|bytes| bytes)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        });

        let body = Body::from_stream(body_stream);
        let response = Response::builder()
            .status(status_code)
            .header("transfer-encoding", "chunked")
            .body(body)
            .unwrap();

        info!("Returning streaming response to client");
        return Ok(response);
    }

    // Non-streaming response handling (existing behavior)
    let Some(response_data_type) = response.headers().get("content-type") else {
        warn!("No content type found in response");
        return Ok("No content type found in response".into_response());
    };

    let Ok(response_data_type_str) = response_data_type.to_str() else {
        warn!("Failed to parse content type from response");
        return Ok("Failed to parse content type from response".into_response());
    };

    let Ok(response_type) = ContentType::from_str(response_data_type_str) else {
        warn!("Failed to parse content type string into enum");
        return Ok("Failed to parse content type string into enum".into_response());
    };

    let status_code_reqwest = response.status().as_u16();
    let status_code =
        StatusCode::from_u16(status_code_reqwest).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    match response_type {
        ContentType::Json => {
            debug!("Parsing response data as JSON");
            let response_data = response.json::<serde_json::Value>().await?;
            info!("Response JSON body received and parsed: {}", response_data);
            trace!("Full response data: {:#?}", response_data);
            Ok((status_code, Json(response_data)).into_response())
        }
        ContentType::Text => {
            debug!("Parsing response data as text");
            let response_data = response.text().await?;
            debug!("Response data parsed successfully. Now returning...");
            trace!("Response data: {:#?}", response_data);
            Ok((status_code, response_data).into_response())
        }
        ContentType::Html => {
            debug!("Parsing response data as HTML");
            let response_data = response.text().await?;
            debug!("Response data parsed successfully. Now returning...");
            trace!("Response data: {:#?}", response_data);
            Ok((status_code, Html(response_data)).into_response())
        }
        ContentType::EventStream => {
            debug!("Parsing response data as text/event-stream (SSE)");
            let response_data = response.text().await?;
            info!("SSE response received: {} bytes", response_data.len());
            trace!("SSE response data: {:#?}", response_data);
            // Return as plain text with the text/event-stream content-type preserved
            use axum::http::header;
            Ok((
                status_code,
                [(header::CONTENT_TYPE, "text/event-stream")],
                response_data,
            )
                .into_response())
        }
    }
}
