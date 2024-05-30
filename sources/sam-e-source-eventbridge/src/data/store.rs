use crate::data::PutEventsRequestEntry;
use sam_e_types::config::{Config, Infrastructure};

use anyhow::{anyhow, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tracing::debug;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRequestItem {
    pub id: Uuid,
    #[serde(flatten)]
    pub event: PutEventsRequestEntry,
}

pub struct EventRequestItemBuilder {
    id: Uuid,
    event: Option<PutEventsRequestEntry>,
}

impl EventRequestItemBuilder {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            event: None,
        }
    }

    pub fn with_id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }

    pub fn with_event(mut self, event: PutEventsRequestEntry) -> Self {
        self.event = Some(event);
        self
    }

    pub fn build(self) -> Result<EventRequestItem> {
        let event = self.event.ok_or_else(|| anyhow!("Event is required"))?;

        Ok(EventRequestItem { id: self.id, event })
    }
}

#[derive(Debug, Clone)]
pub struct EventStore {
    pub event_buses: Arc<RwLock<HashMap<String, Vec<EventRequestItem>>>>,
}

impl EventStore {
    pub fn new() -> Self {
        Self {
            event_buses: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn from_config(config: &Config) -> Self {
        debug!("Building the event store from config...");
        let event_bus_names: Vec<String> = config
            .get_infrastructure()
            .iter()
            .filter_map(|infra| match infra {
                Infrastructure::EventBus(event_bus) => {
                    let event_bus_props = &event_bus.properties;
                    debug!("Found event bus: {}", event_bus_props.name);
                    Some(event_bus_props.name.to_string())
                }
                _ => None,
            })
            .collect();

        debug!("Adding all found event buses to the event store...");
        let mut event_buses = HashMap::new();
        for event_bus_name in event_bus_names {
            event_buses.insert(event_bus_name, Vec::new());
        }

        debug!("Event store built successfully!");

        Self { event_buses: Arc::new(RwLock::new(event_buses)) }
    }

    pub fn add_event(&mut self, event_bus_name: &str, event: EventRequestItem) {
        debug!("Event request received. Adding to event store...");
        debug!("Current event store: {:#?}", self);

        let mut event_buses = self.event_buses.write();
        let events = event_buses.entry(event_bus_name.to_string()).or_insert_with(Vec::new);
        events.push(event);

        debug!("Event added to event store");
        debug!("New event store: {:#?}", self);
    }

    pub fn listen_for_events(&self, event_bus_name: String) {
        debug!("Listening for events for event_bus: {}", event_bus_name);
        let read_store = self.clone();

        tokio::spawn(async move {
            loop {
                debug!("Starting listening loop...");
                let mut processed_events: Vec<Uuid> = Vec::new();
                loop {
                    sleep(Duration::from_millis(500)).await;

                    let blank_events: Vec<EventRequestItem> = Vec::new();
                    let event_buses = read_store.event_buses.read();
                    let events = event_buses.get(&event_bus_name).unwrap_or(&blank_events);

                    if events.len() > 0 {
                        for event in events {
                            debug!("Event received: {:#?}", event);
                            debug!("Processing event...");
                            processed_events.push(event.id);
                        }

                        debug!("Dropping out of read loop...");
                        break;
                    }
                }

                debug!("Removing processed events from event store...");
                let mut event_buses = read_store.event_buses.write();
                let events = event_buses.get_mut(&event_bus_name).unwrap();
                events.retain(|event| !processed_events.contains(&event.id));

                debug!("Events removed from event store");
            }
        });
    }

    pub fn get_event_bus_names(&self) -> Vec<String> {
        self.event_buses.read().keys().cloned().collect()
    }
}
