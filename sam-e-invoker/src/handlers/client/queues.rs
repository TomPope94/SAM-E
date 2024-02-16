use aws_config::{profile::ProfileFileCredentialsProvider, BehaviorVersion};
use aws_sdk_sqs::{config::Region, Client};
use sam_e_types::config::{
    infrastructure::{Infrastructure, InfrastructureType},
    Config,
};
use tokio::time::{sleep, Duration};
use tracing::{debug, error, trace, warn};

pub async fn listen_to_queues(config: &Config) {
    let queues = get_queues_from_config(config);

    for mut queue in queues {
        if !check_queue_exists(queue.get_name()).await {
            debug!("Queue doesn't exist, creating: {}", queue.get_name());
            if let Ok(queue_url) = create_queue(&queue).await {
                debug!("Created queue: {}", queue_url);
                queue.set_queue_url(queue_url);
            } else {
                warn!("Failed to create queue: {}", queue.get_name());
            }
        } else {
            debug!("Queue exists: {}", queue.get_name());
        }

        tokio::spawn(async move {
            debug!("SQS Client to poll queue: {}", queue.get_name());
            poll_queue(&queue).await;
        });
    }
}

pub async fn poll_queue(queue: &Infrastructure) {
    debug!("Polling queue: {}", queue.get_name());
    let client = get_aws_client().await;

    loop {
        // Only check every half second to avoid lock contention
        sleep(Duration::from_millis(500)).await;
        if let Some(url) = queue.get_queue_url() {
            let receive_message = client
                .receive_message()
                .queue_url(url)
                .max_number_of_messages(10) // TODO: this should be configured
                .send()
                .await;

            match receive_message {
                Ok(response) => {
                    if let Some(messages) = response.messages {
                        for message in messages {
                            debug!("Message: {:?}", message);
                        }
                    }
                }
                Err(e) => {
                    debug!("Failed to receive messages: {}", e);
                }
            }
        } else {
            error!("Queue URL not set for queue: {}", queue.get_name());
        }
    }
}

fn get_queues_from_config(config: &Config) -> Vec<Infrastructure> {
    let queues: Vec<Infrastructure> = config
        .get_infrastructure()
        .clone()
        .into_iter()
        .filter(|i| i.to_owned().get_infrastructure_type().clone() == InfrastructureType::Sqs)
        .collect();

    debug!("Found {} SQS queues from config", queues.len());
    trace!("Queues: {:?}", queues);

    queues
}

/// A function to check if the queue exists. In the event that it isn't it's passed onto another
/// process in charge of creating the queue. Note: this is only necessary while the queue .conf
/// file can't be passed as docker volume (within VM setup)
async fn check_queue_exists(queue_name: &str) -> bool {
    debug!("Checking if queue exists: {}", queue_name);
    let client = get_aws_client().await;

    let check_queue = client.get_queue_url().queue_name(queue_name).send().await;
    debug!("Queue exists: {:?}", check_queue);

    match check_queue {
        Ok(_) => true,
        Err(e) => {
            error!("Failed to check if queue exists: {}", e);
            false
        }
    }
}

async fn create_queue(queue: &Infrastructure) -> anyhow::Result<String> {
    let queue_name = queue.get_name();
    debug!("Creating queue: {}", queue_name);
    let client = get_aws_client().await;
    debug!("Client created: {:?}", client);

    let created_queue = client.create_queue().queue_name(queue_name).send().await;

    debug!("Queue created: {:?}", created_queue);

    match &created_queue {
        Ok(_) => debug!("Queue created: {}", queue.get_name()),
        Err(e) => error!("Failed to create queue: {}", e),
    }

    // Need to wait at least 1 second after queue has been created before using
    sleep(Duration::from_secs(1)).await;

    Ok(created_queue.unwrap().queue_url.unwrap())
}

async fn get_aws_client() -> aws_sdk_sqs::Client {
    trace!("Creating AWS SQS client");
    let region = Region::new("eu-west-2");

    let profile_provider = ProfileFileCredentialsProvider::builder()
        .profile_name("staging-mfa")
        .build();

    let config = aws_config::defaults(BehaviorVersion::v2023_11_09())
        .region(region)
        .credentials_provider(profile_provider)
        .endpoint_url("http://sqs-local:9324")
        .load()
        .await;

    Client::new(&config)
}
