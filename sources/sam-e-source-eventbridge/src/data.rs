pub mod store;

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum EventBridgeRequest {
    PutEvents(PutEventsRequest),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "PascalCase", serialize = "kebab-case"))]
pub struct PutEventsRequest {
    pub entries: Vec<PutEventsRequestEntry>,
}

// This should come from the aws-sdk-eventbridge crate but it doesn't implement the Deserialize/Serialize traits
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "PascalCase", serialize = "kebab-case"))]
pub struct PutEventsRequestEntry {
    pub time: Option<String>, // TODO: Change to DateTime
    pub source: String,
    pub resources: Option<Vec<String>>,
    pub detail_type: String,
    pub detail: String,
    pub event_bus_name: Option<String>,
    pub trace_header: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "PascalCase", serialize = "kebab-case"))]
pub struct PutEventsResultEntry {
    pub event_id: uuid::Uuid,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

pub struct PutEventsResultEntryBuilder {
    event_id: uuid::Uuid,
    error_code: Option<String>,
    error_message: Option<String>,
}

impl PutEventsResultEntryBuilder {
    pub fn new() -> Self {
        Self {
            event_id: uuid::Uuid::new_v4(),
            error_code: None,
            error_message: None,
        }
    }

    pub fn with_event_id(mut self, event_id: uuid::Uuid) -> Self {
        self.event_id = event_id;
        self
    }

    pub fn with_error_code(mut self, error_code: String) -> Self {
        self.error_code = Some(error_code);
        self
    }

    pub fn with_error_message(mut self, error_message: String) -> Self {
        self.error_message = Some(error_message);
        self
    }

    pub fn build(self) -> PutEventsResultEntry {
        PutEventsResultEntry {
            event_id: self.event_id,
            error_code: self.error_code,
            error_message: self.error_message,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PutEventsResponse {
    pub failed_entry_count: i64,
    pub entries: Vec<PutEventsResultEntry>,
}

pub struct PutEventsResponseBuilder {
    failed_entry_count: i64,
    entries: Vec<PutEventsResultEntry>,
}

impl PutEventsResponseBuilder {
    pub fn new() -> Self {
        Self {
            failed_entry_count: 0,
            entries: Vec::new(),
        }
    }

    pub fn with_failed_entry_count(mut self, failed_entry_count: i64) -> Self {
        self.failed_entry_count = failed_entry_count;
        self
    }

    pub fn with_entries(mut self, entries: Vec<PutEventsResultEntry>) -> Self {
        self.entries = entries;
        self
    }

    pub fn build(self) -> PutEventsResponse {
        PutEventsResponse {
            failed_entry_count: self.failed_entry_count,
            entries: self.entries,
        }
    }
}
