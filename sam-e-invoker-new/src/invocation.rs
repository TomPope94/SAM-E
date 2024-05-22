pub mod init_error;
pub mod invocation_error;
pub mod next;
pub mod response;
pub mod utils;

use aws_lambda_events::encodings;
use axum::{
    extract::State,
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::{Html, IntoResponse},
    Json,
};
use std::str;
use tracing::{debug, error, info, trace, warn};

use crate::{
    data::api::ApiState,
    api_response::AppError,
};
use sam_e_types::invocation::Invocation;

pub async fn invoke(
    State(api_state): State<ApiState>,
    Json(invocation): Json<Invocation>,
) -> Result<impl IntoResponse, AppError> {
    info!("Invocation requested...");
    trace!("Received invocation: {:#?}", invocation);

    let lambda_name = invocation.get_lambda_name().clone();
    debug!("Lambda name detected as {}", lambda_name);

    let request_id = invocation.get_request_id().clone();
    debug!("Request ID detected as {}", request_id);

    let store = api_state.get_store();

    let _ = utils::write_invocation_to_store(invocation, &store)?;
    let processed_invocation = utils::read_invocation_from_store(&store, &lambda_name, request_id).await?;

    let res_headers = processed_invocation.get_response_headers();
    let res_body = processed_invocation.get_response();

    let mut header_map = HeaderMap::new();
    res_headers.iter().for_each(|(key, value)| {
        header_map.insert(
            HeaderName::try_from(key.as_str())
                .unwrap_or_else(|_| HeaderName::from_static("unknown")),
            HeaderValue::try_from(value.as_str()).unwrap_or_else(|_| {
                HeaderValue::from_static("Unable to convert header to string.")
            }),
        );
    });

    let status_code =
        StatusCode::from_u16(res_body.status_code.try_into().unwrap_or(500))
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    trace!("Returning response with status code: {:?}", status_code);
    info!("Returning response with headers: {:?}", header_map);
    info!("Headers in response: {:?}", &res_body.headers);

    let response = if let Some(response_body) = &res_body.body {
        trace!("Returning response with body: {:?}", response_body);
        match response_body {
            encodings::Body::Text(text) => {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(text) {
                    debug!("Detected JSON response body. Returning JSON response.");
                    Json(parsed).into_response()
                } else {
                    if let Some(content_type) = header_map.get("content-type") {
                        let value_string: &str =
                            str::from_utf8(content_type.as_bytes()).unwrap_or("unknown");
                        if value_string.contains("text/html") {
                            debug!("Detected HTML response body. Returning HTML response.");
                            Html(text.clone()).into_response()
                        } else {
                            debug!("Defaulting to text response");
                            text.clone().into_response()
                        }
                    } else {
                        warn!("No content type found from invocation response. Defaulting to text response");
                        text.clone().into_response()
                    }
                }
            }
            encodings::Body::Binary(binary) => binary.clone().into_response(),
            encodings::Body::Empty => "No Response body found".into_response(),
        }
    } else {
        error!("No response body found. Returning empty response.");
        "No Response body found".into_response()
    };

    Ok(response)
}
