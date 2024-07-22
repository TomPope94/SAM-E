use crate::data::PutEventsRequestEntry;
use sam_e_types::config::{Config, Infrastructure, infrastructure::{triggers::Triggers, event_rule::EventPattern}};

use anyhow::{anyhow, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tracing::{debug, trace, warn};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRule {
    pub triggers: Triggers,
    pub event_pattern: EventPattern,
}

#[derive(Debug, Clone)]
pub struct EventStore {
    pub event_buses: Arc<RwLock<HashMap<String, Vec<EventRequestItem>>>>,
    pub event_rules: Vec<EventRule>,
}

impl EventStore {
    pub fn new() -> Self {
        Self {
            event_buses: Arc::new(RwLock::new(HashMap::new())),
            event_rules: Vec::new(),
        }
    }

    pub fn from_config(config: &Config) -> Self {
        debug!("Building the event store from config...");

        let mut event_bus_names: Vec<String> = Vec::new();
        let mut event_rules: Vec<EventRule> = Vec::new();

        let infrastructure = config.get_infrastructure();

        for i in infrastructure {
            match i {
                Infrastructure::EventBus(event_bus) => {
                    let event_bus_props = &event_bus.properties;
                    debug!("Found event bus: {}", event_bus_props.name);
                    event_bus_names.push(event_bus_props.name.to_string());
                }
                Infrastructure::EventRule(event_rule) => {
                    let event_rule_props = &event_rule.properties;
                    debug!("Found event rule: {}", event_rule_props.name);

                    let event_rule = EventRule {
                        triggers: event_rule_props.triggers.clone().unwrap(), // TODO this unwrap
                        event_pattern: event_rule_props.event_pattern.clone(),
                    };
                    event_rules.push(event_rule);
                }
                _ => {}
            }
        }

        debug!("Adding all found event buses to the event store...");
        let mut event_buses = HashMap::new();
        for event_bus_name in event_bus_names {
            event_buses.insert(event_bus_name, Vec::new());
        }

        debug!("Event store built successfully!");

        Self { 
            event_buses: Arc::new(RwLock::new(event_buses)),
            event_rules,
        }
    }

    pub fn add_event(&mut self, event_bus_name: &str, event: EventRequestItem) {
        debug!("Event request received. Adding to event store...");
        trace!("Current event store: {:#?}", self);

        let mut event_buses = self.event_buses.write();
        let events = event_buses.entry(event_bus_name.to_string()).or_insert_with(Vec::new);
        events.push(event);

        debug!("Event added to event store");
        trace!("New event store: {:#?}", self);
    }

    pub async fn listen_for_events(&self, event_bus_name: String) {
        debug!("Listening for events for event_bus: {}", event_bus_name);
        let read_store = self.clone();

        tokio::spawn(async move {
            loop {
                debug!("Starting listening loop...");
                let mut processed_events: Vec<Uuid> = Vec::new();
                loop {
                    sleep(Duration::from_millis(500)).await;

                    let blank_events: Vec<EventRequestItem> = Vec::new();
                    let event_rules = read_store.event_rules.clone();
                    let event_buses = read_store.event_buses.read().clone();
                    let events = event_buses.get(&event_bus_name).unwrap_or(&blank_events);

                    if events.len() > 0 {
                        for event in events {
                            trace!("Event received: {:#?}", event);
                            debug!("Processing event...");

                            for rule in event_rules.iter() {
                                let event_pattern = &rule.event_pattern;
                                let mut matched = false;

                                if let Some(source) = &event_pattern.source {
                                    debug!("Checking source...");
                                    for s in source {
                                        if event.event.source == *s {
                                            debug!("Source matched");
                                            matched = true;
                                        } else {
                                            debug!("Source did not match");
                                            continue;
                                        }
                                    }
                                }

                                if let Some(detail_type) = &event_pattern.detail_type {
                                    debug!("Checking detail_type...");
                                    for dt in detail_type {
                                        if event.event.detail_type == *dt {
                                            debug!("Detail type matched");
                                            matched = true;
                                        } else {
                                            debug!("Detail type did not match");
                                            continue;
                                        }
                                    }
                                }

                                if matched {
                                    debug!("Event matched rule: {:#?}", rule);
                                    debug!("Sending event to trigger...");

                                    // let event_string = serde_json::to_string(&event).unwrap();
                                    let event_detail_as_value: serde_json::Value = serde_json::from_str(&event.event.detail).unwrap();

                                    let datetime = chrono::Utc::now();
                                    let lambda_event = aws_lambda_events::event::eventbridge::EventBridgeEvent {
                                        version: Some("0".to_string()),
                                        id: Some(event.id.to_string()),
                                        detail_type: event.event.detail_type.clone(),
                                        source: event.event.source.clone(),
                                        account: Some("123456789012".to_string()),
                                        time: Some(datetime),
                                        region: Some("eu-west-1".to_string()),
                                        resources: Some(vec!["resource1".to_string(), "resource2".to_string()]),
                                        detail: event_detail_as_value,
                                    };

                                    let event_string = serde_json::to_string(&lambda_event).unwrap();

                                    let send_res = rule.triggers.send(event_string).await;
                                    if let Err(e) = send_res {
                                        warn!("Error sending event to trigger: {:#?}", e);
                                    } else {
                                        debug!("Event sent to trigger successfully");
                                    }
                                }
                            }

                            processed_events.push(event.id);
                        }

                        debug!("Dropping out of read loop...");
                        break;
                    }
                }

                debug!("Removing processed events from event store...");
                let mut event_buses_write = read_store.event_buses.write();
                let events = event_buses_write.get_mut(&event_bus_name).unwrap();
                events.retain(|event| !processed_events.contains(&event.id));

                debug!("Events removed from event store");
            }
        });
    }

    pub fn get_event_bus_names(&self) -> Vec<String> {
        self.event_buses.read().keys().cloned().collect()
    }
}
