use crate::data::store::{Invocation, InvocationQueue, Status, Store};
use aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use tracing::{debug, trace};
use uuid::Uuid;

pub async fn find_invocation_and_process<'a>(
    store: &'a Store,
    container_name: &str,
    status_to_find: Option<Status>,
    invocation_id: Option<Uuid>,
    new_status: Option<Status>,
) -> Option<Invocation<ApiGatewayProxyRequest, ApiGatewayProxyResponse>> {
    let mut container_queue = get_container_queue(&store, container_name);

    let invocation = container_queue
        .api_invocations
        .iter_mut()
        .find(|invocation| {
            if let (Some(status_to_find), Some(invocation_id)) = (status_to_find, invocation_id) {
                // This is when getting response data from server. Will use both ID and ensure
                // it's been processed
                invocation.get_status().to_owned() == status_to_find
                    && invocation.get_request_id().to_owned() == invocation_id
            } else if let Some(status_to_find) = status_to_find {
                // This is when looking for a specific status (i.e. pending)
                invocation.get_status().to_owned() == status_to_find
            } else if let Some(invocation_id) = invocation_id {
                // This is for when lambda runtime asks for specific id (i.e. in response
                // handler)
                invocation.get_request_id().to_owned() == invocation_id
            } else {
                false
            }
        });

    if let Some(invocation) = invocation {
        debug!("Found an invocation");
        if let Some(new_status) = new_status {
            debug!(
                "Changing status from {:?} to {:?}",
                invocation.get_status(),
                new_status
            );
            invocation.set_status(new_status);
        }
        return Some(invocation.to_owned());
    } else {
        trace!(
            "No invocations found with desired status: {:?}",
            status_to_find
        );
        return None;
    }
}

fn get_container_queue(store: &Store, container_name: &str) -> InvocationQueue {
    let mut store_queues = store.queues.write();
    let container_queue = store_queues.get_mut(container_name);

    if let Some(container_queue) = container_queue {
        debug!("Found container queue");
        return container_queue.to_owned();
    } else {
        debug!("No container queue found, creating new one");
        let new_queue = InvocationQueue::new();
        store_queues.insert(container_name.to_string(), new_queue);
        return store_queues.get_mut(container_name).unwrap().to_owned();
    }
}
