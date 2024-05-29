pub mod store;

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum EventBridgeRequest {
    PutEvents(PutEventsRequest),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PutEventsRequest {
    pub entries: Vec<PutEventsRequestEntry>,
}

// This should come from the aws-sdk-eventbridge crate but it doesn't implement the Deserialize/Serialize traits
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PutEventsRequestEntry {
    pub time: Option<String>, // TODO: Change to DateTime
    pub source: Option<String>,
    pub resources: Option<Vec<String>>,
    pub detail_type: Option<String>,
    pub detail: Option<String>,
    pub event_bus_name: Option<String>,
    pub trace_header: Option<String>,
}
