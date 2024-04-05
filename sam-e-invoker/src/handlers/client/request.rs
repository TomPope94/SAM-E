use crate::{
    data::{
        api::ApiState,
        store::{EventSource, Invocation, RequestType, ResponseType},
    },
    handlers::client::utils::{
        create_api_request, find_matched_lambda, read_invocation_from_store,
        write_invocation_to_store,
    },
};
use aws_lambda_events::encodings;
use axum::{
    debug_handler,
    extract::{Json, Path, Query, State},
    http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode},
    response::{Html, IntoResponse},
};
use std::{collections::HashMap, str};
use tracing::{debug, error, info, trace};

#[debug_handler]
pub async fn request_handler(
    method: Method,
    headers: HeaderMap,
    Path(path): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(api_state): State<ApiState>,
    body: Option<Json<serde_json::Value>>,
) -> impl IntoResponse {
    let api_lambdas = api_state.get_api_lambdas();

    if api_lambdas.is_empty() {
        debug!("No API lambdas detected from SAM template.");
        return "No API lambdas detected from SAM template.".into_response();
    }

    let prepended_path = format!("/{}", path); // Axum doesn't prepend the path with a slash

    let matched_lambda_and_event =
        find_matched_lambda(&api_lambdas, &method.to_string(), &prepended_path);
    if matched_lambda_and_event.is_err() {
        error!("No matching Lambda found");
        return "No matching Lambda found".into_response();
    }

    let (matched_lambda, matched_props) = matched_lambda_and_event.unwrap();
    let matched_api_props = matched_props.get_api_properties().unwrap().to_owned();

    debug!("Event lambda found: {:?}", &matched_lambda);
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
        &matched_api_props.get_base_path(),
        &request_id,
    );
    new_invocation.set_request(RequestType::Api(api_data));

    // Write invocation to store where it'll be picked up by /next endpoint on lambda
    // runtime api
    let _ = write_invocation_to_store(
        new_invocation,
        matched_lambda.get_name(),
        api_state.get_store(),
    );

    let processed_invocation =
        read_invocation_from_store(api_state.get_store(), matched_lambda.get_name(), request_id)
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
            let status_code =
                StatusCode::from_u16(api_response.status_code.try_into().unwrap_or(500))
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

            trace!("Returning response with status code: {:?}", status_code);
            info!("Returning response with headers: {:?}", header_map);
            info!("Headers in response: {:?}", &api_response.headers);

            if let Some(response_body) = &api_response.body {
                trace!("Returning response with body: {:?}", response_body);
                match response_body {
                    encodings::Body::Text(text) => {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(text) {
                            Json(parsed).into_response()
                        } else {
                            if let Some(content_type) = header_map.get("content-type") {
                                let value_string: &str =
                                    str::from_utf8(content_type.as_bytes()).unwrap_or("unknown");
                                if value_string.contains("text/html") {
                                    Html(text.clone()).into_response()
                                } else {
                                    text.clone().into_response()
                                }
                            } else {
                                text.clone().into_response()
                            }
                        }
                    }
                    encodings::Body::Binary(binary) => binary.clone().into_response(),
                    encodings::Body::Empty => "No Response body found".into_response(),
                }
            } else {
                error!("No response body found. Returning empty response.");
                "No Response body found".into_response()
            }
        }
        _ => {
            error!("This response type is not supported. Returning empty response.");
            "This response type is not supported".into_response()
        }
    }
}
