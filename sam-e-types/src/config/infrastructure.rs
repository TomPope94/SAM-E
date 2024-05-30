pub mod event_bus;
pub mod mysql;
pub mod postgres;
pub mod s3;
pub mod sqs;
pub mod triggers;
pub mod event_rule;

pub use event_bus::{EventBusInfrastructure, EventBusBuilder};
pub use event_rule::{EventRuleInfrastructure, EventRuleBuilder};
pub use mysql::{MysqlInfrastructure, MysqlBuilder};
pub use postgres::{PostgresInfrastructure, PostgresBuilder};
pub use s3::{S3Infrastructure, S3Builder};
pub use sqs::{QueueInfrastructure, QueueBuilder};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(tag = "infrastructure_type")]
pub enum Infrastructure {
    #[serde(rename = "SQS")]
    Sqs(ResourceContainer<QueueInfrastructure>),
    #[serde(rename = "Postgres")]
    Postgres(ResourceContainer<PostgresInfrastructure>),
    #[serde(rename = "MySQL")]
    Mysql(ResourceContainer<MysqlInfrastructure>),
    #[serde(rename = "S3")]
    S3(ResourceContainer<S3Infrastructure>),
    #[serde(rename = "EventBus")]
    EventBus(ResourceContainer<EventBusInfrastructure>),
    #[serde(rename = "EventRule")]
    EventRule(ResourceContainer<EventRuleInfrastructure>),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct ResourceContainer<T> {
    pub properties: T,
}

impl<T> ResourceContainer<T> {
    pub fn new(properties: T) -> Self {
        Self { properties }
    }
}
