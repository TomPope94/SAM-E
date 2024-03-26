use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
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

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Resource {
    #[serde(rename = "Type")]
    pub resource_type: ResourceType,
    pub properties: serde_yaml::Value
}
