use aws_lambda_events::{
    apigw::ApiGatewayProxyResponse,
    event::{apigw::ApiGatewayProxyRequest, sqs::SqsEvent},
};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Pending,
    Processing,
    Processed,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum EventRequest {
    Api(ApiGatewayProxyRequest),
    Sqs(SqsEvent),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Invocation
{
    request_id: Uuid,
    date_time: DateTime<Local>,
    status: Status,
    request: EventRequest,
    response: ApiGatewayProxyResponse,
    response_headers: HashMap<String, String>,
}

impl Invocation {
    pub fn new(request: EventRequest) -> Self {
        Self {
            request_id: Uuid::new_v4(),
            date_time: Local::now(),
            status: Status::Pending,
            request, 
            response: ApiGatewayProxyResponse::default(),
            response_headers: HashMap::new(),
        }
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

    pub fn get_request(&self) -> &EventRequest {
        &self.request
    }

    pub fn set_request(&mut self, request: EventRequest) {
        self.request = request;
    }

    pub fn get_response(&self) -> &ApiGatewayProxyResponse {
        &self.response
    }

    pub fn set_response(&mut self, response: ApiGatewayProxyResponse) {
        self.response = response;
    }

    pub fn get_response_headers(&self) -> &HashMap<String, String> {
        &self.response_headers
    }
    
    pub fn set_response_headers(&mut self, headers: HashMap<String, String>) {
        self.response_headers = headers;
    }
}
