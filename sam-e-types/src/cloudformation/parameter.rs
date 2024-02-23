use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Parameter {
    #[serde(rename = "Type")]
    param_type: String,
    description: Option<String>,
    default: Option<String>,
    allowed_values: Option<Vec<String>>,
    allowed_pattern: Option<String>,
    max_length: Option<i64>,
    min_length: Option<i64>,
}
