use crate::data::api::ApiState;

use axum::{
    extract::{Json, Path, State},
    http::HeaderMap,
};
use tracing::{debug, info};
use uuid::Uuid;

pub async fn response_handler(
    headers: HeaderMap,
    Path((container_name, request_id)): Path<(String, Uuid)>,
    State(_api_state): State<ApiState>,
    body: Option<Json<serde_json::Value>>,
) -> Json<serde_json::Value> {
    info!("Error with invocation. See logs for details");
    debug!("Headers: {:?}", headers);
    debug!("Container name: {:?}", container_name);
    debug!("Request ID: {:?}", request_id);
    debug!("Error Body: {:?}", body);
    Json(serde_json::json!({
        "error": "Invocation error. See logs for details"
    }))
}
