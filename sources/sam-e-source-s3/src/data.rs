use sam_e_types::config::{infrastructure::Infrastructure, Config};

use aws_sdk_sqs::{config::Region, Client as QueueClient};

use reqwest::Client as RequestClient;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct ApiState {
    pub infrastructure: Vec<Infrastructure>,
    pub request_client: RequestClient,
    pub queue_client: QueueClient,
}

impl ApiState {
    pub async fn from_config(config: &Config) -> Self {
        debug!("Creating API state from config...");
        debug!("Creating Reqwest Client...");
        let request_client = RequestClient::new();
        debug!("Reqwest client created");

        debug!("Creating SQS Client...");
        let queue_client = create_sqs_client().await;
        debug!("Queue client created");

        Self {
            infrastructure: config.get_infrastructure().to_owned(),
            request_client,
            queue_client,
        }
    }

    pub fn _get_request_client(&self) -> &RequestClient {
        &self.request_client
    }

    pub fn get_queue_client(&self) -> &QueueClient {
        &self.queue_client
    }

    pub fn get_infrastructure(&self) -> &Vec<Infrastructure> {
        &self.infrastructure
    }
}

async fn create_sqs_client() -> QueueClient {
    debug!("Creating AWS SQS client");
    let region = Region::new("eu-west-1");

    // let config = aws_config::defaults(BehaviorVersion::v2023_11_09())
    let config = aws_config::from_env()
        .region(region)
        .endpoint_url("http://sqs-local:9324")
        .load()
        .await;

    QueueClient::new(&config)
}
