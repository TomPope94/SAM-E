pub mod event;
pub mod function;
pub mod apigw;
pub mod base_path_mapping;
pub mod db_instance;
pub mod queue;
pub mod bucket;

pub use event::Event;
pub use function::Function;
pub use apigw::ApiGateway;
pub use base_path_mapping::BasePathMapping;
pub use db_instance::DbInstance;
pub use queue::Queue;
pub use bucket::Bucket;

use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum ResourceType {
    #[serde(rename = "AWS::Serverless::Function")]
    Function,
    #[serde(rename = "AWS::Serverless::Api")]
    ApiGateway,
    #[serde(rename = "AWS::ApiGateway::BasePathMapping")]
    BasePathMapping,
    #[serde(rename = "AWS::RDS::DBInstance")]
    DbInstance,
    #[serde(rename = "AWS::SQS::Queue")]
    Queue,
    #[serde(rename = "AWS::S3::Bucket")]
    Bucket,
    #[serde(untagged)]
    Other(serde_yaml::Value),
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Resource {
    #[serde(rename = "Type")]
    pub resource_type: ResourceType,
    pub properties: serde_yaml::Value
}
