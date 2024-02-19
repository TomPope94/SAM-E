use crate::data::store::{EventSource, InvocationQueue, ResponseType, Status};

use aws_lambda_events::apigw::ApiGatewayProxyResponse;
use axum::{
    body::Bytes,
    debug_handler,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use std::str;
use tracing::{debug, error, info, trace};
use uuid::Uuid;

use crate::data::api::ApiState;

#[debug_handler]
pub async fn response_handler(
    headers: HeaderMap,
    Path((container_name, request_id)): Path<(String, Uuid)>,
    State(api_state): State<ApiState>,
    body: Bytes,
) -> impl IntoResponse {
    info!(
        "Response detected from lambda runtime for container: {}",
        container_name
    );
    let store = api_state.get_store();

    let headers_hashmap = headers
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
            debug!("Raw lambda response body: {:?}", body);

            invocation.set_response_headers(headers_hashmap);
            invocation.set_status(Status::Processed);

            match invocation.get_event_source() {
                EventSource::Api => {
                    debug!("Detected event source as API");
                    let response_data: ApiGatewayProxyResponse =
                        serde_json::from_slice(&body).unwrap();
                    invocation.set_response(ResponseType::Api(response_data));
                }
                EventSource::Sqs => {
                    debug!("Detected event source as SQS");
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
