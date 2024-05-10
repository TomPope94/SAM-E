use crate::{
    data::ApiState,
    response::AppError,
    utils::{find_lambda_with_base_path, create_api_request},
};
use sam_e_types::invocation::{Invocation, EventRequest};

use aws_lambda_events::event::apigw::ApiGatewayProxyRequest;
use axum::{
    extract::{Json, Path, Query, State},
    http::{HeaderMap, Method},
    debug_handler,
};
use std::collections::HashMap;
use tracing::{debug, warn};

#[debug_handler]
pub async fn handler(
    method: Method,
    headers: HeaderMap,
    path: Option<Path<String>>,
    Query(params): Query<HashMap<String, String>>,
    State(api_state): State<ApiState>,
    body: Option<Json<serde_json::Value>>,
) -> Result<(), AppError> {
    debug!("Request received: {:#?}", method);
    let api_lambdas = api_state.get_api_lambdas();
    if api_lambdas.is_empty() {
        warn!("No API lambdas detected from SAM template.");
        return Ok(())
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
    debug!("Event lambda found: {:?}", &matched_lambda);

    debug!("Creating invocation using matched lambda and request data");
    let mut new_invocation = Invocation::new(EventRequest::Api(ApiGatewayProxyRequest::default()));
    let request_id = new_invocation.get_request_id().clone();
    let api_data = create_api_request(
        body,
        headers,
        params,
        method,
        &prepended_path,
        &matched_api_props.get_base_path(),
        &request_id,
    );
    new_invocation.set_request(EventRequest::Api(api_data));

    debug!("Now adding invocation to store");
    // Send message to the invoker with invocation
    let client = api_state.get_client();
    let response = client.post("http://0.0.0.0:3030/invoke")
        .json(&serde_json::json!(new_invocation))
        .send()
        .await?;

    Ok(())
}

