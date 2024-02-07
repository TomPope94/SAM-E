use crate::data::api::ApiState;

use axum::{
    extract::{Json, Path, State},
    http::HeaderMap,
};
use tracing::{debug, info};

pub async fn response_handler(
    headers: HeaderMap,
    Path(container_name): Path<String>,
    State(_api_state): State<ApiState>,
    body: Option<Json<serde_json::Value>>,
) -> Json<serde_json::Value> {
    info!("Error with initiation. See logs for details");
    debug!("Headers: {:?}", headers);
    debug!("Container name: {:?}", container_name);
    debug!("Error Body: {:?}", body);
    Json(serde_json::json!({
        "error": "Initiation error. See logs for details"
    }))
}
