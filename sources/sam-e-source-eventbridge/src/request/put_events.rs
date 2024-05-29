use crate::data::PutEventsRequest;

use anyhow::Result;
use tracing::debug;

pub async fn put_events_handler(put_events_request: PutEventsRequest) -> Result<()> {
    debug!("Inside the put events handler");
    debug!("PutEvents request received: {:#?}", put_events_request);

    Ok(())
}
