use crate::data::ApiState;

use aws_lambda_events::s3::S3Event;
use aws_sdk_sqs::Client;
use axum::{
    extract::{Json, State},
    response::IntoResponse,
};
use sam_e_types::config::infrastructure::{Infrastructure, InfrastructureType};
use tracing::{debug, error, info, trace, warn};

pub async fn handler(State(api_state): State<ApiState>, body: Json<S3Event>) -> impl IntoResponse {
    info!("Received a request to the S3 webhook");
    info!("Request body: {:#?}", body);

    let s3_event = body.0;

    // Match S3 event to bucket in infrastructure
    let bucket_name = s3_event.records[0].s3.bucket.name.clone();
    let infrastructure = api_state.get_infrastructure();

    if let Some(bucket) = bucket_name {
        let s3_buckets: Vec<&Infrastructure> = infrastructure
            .iter()
            .filter(|i| i.get_infrastructure_type() == &InfrastructureType::S3)
            .collect();

        for i in s3_buckets.into_iter() {
            if i.get_name() == bucket {
                info!("Found infrastructure for bucket: {}", bucket);

                let triggers = i.get_triggers();
                if let Some(triggers) = triggers {
                    trace!("Triggers: {:#?}", triggers);

                    if let Some(queues) = triggers.get_queues() {
                        for queue in queues {
                            debug!("Detected queue trigger for: {}", queue);
                            handle_queue_trigger(
                                queue.as_str(),
                                &s3_event,
                                api_state.get_queue_client(),
                            )
                            .await;
                        }
                    }

                    if let Some(lambdas) = triggers.get_lambdas() {
                        for lambda in lambdas {
                            debug!("Detected lambda trigger for: {}", lambda);
                            warn!("Currently unsupported!");
                        }
                    }
                }
            }
        }
    }

    "s3"
}

async fn handle_queue_trigger(queue: &str, s3_event: &S3Event, client: &Client) {
    debug!("Handling queue trigger: {}", queue);

    // Send message to queue
    let queue_url = client.get_queue_url().queue_name(queue).send().await;

    match queue_url {
        Ok(queue_url) => {
            let queue_url = queue_url.queue_url.unwrap();
            info!("Queue URL: {}", queue_url);

            let message = format!("{:#?}", s3_event);
            let send_message = client
                .send_message()
                .queue_url(queue_url)
                .message_body(message)
                .send()
                .await;

            match send_message {
                Ok(_) => {
                    debug!("Successfully sent message to SQS queue");
                }
                Err(e) => {
                    error!("Failed to send message to SQS queue: {}", e);
                }
            }
        }
        Err(e) => {
            error!("Failed to get queue URL: {}", e);
        }
    }
}
