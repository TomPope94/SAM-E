use crate::data::{api::ApiState, store::InvocationQueue};

use sam_e_types::invocation::{EventRequest, Status};

use axum::{
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use tokio::time::{sleep, Duration};
use tracing::{debug, info, trace, warn};

/// Handler for /next that extracts container name from IP address mapping
/// This allows Lambda runtimes to use standard format: AWS_LAMBDA_RUNTIME_API=host:port
pub async fn request_handler(
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    State(api_state): State<ApiState>,
) -> impl IntoResponse {
    let source_ip = addr.ip().to_string();

    debug!(
        "Received /next request from IP: {}, headers: {:?}",
        source_ip, headers
    );

    let container_name = match api_state.get_function_name_from_ip(&source_ip).await {
        Ok(name) => name,
        Err(_) => {
            warn!(
                "Function name for IP {} not found in mapping, using 'unknown'",
                source_ip
            );
            "unknown".to_string()
        }
    };

    trace!("Processing request for function: {}", container_name);
    trace!("Current state: {:#?}", api_state);

    let store = api_state.get_store();

    let read_store = store.clone();
    let read_queue = InvocationQueue::new();
    let read_container_name = container_name.clone();

    let invocation = tokio::task::spawn(async move {
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
                .any(|invocation| invocation.get_status() == &Status::Pending)
            {
                info!(
                    "Found a pending invocation for container: {}",
                    read_container_name
                );
                break;
            }
        }
    })
    .await;

    // Will only reach here once a pending invocation has been found
    debug!(
        "Processing the invocation for container: {}",
        container_name
    );

    let write_store = store.clone();
    let write_queue = InvocationQueue::new();
    let write_container_name = container_name.clone();

    let invocation_to_process = match write_store
        .queues
        .write()
        .entry(write_container_name)
        .or_insert(write_queue)
        .get_invocations_mut()
        .iter_mut()
        .find(|invocation| invocation.get_status() == &Status::Pending)
    {
        Some(invocation) => {
            invocation.set_status(Status::Processing);

            trace!("Invocation: {:#?}", invocation);

            Some(invocation.to_owned())
        }
        None => None,
    };

    // Return the response
    if invocation.is_ok() && invocation_to_process.is_some() {
        let invocation_data = invocation_to_process.unwrap();
        let Some(time_delta) = chrono::Duration::try_days(1) else {
            panic!("Failed to create time delta");
        };

        let dt = chrono::Local::now() + time_delta;

        let event_request = invocation_data.get_request();

        match event_request {
            EventRequest::Api(api_request) => {
                debug!("Detected invocation source as API Gateway");
                debug!("Event being sent: {:#?}", api_request);

                let data_as_value = serde_json::to_value(api_request).unwrap();

                return (
                    StatusCode::OK,
                    [
                        (
                            "lambda-runtime-aws-request-id",
                            invocation_data.get_request_id().to_string(),
                        ),
                        ("lambda-runtime-deadline-ms", dt.timestamp().to_string()),
                    ],
                    Json(data_as_value),
                );
            }
            EventRequest::Sqs(sqs_request) => {
                debug!("Processing an SQS invocation");

                let data_as_value = serde_json::to_value(sqs_request).unwrap();

                return (
                    StatusCode::OK,
                    [
                        (
                            "lambda-runtime-aws-request-id",
                            invocation_data.get_request_id().to_string(),
                        ),
                        ("lambda-runtime-deadline-ms", dt.timestamp().to_string()),
                    ],
                    Json(data_as_value),
                );
            }
        }
    } else {
        return (
            StatusCode::OK,
            [
                (
                    "lambda-runtime-aws-request-id",
                    "1111-11111-11111".to_string(),
                ),
                ("lambda-runtime-deadline-ms", "1600000000000".to_string()),
            ],
            Json(serde_json::json!({
                "message": "No pending invocations found"
            })),
        );
    }
}
