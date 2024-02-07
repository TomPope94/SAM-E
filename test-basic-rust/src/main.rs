use lambda_http::{service_fn, Error, IntoResponse, Request, RequestExt, Response};
use aws_lambda_events::event::apigw::ApiGatewayProxyResponse;
use tracing::info;

async fn handler(event: Request) -> Result<impl IntoResponse, Error> {
    let resp: lambda_http::Response<String> = Response::builder()
        .status(200)
        .header("content-type", "application/json")
        // .header("content-type", "text/html")
        // .body("Hello AWS Lambda HTTP request".into())
        .body(serde_json::json!({
            "message": "Hello AWS Lambda HTTP request",
        }).to_string().into())
        .map_err(Box::new)?;

    info!("Response: {:#?}", resp);

    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        // disable printing the name of the module in every log line.
        // .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();
    lambda_http::run(service_fn(handler)).await
}
