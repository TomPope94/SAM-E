mod handlers;

use aws_lambda_events::event::{
    sqs::SqsEventObj,
    eventbridge::EventBridgeEvent,
};
use lambda_runtime::{LambdaEvent, Error};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tracing::{debug, info};

#[derive(Debug, Serialize, Deserialize)]
pub struct EventBridgeTest {
    name: String,
    test_value: serde_json::Value,
}

pub async fn msg_handler(event: LambdaEvent<SqsEventObj<EventBridgeEvent<EventBridgeTest>>>) -> Result<(), Error> {
    info!("Received event: {:?}", event);

    for record in event.payload.records {
        debug!("Processing record: {:?}", record);

        handlers::dequeue_mesesage("TestDeliverySQS", &record.receipt_handle).await?;
    }

    Ok(())
}

