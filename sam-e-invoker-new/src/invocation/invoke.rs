/// Not an AWS Lambda runtime api but the local invoker endpoint for passing invocations to the
/// invoker from event sources.
use crate::data::api::ApiState;
use sam_e_types::invocation::Invocation;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tokio::time::{sleep, Duration};
use tracing::{debug, info, trace};

pub async fn handler(
    State(api_state): State<ApiState>,
    Json(invocation): Json<Invocation>,
) -> impl IntoResponse {
    debug!("HELLO FROM THE INVOKER");
    debug!("Received invocation: {:#?}", invocation);
    "Hello world".to_string()
}
    
