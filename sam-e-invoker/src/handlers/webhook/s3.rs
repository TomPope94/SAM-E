use crate::data::api::ApiState;

use aws_lambda_events::s3::S3Event;
use axum::{
    debug_handler,
    extract::{Json, Path, Query, State},
    http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode},
    response::IntoResponse,
};
use std::collections::HashMap;
use tracing::{debug, error, info, trace};

pub async fn handler(
    State(api_state): State<ApiState>,
    body: Json<S3Event>,
) -> impl IntoResponse {
    info!("Received a request to the S3 webhook");
    debug!("Request body: {:#?}", body);

    let s3_event = body.0;
    "s3"
}

