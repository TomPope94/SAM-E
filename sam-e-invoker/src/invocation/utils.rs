use crate::data::store::{InvocationQueue, Store};
use sam_e_types::invocation::{Invocation, Status};

use anyhow::Result;
use tokio::time::{sleep, Duration};
use tracing::{debug, info, trace};
use uuid::Uuid;

pub fn write_invocation_to_store(
    invocation: Invocation,
    store: &Store,
) -> Result<()> {
    debug!("Getting write queue...");
    let write_queue = InvocationQueue::new();

    debug!("Writing invocation to store...");
    store
        .queues
        .write()
        .entry(invocation.get_lambda_name().to_owned())
        .or_insert(write_queue)
        .get_invocations_mut()
        .push(invocation.to_owned());

    info!("Invocation written to the store successfully");
    trace!("Invocation details: {:?}", invocation);
    Ok(())
}

pub async fn read_invocation_from_store(
    store: &Store,
    container_name: &str,
    new_invocation_uuid: Uuid,
) -> Result<Invocation> {
    debug!("Reading invocation from store...");
    let read_store = store.clone();
    let read_queue = InvocationQueue::new();
    let read_container_name = container_name.to_owned();

    // TODO: add a timeout to this loop
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
                info!("Found a processed invocation");
                debug!(
                    "Processed invocation for container: {}",
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

    Ok(processed_invocation.unwrap())
}


