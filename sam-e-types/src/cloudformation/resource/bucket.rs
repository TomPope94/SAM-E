use serde::Deserialize;
use crate::cloudformation::template::CloudFormationValue as Value;
// use serde_yaml::Value;

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Bucket {
    bucket_name: Value,
    notification_configuration: Option<NotificationConfiguration>,
}

impl Bucket {
    pub fn get_bucket_name(&self) -> &Value {
        &self.bucket_name
    }

    pub fn get_notification_configuration(&self) -> &Option<NotificationConfiguration> {
        &self.notification_configuration
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct QueueConfigurations {
    event: Value,
    queue: Value,
}

impl QueueConfigurations {
    pub fn get_event(&self) -> &Value {
        &self.event
    }

    pub fn get_queue(&self) -> &Value {
        &self.queue
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct LambdaConfigurations {
    event: Value,
    function: Value,
}

impl LambdaConfigurations {
    pub fn get_event(&self) -> &Value {
        &self.event
    }

    pub fn get_function(&self) -> &Value {
        &self.function
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct NotificationConfiguration {
    queue_configurations: Option<Vec<QueueConfigurations>>,
    lambda_configurations: Option<Vec<Value>>,
}

impl NotificationConfiguration {
    pub fn get_queue_configurations(&self) -> &Option<Vec<QueueConfigurations>> {
        &self.queue_configurations
    }

    pub fn get_lambda_configurations(&self) -> &Option<Vec<Value>> {
        &self.lambda_configurations
    }
}
