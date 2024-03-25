use std::collections::HashMap;

use crate::data::store::{Invocation, InvocationQueue, Status, Store};
use sam_e_types::config::lambda::{Event, EventProperties, Lambda};

use anyhow::Result;
use aws_lambda_events::apigw::{
    ApiGatewayProxyRequest, ApiGatewayProxyRequestContext, ApiGatewayRequestIdentity,
};
use axum::{
    extract::Json,
    http::{HeaderMap, HeaderName, HeaderValue, Method},
};
use tokio::time::{sleep, Duration};
use tracing::{debug, info, trace, warn};
use uuid::Uuid;

pub fn write_invocation_to_store(
    invocation: Invocation,
    container_name: &str,
    store: &Store,
) -> Result<(), String> {
    let write_queue = InvocationQueue::new();

    store
        .queues
        .write()
        .entry(container_name.to_owned())
        .or_insert(write_queue)
        .get_invocations_mut()
        .push(invocation.to_owned());

    let _ = invocation.get_request_id().to_owned();
    debug!("Invocation written to store: {:?}", invocation);
    Ok(())
}

pub async fn read_invocation_from_store(
    store: &Store,
    container_name: &str,
    new_invocation_uuid: Uuid,
) -> Invocation {
    let read_store = store.clone();
    let read_queue = InvocationQueue::new();
    let read_container_name = container_name.to_owned();

    let _invocation = tokio::task::spawn(async move {
        loop {
            // Only check every 0.1 seconds to avoid lock contention
            sleep(Duration::from_millis(100)).await;

            let results = read_store.queues.read();
            let result = results.get(&read_container_name);

            if result
                .unwrap_or(&read_queue)
                .get_invocations()
                .clone()
                .into_iter()
                .any(|invocation| {
                    invocation.get_status() == &Status::Processed
                        && invocation.get_request_id() == &new_invocation_uuid
                })
            {
                info!(
                    "Found a processed invocation for container: {}",
                    read_container_name
                );
                break;
            }
        }
    })
    .await;

    info!("Invocation processed");

    // Get the processed record
    let read_store = store.clone();
    let read_queue = InvocationQueue::new();

    let results = read_store.queues.read();
    let result = results.get(container_name);

    let processed_invocation = result
        .unwrap_or(&read_queue)
        .get_invocations()
        .clone()
        .into_iter()
        .find(|invocation| {
            invocation.get_request_id() == &new_invocation_uuid
                && invocation.get_status() == &Status::Processed
        });

    trace!("Processed invocation: {:?}", processed_invocation);

    processed_invocation.unwrap()
}

fn remove_base_path(path: &str, base_path: &Option<&String>) -> String {
    if let Some(base_path) = base_path.to_owned() {
        let base_path_with_slash = if base_path.ends_with('/') {
            base_path.to_owned()
        } else {
            format!("{}/", base_path)
        };
        path.replace(&base_path_with_slash, "")
    } else {
        path.to_owned()
    }
}

pub fn find_matched_lambda(
    lambdas: &Vec<&Lambda>,
    method: &str,
    prepended_path: &str,
) -> Result<(Lambda, Event)> {
    for lambda in lambdas {
        for event in lambda.get_events() {
            let Some(event_props) = event.get_properties() else {
                warn!("No event properties found for event: {:?}", event);
                continue;
            };
            match event_props {
                EventProperties::Api(api_props) => {
                    let route_filter = if let Ok(route_match) =
                        api_props.get_route_regex().is_match(&prepended_path)
                    {
                        route_match
                    } else {
                        false
                    };

                    let method_filter = ["ANY", &method.to_uppercase()]
                        .contains(&api_props.get_method().to_uppercase().as_str());

                    if route_filter && method_filter {
                        return Ok((lambda.to_owned().clone(), event.to_owned()));
                    }
                }
                _ => {}
            }
        }
    }

    Err(anyhow::anyhow!("No matching Lambda found"))
}

pub fn create_api_request(
    body: Option<Json<serde_json::Value>>,
    headers: HeaderMap,
    params: HashMap<String, String>,
    method: Method,
    path: &str,
    base_path: &Option<&String>,
    request_id: &Uuid,
) -> ApiGatewayProxyRequest {
    debug!("Creating API Gateway request");
    let resource_path = remove_base_path(path, base_path);
    debug!("Resource path: {:?}", resource_path);

    let mut header_map = headers;
    header_map.insert(
        HeaderName::from_static("x-forwarded-proto"),
        HeaderValue::from_static("http"),
    );

    let request_context = create_api_request_context(path, request_id, &method, &header_map);
    let api_request = ApiGatewayProxyRequest {
        body: body.map(|b| b.0.to_string()),
        headers: header_map,
        http_method: method,
        is_base64_encoded: false,
        multi_value_headers: Default::default(),
        multi_value_query_string_parameters: Default::default(),
        path: Some(path.to_owned()),
        // path: Some(resource_path.to_owned()),
        path_parameters: vec![resource_path.to_owned()]
            .iter()
            .map(|path| ("path".to_owned(), path.to_owned()))
            .collect(),
        query_string_parameters: params.into(),
        request_context,
        resource: Some("/{path+}".to_string()),
        stage_variables: Default::default(),
    };

    trace!("API Gateway request: {:#?}", api_request);

    api_request
}

fn create_api_request_context(
    path: &str,
    request_id: &Uuid,
    method: &Method,
    headers: &HeaderMap,
) -> ApiGatewayProxyRequestContext {
    let dt = chrono::Local::now();
    let request_context: ApiGatewayProxyRequestContext = ApiGatewayProxyRequestContext {
        account_id: Some("123456789012".to_string()),
        apiid: Some("1234567890".to_owned()),
        resource_id: Some("123456".to_string()),
        // resource_path: Some(path.to_string()),
        resource_path: Some("/{path+}".to_owned()),
        path: Some(format!("/Prod{}", path).to_owned()),
        stage: Some("Prod".to_string()),
        domain_name: Some(headers.get("host").unwrap().to_str().unwrap().to_string()),
        domain_prefix: Some(headers.get("host").unwrap().to_str().unwrap().to_string()),
        request_id: Some(request_id.to_string()),
        protocol: Some("HTTP".to_string()),
        http_method: method.clone(),
        request_time: Some(dt.to_rfc3339()),
        request_time_epoch: dt.timestamp_millis(),
        operation_name: None,
        identity: create_api_request_identity(headers),
        authorizer: HashMap::new(),
    };

    request_context
}

fn create_api_request_identity(headers: &HeaderMap) -> ApiGatewayRequestIdentity {
    ApiGatewayRequestIdentity {
        access_key: None,
        account_id: None,
        api_key: None,
        api_key_id: None,
        caller: None,
        cognito_authentication_provider: None,
        cognito_authentication_type: None,
        cognito_identity_id: None,
        cognito_identity_pool_id: None,
        source_ip: Some("0.0.0.0".to_string()),
        user: None,
        user_agent: headers
            .get("user-agent")
            .map(|v| v.to_str().unwrap_or("unknown").to_string()),
        user_arn: None,
    }
}
