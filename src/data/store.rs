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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum EventSource {
    Api,
    // Sqs,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum RequestType {
    Api(ApiGatewayProxyRequest),
    Sqs(SqsEvent),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ResponseType {
    Api(ApiGatewayProxyResponse),
    Sqs(SqsEventResponse),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Invocation {
    request_id: Uuid,
    date_time: DateTime<Local>,
    status: Status,
    event_source: EventSource,
    request: RequestType,
    response: ResponseType,
    response_headers: HashMap<String, String>,
    sqs_queue_url: Option<String>,
}

impl Invocation {
    pub fn new(event_source: EventSource) -> Self {
        match event_source {
            EventSource::Api => Self::new_api(),
            // EventSource::Sqs => Self::new_sqs(),
        }
    }

    pub fn new_api() -> Self {
        Self {
            request_id: Uuid::new_v4(),
            date_time: Local::now(),
            status: Status::Pending,
            event_source: EventSource::Api,
            request: RequestType::Api(ApiGatewayProxyRequest::default()),
            response: ResponseType::Api(ApiGatewayProxyResponse::default()),
            sqs_queue_url: None,
            response_headers: HashMap::new(),
        }
    }

    // pub fn new_sqs() -> Self {
    //     Self {
    //         request_id: Uuid::new_v4(),
    //         date_time: Local::now(),
    //         status: Status::Pending,
    //         request: RequestType::Sqs(SqsEvent::default()),
    //         response: ResponseType::Sqs(SqsEventResponse),
    //         event_source: EventSource::Sqs,
    //         sqs_queue_url: None,
    //         response_headers: HashMap::new(),
    //     }
    // }

    pub fn update_status(&mut self, status: Status) {
        self.status = status;
    }

    pub fn get_event_source(&self) -> &EventSource {
        &self.event_source
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

    pub fn get_request(&self) -> &RequestType {
        &self.request
    }

    pub fn set_request(&mut self, request: RequestType) {
        self.request = request;
    }

    pub fn get_response(&self) -> &ResponseType {
        &self.response
    }

    pub fn set_response(&mut self, response: ResponseType) {
        self.response = response;
    }

    pub fn set_response_headers(&mut self, headers: HashMap<String, String>) {
        self.response_headers = headers;
    }
    pub fn get_response_headers(&self) -> &HashMap<String, String> {
        &self.response_headers
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InvocationQueue {
    invocations: Vec<Invocation>,
}

impl InvocationQueue {
    pub fn new() -> Self {
        Self {
            invocations: Vec::new(),
        }
    }

    pub fn add_invocation(&mut self, invocation: Invocation) {
        self.invocations.push(invocation);
    }

    pub fn get_invocations(&self) -> &Vec<Invocation> {
        &self.invocations
    }
    pub fn get_invocations_mut(&mut self) -> &mut Vec<Invocation> {
        &mut self.invocations
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
