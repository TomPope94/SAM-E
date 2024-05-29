use crate::data::PutEventsRequest;
use sam_e_types::config::{Config, Infrastructure};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRequestItem {
    pub id: Uuid,
    #[serde(flatten)]
    pub event: PutEventsRequest,
}

pub struct EventRequestItemBuilder {
    id: Uuid,
    event: Option<PutEventsRequest>,
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

    pub fn with_event(mut self, event: PutEventsRequest) -> Self {
        self.event = Some(event);
        self
    }

    pub fn build(self) -> Result<EventRequestItem> {
        let event = self.event.ok_or_else(|| anyhow!("Event is required"))?;

        Ok(EventRequestItem { id: self.id, event })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStore {
    pub event_buses: HashMap<String, Vec<EventRequestItem>>,
}

impl EventStore {
    pub fn new() -> Self {
        Self {
            event_buses: HashMap::new(),
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

        Self { event_buses }
    }

    pub fn add_event(&mut self, event_bus_name: &str, event: EventRequestItem) {
        let events = self
            .event_buses
            .entry(event_bus_name.to_string())
            .or_insert_with(Vec::new);
        events.push(event);
    }

    pub fn get_events(&self, event_bus_name: &str) -> Option<&Vec<EventRequestItem>> {
        self.event_buses.get(event_bus_name)
    }
}
