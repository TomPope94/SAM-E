use std::{collections::HashMap, sync::Arc};

use aws_lambda_events::{
    apigw::ApiGatewayProxyResponse,
    event::{apigw::ApiGatewayProxyRequest, sqs::SqsEvent},
    streams::SqsEventResponse,
};
use chrono::{DateTime, Local};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
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
pub struct Invocation<T, R> {
    request_id: Uuid,
    date_time: DateTime<Local>,
    status: Status,
    pub event: T,
    event_source: EventSource,
    sqs_queue_url: Option<String>,
    pub response_headers: HashMap<String, String>,
    pub response_body: R,
}

impl<T, R> Invocation<T, R> {
    pub fn update_status(&mut self, status: Status) {
        self.status = status;
    }

    pub fn get_status(&self) -> &Status {
        &self.status
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status;
    }

    pub fn get_request_id(&self) -> &Uuid {
        &self.request_id
    }

    pub fn get_event_source(&self) -> &EventSource {
        &self.event_source
    }

    pub fn get_event(&self) -> &T {
        &self.event
    }

    pub fn set_response_headers(&mut self, headers: HashMap<String, String>) {
        self.response_headers = headers;
    }
    pub fn get_response_headers(&self) -> &HashMap<String, String> {
        &self.response_headers
    }

    pub fn set_response_body(&mut self, body: R) {
        self.response_body = body;
    }
    pub fn get_response_body(&self) -> &R {
        &self.response_body
    }
}

impl Invocation<ApiGatewayProxyRequest, ApiGatewayProxyResponse> {
    pub fn new(event: ApiGatewayProxyRequest, request_id: Uuid) -> Self {
        Self {
            request_id,
            date_time: Local::now(),
            status: Status::Pending,
            event,
            event_source: EventSource::Api,
            sqs_queue_url: None,
            response_headers: HashMap::new(),
            response_body: ApiGatewayProxyResponse::default(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InvocationQueue {
    pub api_invocations: Vec<Invocation<ApiGatewayProxyRequest, ApiGatewayProxyResponse>>,
    pub sqs_invocations: Vec<Invocation<SqsEvent, SqsEventResponse>>,
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
    pub queues: Arc<RwLock<InvocationQueues>>,
}

impl Store {
    pub fn new() -> Self {
        Store {
            queues: Arc::new(RwLock::new(InvocationQueues::new())),
        }
    }
}
