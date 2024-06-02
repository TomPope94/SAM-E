mod handlers;

use aws_lambda_events::event::sqs::SqsEventObj;
use lambda_runtime::{LambdaEvent, Error};
use tracing::{debug, info};

pub async fn msg_handler(event: LambdaEvent<SqsEventObj<serde_json::Value>>) -> Result<(), Error> {
    info!("Received event: {:?}", event);

    for record in event.payload.records {
        debug!("Processing record: {:?}", record);

        handlers::dequeue_mesesage("TestDeliverySQS", &record.receipt_handle).await?;
    }

    Ok(())
}

