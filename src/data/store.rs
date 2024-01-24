use std::{collections::HashMap, sync::Arc};

use aws_lambda_events::event::{apigw::ApiGatewayV2httpRequest, sqs::SqsEvent};
use chrono::{DateTime, Local};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Pending,
    Processing,
    Processed,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
pub enum EventSource {
    Api,
    Sqs,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Event {
    Api(ApiGatewayV2httpRequest),
    Sqs(SqsEvent),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Invocation {
    uuid: Uuid,
    date_time: DateTime<Local>,
    status: Status,
    event: Event,
    event_source: EventSource,
    sqs_queue_url: Option<String>,
    response_headers: HashMap<String, Vec<String>>,
    response_body: Value,
}

impl Invocation {
    pub fn new(event: Event, event_source: EventSource, sqs_queue_url: Option<String>) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            date_time: Local::now(),
            status: Status::Pending,
            event,
            event_source,
            sqs_queue_url,
            response_headers: HashMap::new(),
            response_body: json!({}),
        }
    }

    pub fn update_status(&mut self, status: Status) {
        self.status = status;
    }

    pub fn get_status(&self) -> Status {
        self.status
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InvocationQueue {
    pub invocations: Vec<Invocation>,
}

impl InvocationQueue {
    pub fn new() -> Self {
        Self {
            invocations: vec![],
        }
    }
}

pub type InvocationQueues = HashMap<String, InvocationQueue>;

#[derive(Clone, Debug)]
pub struct Store {
    pub queues: Arc<RwLock<InvocationQueues>>,
}

impl Store {
    pub fn new() -> Self {
        Store {
            queues: Arc::new(RwLock::new(InvocationQueues::new())),
        }
    }
}
