use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use aws_sdk_sqs::{config::Region, Client};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Triggers {
    #[serde(skip_serializing_if = "Option::is_none")]
    lambdas: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    queues: Option<Vec<String>>,
}

impl Triggers {
    pub fn new(lambdas: Option<Vec<String>>, queues: Option<Vec<String>>) -> Self {
        Self { lambdas, queues }
    }

    pub fn get_lambdas(&self) -> &Option<Vec<String>> {
        &self.lambdas
    }

    pub fn add_lambda(&mut self, lambda: String) {
        if let Some(lambdas) = &mut self.lambdas {
            lambdas.push(lambda);
        } else {
            self.lambdas = Some(vec![lambda]);
        }
    }

    pub fn set_lambdas(&mut self, lambdas: Vec<String>) {
        self.lambdas = Some(lambdas);
    }

    pub fn get_queues(&self) -> &Option<Vec<String>> {
        &self.queues
    }

    pub fn add_queue(&mut self, queue: String) {
        if let Some(queues) = &mut self.queues {
            queues.push(queue);
        } else {
            self.queues = Some(vec![queue]);
        }
    }

    pub fn set_queues(&mut self, queues: Vec<String>) {
        self.queues = Some(queues);
    }

    pub async fn send(&self, event: String) -> Result<()> {
        if let Some(lambdas) = &self.lambdas {
            for lambda in lambdas {
                debug!("Sending event {} to lambda {}", event, lambda);
            }
        }

        if let Some(queues) = &self.queues {
            debug!("Creating AWS SQS client");
            let region = Region::new("eu-west-1");

            // let config = aws_config::defaults(BehaviorVersion::v2023_11_09())
            let config = aws_config::from_env()
                .region(region)
                .endpoint_url("http://sqs-local:9324")
                .load()
                .await;

            let client = Client::new(&config);

            for queue in queues {
                debug!("Sending event {} to queue {}", event, queue);
                let queue_url = client.get_queue_url().queue_name(queue).send().await;

                let Ok(queue_url_output) = queue_url else {
                    error!("Error getting queue URL for queue: {}", queue);
                    return Err(anyhow!("Error getting queue URL for queue: {}", queue));
                };

                let Some(queue_url) = queue_url_output.queue_url else {
                    error!("Queue URL not set for queue: {}", queue);
                    return Err(anyhow!("Queue URL not set for queue: {}", queue));
                };

                client
                    .send_message()
                    .queue_url(queue_url)
                    .message_body(&event)
                    .send()
                    .await?;

                debug!("Event sent to queue successfully");
            }
        }

        Ok(())
    }
}
