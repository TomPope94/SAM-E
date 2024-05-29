mod put_events;

use crate::data::EventBridgeRequest;
use crate::response::AppError;

use axum::response::IntoResponse;
use tracing::debug;

// Event bridge request is parsed to correct type via the middleware
pub async fn handler(
    event_bridge_request: EventBridgeRequest,
) -> Result<impl IntoResponse, AppError> {
    match event_bridge_request {
        EventBridgeRequest::PutEvents(put_events_request) => {
            debug!("Recognised a put events request");
            put_events::put_events_handler(put_events_request).await?;

            Ok("PutEvents request handled successfully".into_response())
        }
    }
}
