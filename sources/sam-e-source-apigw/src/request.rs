use crate::{
    data::{ApiState, ContentType},
    response::AppError,
    utils::{create_api_request, find_lambda_with_base_path},
};
use sam_e_types::invocation::{EventRequest, InvocationBuilder};

use axum::{
    extract::{Json, Path, Query, State},
    http::{HeaderMap, Method},
    response::{Html, IntoResponse},
    debug_handler,
};
use std::{str::FromStr, collections::HashMap};
use uuid::Uuid;
use tracing::{debug, trace, warn};

#[debug_handler]
pub async fn handler(
    method: Method,
    headers: HeaderMap,
    path: Option<Path<String>>,
    Query(params): Query<HashMap<String, String>>,
    State(api_state): State<ApiState>,
    body: Option<Json<serde_json::Value>>,
) -> Result<impl IntoResponse, AppError> {
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

    let (matched_lambda, matched_event) = find_lambda_with_base_path(api_lambdas, &prepended_path, &method.to_string())?;
    let matched_api_props = matched_event.get_api_properties().unwrap().to_owned();
    trace!("Event lambda found: {:?}", &matched_lambda);

    debug!("Creating invocation using matched lambda and request data");
    let request_id = Uuid::new_v4();
    let api_data = create_api_request(
        body,
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
    let response = client.post("http://0.0.0.0:3030/invoke")
        .json(&serde_json::json!(new_invocation))
        .send()
        .await?;

    debug!("Response from invoker");
    trace!("Response generated: {:#?}", response);

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

    match response_type {
        ContentType::Json => {
            debug!("Parsing response data as JSON");
            let response_data = response.json::<serde_json::Value>().await?;
            debug!("Response data parsed successfully. Now returning...");
            trace!("Response data: {:#?}", response_data);
            Ok(Json(response_data).into_response())
        }
        ContentType::Text => {
            debug!("Parsing response data as text");
            let response_data = response.text().await?;
            debug!("Response data parsed successfully. Now returning...");
            trace!("Response data: {:#?}", response_data);
            Ok(response_data.into_response())
        }
        ContentType::Html => {
            debug!("Parsing response data as HTML");
            let response_data = response.text().await?;
            debug!("Response data parsed successfully. Now returning...");
            trace!("Response data: {:#?}", response_data);
            Ok(Html(response_data).into_response())
        }
    }
}

