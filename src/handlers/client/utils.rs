use crate::data::{
    sam::Route,
    store::{Invocation, InvocationQueue, Status, Store},
};

use aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use tokio::time::{sleep, Duration};
use tracing::info;
use uuid::Uuid;

pub fn write_invocation_to_store(
    invocation: Invocation<ApiGatewayProxyRequest, ApiGatewayProxyResponse>,
    route: &Route,
    store: &Store,
) -> Uuid {
    let write_queue = InvocationQueue::new();

    store
        .queues
        .write()
        .entry(route.container_name.to_owned())
        .or_insert(write_queue)
        .api_invocations
        .push(invocation.to_owned());

    invocation.get_request_id().to_owned()
}

pub async fn read_invocation_from_store(
    store: &Store,
    container_name: &str,
    new_invocation_uuid: Uuid,
) -> Invocation<ApiGatewayProxyRequest, ApiGatewayProxyResponse> {
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
                .api_invocations
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
        .api_invocations
        .clone()
        .into_iter()
        .find(|invocation| {
            invocation.get_request_id() == &new_invocation_uuid
                && invocation.get_status() == &Status::Processed
        });

    info!("Processed invocation: {:?}", processed_invocation);

    processed_invocation.unwrap()
}
