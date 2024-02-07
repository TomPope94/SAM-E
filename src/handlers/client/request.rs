use crate::{
    data::{
        api::ApiState,
        store::{EventSource, Invocation, RequestType, ResponseType},
    },
    handlers::client::utils::{
        create_api_request, find_matched_route, read_invocation_from_store,
        write_invocation_to_store,
    },
};
use aws_lambda_events::encodings::Body;
use axum::{
    debug_handler,
    extract::{Json, Path, Query, State},
    http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode},
    response::IntoResponse,
};
use std::collections::HashMap;
use tracing::debug;

#[debug_handler]
pub async fn request_handler(
    method: Method,
    headers: HeaderMap,
    Path(path): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(api_state): State<ApiState>,
    body: Option<Json<serde_json::Value>>,
) -> impl IntoResponse {
    if let Some(routes) = api_state.get_routes_vec() {
        debug!("Routes detected. Checking for route match");

        let prepended_path = format!("/{}", path); // Axum doesn't prepend the path with a slash
        let matched_route = find_matched_route(routes, &method.to_string(), &prepended_path);

        if let Some(matched_route) = matched_route {
            debug!("Route match found: {:?}", matched_route);
            debug!("Now adding invocation to store");

            // Creates an empty invocation with default empty request and response of correct type
            let mut new_invocation = Invocation::new(EventSource::Api);

            // Add api_data to the invocation request
            let request_id = new_invocation.get_request_id().clone();
            let api_data = create_api_request(
                body,
                headers,
                params,
                method,
                &prepended_path,
                matched_route.get_route_base_path(),
                &request_id,
            );
            new_invocation.set_request(RequestType::Api(api_data));

            // Write invocation to store where it'll be picked up by /next endpoint on lambda
            // runtime api
            let _ = write_invocation_to_store(new_invocation, &matched_route, api_state.get_store());

            let processed_invocation = read_invocation_from_store(
                api_state.get_store(),
                &matched_route.container_name,
                request_id,
            )
            .await;

            let res_headers = processed_invocation.get_response_headers();
            let res_body = processed_invocation.get_response();

            let mut header_map = HeaderMap::new();

            res_headers.iter().for_each(|(key, value)| {
                header_map.insert(
                    HeaderName::try_from(key.as_str())
                        .unwrap_or_else(|_| HeaderName::from_static("unknown")),
                    HeaderValue::try_from(value.as_str()).unwrap_or_else(|_| {
                        HeaderValue::from_static("Unable to convert header to string.")
                    }),
                );
            });

            match res_body {
                ResponseType::Api(api_response) => {
                    let status_code = StatusCode::from_u16(api_response.status_code.try_into().unwrap_or(500))
                        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

                    debug!("Returning response with status code: {:?}", status_code);
                    debug!("Returning response with headers: {:?}", header_map);
                    debug!("Returning response with body: {:?}", api_response);

                    if let Some(payload) = api_response.body.clone() {
                        match Body::from(payload) {
                            Body::Text(payload) => {
                                let json_map: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&payload).unwrap_or_default();
                                Json(json_map)
                            }
                            Body::Binary(payload) => {
                                let json_map: serde_json::Map<String, serde_json::Value> = serde_json::from_slice(&payload).unwrap_or_default();
                                Json(json_map)
                            }
                            Body::Empty => {
                                let mut json_map: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
                                json_map.insert("message".to_string(), serde_json::Value::String("Empty response body".to_string()));
                                Json(json_map)
                            }
                        }
                    } else {
                        let mut json_map: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
                        json_map.insert(
                            "message".to_string(),
                            serde_json::Value::String("No response body".to_string()),
                        );
                        Json(json_map)
                    }
                }
                _ => {
                    debug!("No response body found");
                    let mut json_map: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
                    json_map.insert(
                        "message".to_string(),
                        serde_json::Value::String("No response body".to_string()),
                    );
                    Json(json_map)
                }
            }
        } else {
            debug!("No route match found");

            let mut json_map: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
            json_map.insert(
                "message".to_string(),
                serde_json::Value::String("Found routes but no route match found".to_string()),
            );
            Json(json_map)
        }
    } else {
        debug!("No routes detected.");

        let mut json_map: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
        json_map.insert(
            "message".to_string(),
            serde_json::Value::String("No routes detected".to_string()),
        );
        Json(json_map)
    }
}
