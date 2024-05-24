use anyhow::{anyhow, Result};
use aws_sdk_sqs::{config::Region, Client};
use tracing::{debug, error, info};

pub async fn dequeue_mesesage(queue_name: &str, receipt_handle: &Option<String>) -> Result<()> {
    info!("Dequeueing email from SQS");
    debug!("First parsing receipt handle");
    let Some(receipt_handle) = receipt_handle else {
       error!("No receipt handle found");
       return Err(anyhow!("No receipt handle found"));
    };

    debug!("Creating AWS SQS client");
    let region = Region::new("eu-west-1");

    // let config = aws_config::defaults(BehaviorVersion::v2023_11_09())
    let config = aws_config::from_env()
        .region(region)
        .endpoint_url("http://sqs-local:9324")
        .load()
        .await;

    let client = Client::new(&config);

    let queue_url_output = client.get_queue_url()
        .queue_name(queue_name)
        .send()
        .await?;
    let queue_url = match queue_url_output.queue_url {
        Some(url) => url,
        None => {
            return Err(anyhow!("No queue URL found"));
        }
    };

    let delete_message_output = client.delete_message()
        .queue_url(queue_url)
        .receipt_handle(receipt_handle)
        .send()
        .await?;
    debug!("Message deleted: {:?}", receipt_handle);
    Ok(())
}
