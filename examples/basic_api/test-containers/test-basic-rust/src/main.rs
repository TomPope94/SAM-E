use lambda_http::{service_fn, Error, IntoResponse, Request, Response};
use tracing::info;
use tracing_subscriber::EnvFilter;

async fn handler(_event: Request) -> Result<impl IntoResponse, Error> {
    let resp: lambda_http::Response<String> = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body("Hello AWS Lambda HTTP request".into())
        .map_err(Box::new)?;

    info!("Response: {:#?}", resp);

    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        // disable printing the name of the module in every log line.
        // .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();
    lambda_http::run(service_fn(handler)).await
}
