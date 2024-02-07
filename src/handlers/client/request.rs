use crate::{
    data::{api::ApiState, store::Invocation},
    handlers::client::utils::{read_invocation_from_store, write_invocation_to_store},
};
use aws_lambda_events::{
    encodings::Body,
    apigw::{
    ApiGatewayProxyRequest, ApiGatewayProxyRequestContext, ApiGatewayRequestIdentity, ApiGatewayProxyResponse,
}};
use axum::{
    debug_handler,
    extract::{Json, Path, Query, State},
    http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode},
    response::IntoResponse,
};
use std::collections::HashMap;
use tracing::{debug, info, trace};
use uuid::Uuid;

#[debug_handler]
pub async fn request_handler(
    method: Method,
    headers: HeaderMap,
    Path(path): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(api_state): State<ApiState>,
    body: Option<Json<serde_json::Value>>,
) -> impl IntoResponse {
    info!("API Gateway received request");
    trace!("API Gateway received request with path: {:?}", path);
    trace!("API Gateway received request with method: {:?}", method);
    trace!("API Gateway received request with headers: {:?}", headers);
    trace!("API Gateway received request with params: {:?}", params);
    trace!("API Gateway received request with body: {:?}", body);

    // TODO: add an API invocation to the store (passed via state)
    if let Some(routes) = api_state.get_routes_vec() {
        debug!("Routes detected. Checking for route match");
        let prepended_path = format!("/{}", path);

        // Find the route that matches the path and Method
        let matched_route = routes.into_iter().find(|route| {
            let route_filter = if let Ok(route_match) = route.route_regex.is_match(&prepended_path)
            {
                route_match
            } else {
                false
            };

            let method_str = method.to_string().to_uppercase();
            let method_filter =
                ["ANY", &method_str].contains(&route.method.to_uppercase().as_str());

            route_filter && method_filter
        });

        if let Some(matched_route) = matched_route {
            debug!("Route match found: {:?}", matched_route);
            debug!("Now adding invocation to store");

            let request_id = Uuid::new_v4();

            // remove base path from path
            let resource_path = if let Some(base_path) = &matched_route.route_base_path {
                debug!("Removing base path from path");
                debug!("Base path: {:?}", base_path);

                prepended_path.replace(base_path, "")  
            } else {
                prepended_path.clone()
            };

            debug!("Resource path: {:?}", resource_path);

            let api_data = create_api_request(body, headers, params, method, &resource_path, &request_id);

            let new_invocation: Invocation<ApiGatewayProxyRequest, ApiGatewayProxyResponse> =
                Invocation::new(api_data, request_id);

            let invocation_id =
                write_invocation_to_store(new_invocation, &matched_route, api_state.get_store());

            debug!("Invocation ID: {:?}", invocation_id);

            let processed_invocation = read_invocation_from_store(
                api_state.get_store(),
                &matched_route.container_name,
                invocation_id,
            )
            .await;

            let res_headers = processed_invocation.get_response_headers();
            let res_body = processed_invocation.get_response_body();

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

            let status_code = StatusCode::from_u16(res_body.status_code.try_into().unwrap_or(500))
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

            debug!("Returning response with status code: {:?}", status_code);
            debug!("Returning response with headers: {:?}", header_map);
            debug!("Returning response with body: {:?}", res_body);

            if let Some(payload) = res_body.body.clone() {
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
                json_map.insert("message".to_string(), serde_json::Value::String("No response body".to_string()));
                Json(json_map)
            }
        } else {
            debug!("No route match found");

            let mut json_map: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
            json_map.insert("message".to_string(), serde_json::Value::String("Found routes but no route match found".to_string()));
            Json(json_map)
        }
    } else {
        debug!("No routes detected.");

        let mut json_map: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
        json_map.insert("message".to_string(), serde_json::Value::String("No routes detected".to_string()));
        Json(json_map)
    }
}

fn create_api_request(
    body: Option<Json<serde_json::Value>>,
    headers: HeaderMap,
    params: HashMap<String, String>,
    method: Method,
    path: &str,
    request_id: &Uuid,
) -> ApiGatewayProxyRequest {
    debug!("Creating API Gateway request");

    let mut header_map = headers;
    header_map.insert(
        HeaderName::from_static("x-forwarded-proto"),
        HeaderValue::from_static("http"),
    );

    let request_context = create_api_request_context(path, request_id, &method, &header_map);
    let api_request = ApiGatewayProxyRequest {
        body: body.map(|b| b.0.to_string()),
        headers: header_map,
        http_method: method,
        is_base64_encoded: false,
        multi_value_headers: Default::default(),
        multi_value_query_string_parameters: Default::default(),
        // path: Some(path.to_owned()),
        path: Some(path.to_owned()),
        path_parameters: vec![path.to_owned()]
            .iter()
            .map(|path| ("path".to_owned(), path.to_owned()))
            .collect(),
        query_string_parameters: params.into(),
        request_context,
        resource: Some("/{path+}".to_string()),
        stage_variables: Default::default(),
    };

    trace!("API Data: {:#?}", api_request);

    api_request
}

fn create_api_request_context(
    path: &str,
    request_id: &Uuid,
    method: &Method,
    headers: &HeaderMap,
) -> ApiGatewayProxyRequestContext {
    let dt = chrono::Local::now();
    let request_context: ApiGatewayProxyRequestContext = ApiGatewayProxyRequestContext {
        account_id: Some("123456789012".to_string()),
        apiid: Some("1234567890".to_owned()),
        resource_id: Some("123456".to_string()),
        // resource_path: Some(path.to_string()),
        resource_path: Some("/{path+}".to_owned()),
        path: Some(format!("/Prod{}", path).to_owned()),
        stage: Some("Prod".to_string()), 
        domain_name: Some(headers.get("host").unwrap().to_str().unwrap().to_string()),
        domain_prefix: Some(headers.get("host").unwrap().to_str().unwrap().to_string()),
        request_id: Some(request_id.to_string()),
        protocol: Some("HTTP".to_string()),
        http_method: method.clone(),
        request_time: Some(dt.to_rfc3339()),
        request_time_epoch: dt.timestamp_millis(),
        operation_name: None,
        identity: create_api_request_identity(headers),
        authorizer: HashMap::new(),
    };

    request_context
}

fn create_api_request_identity(headers: &HeaderMap) -> ApiGatewayRequestIdentity {
    ApiGatewayRequestIdentity {
        access_key: None,
        account_id: None,
        api_key: None,
        api_key_id: None,
        caller: None,
        cognito_authentication_provider: None,
        cognito_authentication_type: None,
        cognito_identity_id: None,
        cognito_identity_pool_id: None,
        source_ip: Some("0.0.0.0".to_string()),
        user: None,
        user_agent: headers
            .get("user-agent")
            .map(|v| v.to_str().unwrap_or("unknown").to_string()),
        user_arn: None,
    }
}
