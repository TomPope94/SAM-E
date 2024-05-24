use anyhow::{anyhow, Result};
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
    lambda_name: String,
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
            lambda_name: String::new(),
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

    pub fn get_lambda_name(&self) -> &String {
        &self.lambda_name
    }

    pub fn set_lambda_name(&mut self, name: String) {
        self.lambda_name = name;
    }
}

pub struct InvocationBuilder {
    request_id: Uuid,
    date_time: DateTime<Local>,
    status: Status,
    request: Option<EventRequest>,
    response: ApiGatewayProxyResponse,
    response_headers: HashMap<String, String>,
    lambda_name: Option<String>,
}

impl InvocationBuilder {
    pub fn new() -> Self {
        Self {
            request_id: Uuid::new_v4(),
            date_time: Local::now(),
            status: Status::Pending,
            request: None, 
            response: ApiGatewayProxyResponse::default(),
            response_headers: HashMap::new(),
            lambda_name: None,
        }
    }

    pub fn with_request_id(mut self, request_id: Uuid) -> Self {
        self.request_id = request_id;
        self
    }

    pub fn with_date_time(mut self, date_time: DateTime<Local>) -> Self {
        self.date_time = date_time;
        self
    }

    pub fn with_status(mut self, status: Status) -> Self {
        self.status = status;
        self
    }

    pub fn with_request(mut self, request: EventRequest) -> Self {
        self.request = Some(request);
        self
    }

    pub fn with_response(mut self, response: ApiGatewayProxyResponse) -> Self {
        self.response = response;
        self
    }

    pub fn with_response_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.response_headers = headers;
        self
    }

    pub fn with_lambda_name(mut self, name: String) -> Self {
        self.lambda_name = Some(name);
        self
    }

    pub fn build(self) -> Result<Invocation> {
        let Some(request) = self.request else {
            return Err(anyhow!("Request is required to build an invocation"));
        };

        let Some(lambda_name) = self.lambda_name else {
            return Err(anyhow!("Lambda name is required to build an invocation"));
        };

        Ok(Invocation {
            request_id: self.request_id,
            date_time: self.date_time,
            status: self.status,
            request,
            response: self.response,
            response_headers: self.response_headers,
            lambda_name,
        })
    }
}
