use crate::data::{EventBridgeRequest, PutEventsRequest};

use axum::{
    async_trait,
    body::{Body, Bytes},
    extract::{FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use tracing::{debug, error};

#[async_trait]
impl<S> FromRequest<S> for EventBridgeRequest
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        debug!("Parsing the EventBridge request middleware...");
        // Get the X-Amz-Target header value
        let Some(target_header) = req.headers().get("X-Amz-Target") else {
            error!("No X-Amz-Target header found");
            return Err(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::empty())
                .unwrap());
        };
        let Ok(target_header_str) = target_header.to_str() else {
            error!("X-Amz-Target header could not be parsed");
            return Err(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::empty())
                .unwrap());
        };

        // Parse the X-Amz-Target header Header
        debug!("Header detected, now parsing...");
        let target_header_string = target_header_str.to_string();
        let target_header_parts: Vec<&str> = target_header_string.split('.').collect();
        if target_header_parts.len() != 2 {
            return Err(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::empty())
                .unwrap());
        }

        debug!("Header parsed successfully, now parsing the body...");
        let body = Bytes::from_request(req, state)
            .await
            .map_err(IntoResponse::into_response)?;
        let Ok(body_str) = std::str::from_utf8(&body) else {
            error!("Body could not be parsed into a string");
            return Err(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::empty())
                .unwrap());
        };

        // Match the X-Amz-Target header Header
        match target_header_parts[1] {
            "PutEvents" => {
                debug!("Detected a PutEvents request");
                let Ok(put_events_request) = serde_json::from_str::<PutEventsRequest>(body_str)
                else {
                    error!("Body could not be parsed into a PutEventsRequest");
                    return Err(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(Body::empty())
                        .unwrap());
                };
                Ok(EventBridgeRequest::PutEvents(put_events_request))
            }
            _ => {
                error!("No matching target header found");
                Err(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::empty())
                    .unwrap())
            }
        }
    }
}
