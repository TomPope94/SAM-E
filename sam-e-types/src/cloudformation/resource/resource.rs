use serde::Deserialize;

// #[derive(Deserialize, Debug, PartialEq, Eq)]
// #[serde(rename_all = "PascalCase")]
// #[serde(tag = "Type")]
// pub enum Resource {
//     #[serde(rename = "AWS::Serverless::Api")]
//     ApiGateway(ResourceContainer<ApiGateway>),
//     #[serde(rename = "AWS::ApiGateway::BasePathMapping")]
//     BasePathMapping(ResourceContainer<BasePathMapping>),
//     #[serde(rename = "AWS::Serverless::Function")]
//     Function(ResourceContainer<Function>),
//     #[serde(rename = "AWS::RDS::DBInstance")]
//     RDSInstance(ResourceContainer<DbInstance>),
//     #[serde(rename = "AWS::SQS::Queue")]
//     SQSQueue(ResourceContainer<Queue>),
//     #[serde(rename = "AWS::S3::Bucket")]
//     S3Bucket(ResourceContainer<Bucket>),
//     #[serde(untagged)]
//     Other(serde_yaml::Value), // A catch all for unsupported resources
// }
//
// #[derive(Deserialize, Debug, PartialEq, Eq)]
// #[serde(rename_all = "PascalCase")]
// pub struct ResourceContainer<T> {
//     properties: T,
// }

// impl <T> ResourceContainer<T> {
//     pub fn get_properties(&self) -> &T {
//         &self.properties
//     }
// }
//

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
