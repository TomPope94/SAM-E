use sam_e_types::config::{lambda::Lambda, Config};

use aws_sdk_sqs::{Client as QueueClient, config::Region};

use reqwest::Client as RequestClient;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct QueueState {
    pub lambdas: Vec<Lambda>,
    pub request_client: RequestClient,
    pub queue_client: QueueClient,
}

impl QueueState {
    pub async fn from_config(config: &Config) -> Self {
        debug!("Creating reqwest client");
        let request_client = RequestClient::new();
        debug!("Reqwest client created");

        let queue_client = create_sqs_client().await;
        debug!("Queue client created");

        Self {
            lambdas: config.get_lambdas().to_owned(),
            request_client,
            queue_client,
        }
    }

    pub fn _get_queue_lambdas(&self) -> Vec<&Lambda> {
        self.lambdas
            .iter()
            .filter(|l| {
                l.get_events()
                    .into_iter()
                    .any(|e| e.get_sqs_properties().is_some())
            })
            .collect()
    }

    pub fn get_request_client(&self) -> &RequestClient {
        &self.request_client
    }

    pub fn get_queue_client(&self) -> &QueueClient {
        &self.queue_client
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
