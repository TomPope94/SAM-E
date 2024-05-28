use aws_sdk_eventbridge::types::PutEventsRequestEntry;

pub enum EventBridgeRequest {
    PutEvents(PutEventsRequest),
}

pub struct PutEventsRequest {
    pub entries: Vec<PutEventsRequestEntry>,
}

