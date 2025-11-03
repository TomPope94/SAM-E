use anyhow::{anyhow, Result};
use aws_lambda_events::{
    apigw::ApiGatewayProxyResponse,
    event::{apigw::ApiGatewayProxyRequest, sqs::SqsEvent},
};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum StreamChunk {
    Data(Vec<u8>),      // Payload data
    Complete,           // Stream finished successfully
    Error(String),      // Stream error
}

#[derive(Debug, Clone)]
pub struct Extension {
    pub id: String,              // UUID assigned to this extension
    pub name: String,            // Extension name from registration
    event_sender: mpsc::UnboundedSender<ExtensionEvent>,
}

impl Extension {
    pub fn new(
        id: String,
        name: String,
        event_sender: mpsc::UnboundedSender<ExtensionEvent>,
    ) -> Self {
        Self {
            id,
            name,
            event_sender,
        }
    }

    pub fn get_event_sender(&self) -> &mpsc::UnboundedSender<ExtensionEvent> {
        &self.event_sender
    }
}

#[derive(Debug, Clone)]
pub enum ExtensionEvent {
    Invoke {
        request_id: String,
        deadline_ms: u64,
        invoked_function_arn: String,
    },
    Shutdown {
        reason: String,
        deadline_ms: u64,
    },
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Pending,
    Processing,
    Streaming,          // Lambda is streaming response
    Processed,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum EventRequest {
    Api(ApiGatewayProxyRequest),
    Sqs(SqsEvent),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Invocation {
    request_id: Uuid,
    date_time: DateTime<Local>,
    status: Status,
    request: EventRequest,
    response: ApiGatewayProxyResponse,
    response_headers: HashMap<String, String>,
    lambda_name: String,
    #[serde(skip)]
    stream_sender: Option<mpsc::UnboundedSender<StreamChunk>>,
    #[serde(skip)]
    extensions: Vec<Extension>,
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
            stream_sender: None,
            extensions: Vec::new(),
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

    pub fn set_stream_sender(&mut self, sender: mpsc::UnboundedSender<StreamChunk>) {
        self.stream_sender = Some(sender);
    }

    pub fn get_stream_sender(&self) -> Option<&mpsc::UnboundedSender<StreamChunk>> {
        self.stream_sender.as_ref()
    }

    pub fn send_chunk(&self, chunk: StreamChunk) -> Result<()> {
        if let Some(sender) = &self.stream_sender {
            sender.send(chunk).map_err(|e| anyhow!("Failed to send chunk: {}", e))?;
            Ok(())
        } else {
            Err(anyhow!("No stream sender available"))
        }
    }

    // Extension management methods
    pub fn add_extension(&mut self, extension: Extension) {
        self.extensions.push(extension);
    }

    pub fn get_extensions(&self) -> &Vec<Extension> {
        &self.extensions
    }

    pub fn get_extension_by_id(&self, id: &str) -> Option<&Extension> {
        self.extensions.iter().find(|ext| ext.id == id)
    }

    pub fn send_event_to_all_extensions(&self, event: ExtensionEvent) -> Result<()> {
        for extension in &self.extensions {
            extension.event_sender
                .send(event.clone())
                .map_err(|e| anyhow!("Failed to send event to extension {}: {}", extension.name, e))?;
        }
        Ok(())
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
    stream_sender: Option<mpsc::UnboundedSender<StreamChunk>>,
    extensions: Vec<Extension>,
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
            stream_sender: None,
            extensions: Vec::new(),
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

    pub fn with_stream_sender(mut self, sender: mpsc::UnboundedSender<StreamChunk>) -> Self {
        self.stream_sender = Some(sender);
        self
    }

    pub fn with_extensions(mut self, extensions: Vec<Extension>) -> Self {
        self.extensions = extensions;
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
            stream_sender: self.stream_sender,
            extensions: self.extensions,
        })
    }
}
