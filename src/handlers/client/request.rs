use crate::data::{
    api::ApiState,
    sam::Route,
    store::{Invocation, InvocationQueue},
};
use aws_lambda_events::apigw::ApiGatewayV2httpRequest;
use axum::{
    extract::{Json, Path, Query, State},
    http::{HeaderMap, Method},
};
use std::collections::HashMap;
use tracing::{debug, info, trace};
use uuid::Uuid;

pub async fn request_handler(
    method: Method,
    headers: HeaderMap,
    Path(path): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(api_state): State<ApiState>,
    Json(body): Json<serde_json::Value>,
) -> String {
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

            let api_data = create_api_request(body, headers, params, method, &path);

            let new_invocation: Invocation<ApiGatewayV2httpRequest> = Invocation::new(api_data);

            let invocation_id = send_invocation_to_store(new_invocation, matched_route, api_state);
            invocation_id.to_string()

            // TODO: pass UUID to wait for invocation to be processed
            // TODO: return invocation result
        } else {
            debug!("No route match found");

            let res = format!("No route match found for path: {}", path);
            res
        }
    } else {
        debug!("No routes detected.");

        let res = format!("No SAM routes detected");
        res
    }
}

fn create_api_request(
    body: serde_json::Value,
    headers: HeaderMap,
    params: HashMap<String, String>,
    method: Method,
    path: &str,
) -> ApiGatewayV2httpRequest {
    debug!("Creating API Gateway request");
    let mut api_data = ApiGatewayV2httpRequest::default();
    api_data.body = Some(body.to_string());
    api_data.headers = headers;
    api_data.query_string_parameters = params.into();
    api_data.http_method = method;
    api_data.raw_path = Some(path.to_string());

    trace!("API Data: {:#?}", api_data);

    api_data
}

fn send_invocation_to_store(
    invocation: Invocation<ApiGatewayV2httpRequest>,
    route: Route,
    api_state: ApiState,
) -> Uuid {
    let write_queue = InvocationQueue::new();
    let store = api_state.get_store();

    store
        .get_queues()
        .write()
        .entry(route.container_name.to_owned())
        .or_insert(write_queue)
        .api_invocations
        .push(invocation.to_owned());

    invocation.get_uuid().to_owned()
}
