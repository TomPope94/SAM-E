use axum::{
    response::Json,
    routing::post,
    Router,
};
use lambda_http::{
    run, 
    Error,
};
use serde_json::{json, Value};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

use aws_sdk_eventbridge::{config::Region, Client, types::builders::PutEventsRequestEntryBuilder};

#[derive(Debug, Serialize, Deserialize)]
struct EventBridgeTest {
    name: String,
    test_value: serde_json::Value,
}

async fn post_foo() -> Json<Value> {
    debug!("Creating AWS Eventbridge client");

    let region = Region::new("eu-west-1");
    let config = aws_config::from_env()
        .region(region)
        .endpoint_url("http://sam-e-invoker:3002")
        .load()
        .await;

    let client = Client::new(&config);

    debug!("Client created successfully");

    let test_event = EventBridgeTest {
        name: "test".to_string(),
        test_value: serde_json::json!({"test": "value"}),
    };

    let test_event_value = serde_json::to_value(&test_event).unwrap();
    let test_event_str = serde_json::to_string(&test_event_value).unwrap();
    debug!("test_event_str: {}", test_event_str);

    let test_event_str_from_struct = serde_json::to_string(&test_event).unwrap();
    debug!("test_event_str_from_struct: {}", test_event_str_from_struct);

    debug!("Creating event entry...");
    let event_entry = PutEventsRequestEntryBuilder::default()
        .detail(test_event_str)
        .detail_type("test.event")
        .source("service.my_micro_service")
        .event_bus_name("TestEventBus")
        .build();
    debug!("Event entry created successfully");
    debug!("Event entry: {:#?}", event_entry);

    debug!("Sending event...");
    let res = client.put_events()
        .entries(event_entry)
        .send()
        .await;

    match res {
        Ok(_) => {
            debug!("Event sent successfully");
            return Json(json!({ "msg": "Events sent successfully" }));
        }
        Err(e) => {
            debug!("Error sending event: {:?}", e);
            return Json(json!({ "msg": "Error sending event" }));
        }
    }
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

    info!("Starting Lambda");

    let app = Router::new()
        .route("/Prod/eventbridge/foo", post(post_foo));

    run(app).await
}

