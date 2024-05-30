use std::collections::HashMap;

use aws_lambda_events::sqs::{SqsEvent, SqsMessage};
use aws_sdk_sqs::Client;
use sam_e_types::{
    config::{
        infrastructure::{sqs::QueueInfrastructure, Infrastructure},
        Config,
    },
    invocation::{EventRequest, InvocationBuilder},
};
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, trace, warn};

use crate::data::QueueState;

pub async fn listen_to_queues(config: Config, queue_state: QueueState) {
    let config_infrastructure = config.get_infrastructure();
    // let queues = get_queues_from_config(&config);

    for infrastructure in config_infrastructure {
        match infrastructure {
            Infrastructure::Sqs(sqs_data) => {
                debug!("Found SQS infrastructure");
                let mut props = sqs_data.properties.clone();

                if !check_queue_exists(&props.name, queue_state.get_queue_client()).await {
                    debug!("Queue doesn't exist, creating: {}", props.name);
                    if let Ok(queue_url) =
                        create_queue(&props, queue_state.get_queue_client()).await
                    {
                        debug!("Created queue: {}", queue_url);
                        props.queue_url = Some(queue_url);
                    } else {
                        warn!("Failed to create queue: {}", props.name);
                    }
                } else {
                    debug!("Queue exists: {}", props.name);
                }

                debug!("SQS Client to poll queue: {}", props.name);
                let _ = poll_queue_for_invoke(props, queue_state.clone()).await;
            }
            _ => {
                trace!("Skipping infrastructure type");
                continue;
            }
        }
    }

    loop {
        trace!("Sleeping for 1 second before staying awake...");
        sleep(Duration::from_secs(1)).await;
    }
}

pub async fn poll_queue_for_invoke(queue: QueueInfrastructure, queue_state: QueueState) {
    debug!("Polling queue: {}", queue.name);

    tokio::task::spawn(async move {
        let client = queue_state.get_queue_client().clone();
        loop {
            // Only check every half second to avoid lock contention
            sleep(Duration::from_millis(500)).await;

            let Some(url) = &queue.queue_url else {
                error!("Queue URL not set for queue: {}", queue.name);
                continue;
            };

            let receive_message_res = &client
                .receive_message()
                .queue_url(url)
                .max_number_of_messages(10) // TODO: this should be configured
                .send()
                .await;

            match receive_message_res {
                Ok(message_output) => {
                    let Some(messages) = &message_output.messages else {
                        trace!("No messages found in queue: {}", queue.name);
                        continue;
                    };
                    let formatted_messages = messages
                        .iter()
                        .map(|m| SqsMessage {
                            message_id: m.message_id.clone(),
                            receipt_handle: m.receipt_handle.clone(),
                            body: m.body.clone(),
                            attributes: HashMap::new(),
                            md5_of_body: m.md5_of_body.clone(),
                            event_source: Some(queue.name.clone()),
                            aws_region: Some("eu-west-2".to_string()),
                            event_source_arn: None,
                            md5_of_message_attributes: None,
                            message_attributes: HashMap::new(),
                        })
                        .collect::<Vec<SqsMessage>>();

                    if formatted_messages.len() > 0 {
                        debug!(
                            "Found {} messages in queue: {}",
                            formatted_messages.len(),
                            queue.name
                        );
                        trace!("Messages: {:#?}", formatted_messages);

                        if let Some(triggers) = &queue.triggers {
                            let lambda_triggers = triggers.get_lambdas();

                            if let Some(lambda_triggers) = lambda_triggers {
                                debug!("Detected lambda triggers for queue: {:?}", lambda_triggers);
                                for lambda in lambda_triggers {
                                    debug!("Adding SQS invocation for container: {}", lambda);
                                    let new_invocation = InvocationBuilder::new()
                                        .with_request(EventRequest::Sqs(SqsEvent {
                                            records: formatted_messages.clone(),
                                        }))
                                        .with_lambda_name(lambda.clone())
                                        .build();

                                    if let Ok(invocation) = new_invocation {
                                        debug!(
                                            "Invocation created successfully. Now adding to store"
                                        );
                                        let client = queue_state.get_request_client();
                                        let response = client
                                            .post("http://0.0.0.0:3030/invoke")
                                            .json(&serde_json::json!(invocation))
                                            .send()
                                            .await;

                                        match response {
                                            Ok(_) => debug!("Successfully invoked lambda"),
                                            Err(e) => error!("Failed to invoke lambda: {}", e),
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    debug!("Failed to receive messages: {}", e);
                    continue;
                }
            }
        }
    });
}

/// A function to check if the queue exists. In the event that it isn't it's passed onto another
/// process in charge of creating the queue. Note: this is only necessary while the queue .conf
/// file can't be passed as docker volume (within VM setup)
async fn check_queue_exists(queue_name: &str, client: &Client) -> bool {
    debug!("Checking if queue exists: {}", queue_name);

    let check_queue = client.get_queue_url().queue_name(queue_name).send().await;
    debug!("Check queue result: {:?}", check_queue);

    match check_queue {
        Ok(_) => true,
        Err(_e) => {
            warn!("Queue not found. Will create before polling");
            false
        }
    }
}

async fn create_queue(queue: &QueueInfrastructure, client: &Client) -> anyhow::Result<String> {
    let queue_name = &queue.name;
    debug!("Creating queue: {}", queue_name);

    let created_queue = client.create_queue().queue_name(queue_name).send().await;

    info!("Queue created: {:?}", created_queue);

    match &created_queue {
        Ok(_) => debug!("Queue created: {}", queue.name),
        Err(e) => error!("Failed to create queue: {}", e),
    }

    // Need to wait at least 1 second after queue has been created before using
    sleep(Duration::from_secs(1)).await;

    Ok(created_queue.unwrap().queue_url.unwrap())
}
