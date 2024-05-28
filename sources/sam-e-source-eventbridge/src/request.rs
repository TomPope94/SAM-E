use crate::response::AppError;
use crate::data::EventBridgeRequest;

use axum::response::IntoResponse;
use tracing::debug;

// Event bridge request is parsed to correct type via the middleware
pub async fn handler(event_bridge_request: EventBridgeRequest) -> Result<impl IntoResponse, AppError> {
    match event_bridge_request {
        EventBridgeRequest::PutEvents(put_events_request) => {
            debug!("PutEvents request received: {:#?}", put_events_request);
            Ok("PutEvents request received".into_response())
        }
    }
}
