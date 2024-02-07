use aws_lambda_events::apigw::ApiGatewayProxyResponse;
use axum::{
    body::{to_bytes, Body, Bytes},
    http::{header, Method, Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use tower_http::cors::{Any, CorsLayer};
use tracing::trace;

pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
}

// Adds headers to the request if they're in body and not in request itself - Something that
// happens with the Rust Lambda runtime
pub async fn headers_mw(req: Request<Body>, next: Next) -> impl IntoResponse {
    trace!("Request = {:#?}", req);

    let (mut parts, body) = req.into_parts();
    let body_bytes = body_to_bytes("middleware request", body).await.unwrap();

    if let Ok(body_value) = serde_json::from_slice::<ApiGatewayProxyResponse>(&body_bytes) {
        trace!("Body value: {:#?}", body_value);

        let headers = body_value.headers;
        headers.iter().for_each(|(key, value)| {
            let check_header = parts.headers.get(key);
            if check_header.is_none() {
                parts.headers.insert(key.clone(), value.clone());
            }
        });
    }

    trace!("Parts = {:#?}", parts);
    let req = Request::from_parts(parts, Body::from(body_bytes));

    next.run(req).await
}

async fn body_to_bytes(
    direction: &str,
    body: axum::body::Body,
) -> Result<Bytes, (StatusCode, String)> {
    let bytes = match to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {} body: {}", direction, err),
            ));
        }
    };

    Ok(bytes)
}
