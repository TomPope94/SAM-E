use serde::Deserialize;
use crate::cloudformation::resource::{
    Function,
    ApiGateway,
    BasePathMapping,
    DbInstance,
    Queue,
    Bucket
};

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
#[serde(tag = "Type")]
pub enum Resource {
    #[serde(rename = "AWS::Serverless::Api")]
    ApiGateway(ResourceContainer<ApiGateway>),
    #[serde(rename = "AWS::ApiGateway::BasePathMapping")]
    BasePathMapping(ResourceContainer<BasePathMapping>),
    #[serde(rename = "AWS::Serverless::Function")]
    Function(ResourceContainer<Function>),
    #[serde(rename = "AWS::RDS::DBInstance")]
    RDSInstance(ResourceContainer<DbInstance>),
    #[serde(rename = "AWS::SQS::Queue")]
    SQSQueue(ResourceContainer<Queue>),
    #[serde(rename = "AWS::S3::Bucket")]
    S3Bucket(ResourceContainer<Bucket>),
    #[serde(untagged)]
    Other(serde_yaml::Value), // A catch all for unsupported resources
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct ResourceContainer<T> {
    properties: T,
}

impl <T> ResourceContainer<T> {
    pub fn get_properties(&self) -> &T {
        &self.properties
    }
}
