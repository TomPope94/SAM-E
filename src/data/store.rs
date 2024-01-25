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
pub struct Invocation<T> {
    uuid: Uuid,
    date_time: DateTime<Local>,
    status: Status,
    event: T,
    event_source: EventSource,
    sqs_queue_url: Option<String>,
    response_headers: HashMap<String, Vec<String>>,
    response_body: Value,
}

impl<T> Invocation<T> {
    pub fn update_status(&mut self, status: Status) {
        self.status = status;
    }

    pub fn get_status(&self) -> &Status {
        &self.status
    }

    pub fn get_uuid(&self) -> &Uuid {
        &self.uuid
    }
}

impl Invocation<ApiGatewayV2httpRequest> {
    pub fn new(event: ApiGatewayV2httpRequest) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            date_time: Local::now(),
            status: Status::Pending,
            event,
            event_source: EventSource::Api,
            sqs_queue_url: None,
            response_headers: HashMap::new(),
            response_body: json!({}),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InvocationQueue {
    pub api_invocations: Vec<Invocation<ApiGatewayV2httpRequest>>,
    pub sqs_invocations: Vec<Invocation<SqsEvent>>,
}

impl InvocationQueue {
    pub fn new() -> Self {
        Self {
            api_invocations: vec![],
            sqs_invocations: vec![],
        }
    }
}

pub type InvocationQueues = HashMap<String, InvocationQueue>;

#[derive(Clone, Debug)]
pub struct Store {
    queues: Arc<RwLock<InvocationQueues>>,
}

impl Store {
    pub fn new() -> Self {
        Store {
            queues: Arc::new(RwLock::new(InvocationQueues::new())),
        }
    }

    pub fn get_queues(&self) -> Arc<RwLock<InvocationQueues>> {
        self.queues.clone()
    }
}
