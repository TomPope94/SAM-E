use crate::data::{store::{EventRequestItemBuilder, EventStore}, PutEventsRequest, PutEventsResponse, PutEventsResponseBuilder, PutEventsResultEntryBuilder};

use anyhow::{anyhow, Result};
use tracing::{debug, trace};

pub async fn put_events_handler(put_events_request: PutEventsRequest, event_store: &EventStore) -> Result<PutEventsResponse> {
    debug!("Inside the put events handler");
    trace!("PutEvents request received: {:#?}", put_events_request);

    debug!("Adding event to event store");
    let mut entry_ids = Vec::new();
    for entry in put_events_request.entries {
        let Some(event_bus_name) = &entry.event_bus_name else {
            return Err(anyhow!("Event bus name is required"));
        };
        trace!("Adding entry to event bus: {:#?}", &entry.event_bus_name);
        
        let event_id = uuid::Uuid::new_v4();
        let new_event_request = EventRequestItemBuilder::new()
            .with_event(entry.clone())
            .with_id(event_id)
            .build()?;

        let mut event_buses = event_store.event_buses.write();
        event_buses.entry(event_bus_name.clone()).or_insert_with(Vec::new).push(new_event_request);
        debug!("Entry added to the event bus successfully");
        trace!("New state: {:#?}", event_buses);

        let resp = PutEventsResultEntryBuilder::new()
            .with_event_id(event_id)
            .build();

        entry_ids.push(resp);
    }

    debug!("PutEvents request handled successfully");

    let response = PutEventsResponseBuilder::new()
        .with_failed_entry_count(0)
        .with_entries(entry_ids)
        .build();
    Ok(response)
}
