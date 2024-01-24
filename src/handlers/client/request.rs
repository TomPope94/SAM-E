use crate::data::api::ApiState;
use axum::{
    extract::{Json, Path, Query, State},
    http::{HeaderMap, Method},
};
use std::collections::HashMap;
use tracing::{debug, info};

pub async fn request_handler(
    method: Method,
    headers: HeaderMap,
    Path(path): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(api_state): State<ApiState>,
    Json(body): Json<serde_json::Value>,
) -> String {
    info!("API Gateway received request");
    debug!("API Gateway received request with path: {:?}", path);
    debug!("API Gateway received request with method: {:?}", method);
    debug!("API Gateway received request with headers: {:?}", headers);
    debug!("API Gateway received request with params: {:?}", params);
    debug!("API Gateway received request with body: {:?}", body);

    let path_name = format!("Hello from: {}", path);
    path_name
}
