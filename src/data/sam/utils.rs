use fancy_regex::Regex;
use serde_yaml::Value;
use std::collections::HashMap;
use anyhow::{anyhow, Result};

use crate::data::sam::Route;

pub fn create_sam_routes(
    sam_template: &str,
) -> Result<HashMap<String, Route>> { 
    let sam_value: Value = serde_yaml::from_str(&sam_template).expect("invalid SAM template");

    let sam_resources = serde_yaml::from_value(sam_value["Resources"].to_owned());

    if let Ok(sam_resources) = sam_resources {
        Ok(create_sam_routes_from_resources(sam_resources))
    } else {
        Err(anyhow!("no SAM Resources in template"))
    }
}

fn create_sam_routes_from_resources(
    sam_resources: HashMap<String, Value>,
) -> HashMap<String, Route> {
    sam_resources
        .iter()
        .filter(|(_, value)| value["Type"] == "AWS::Serverless::Function")
        .fold(HashMap::new(), |mut acc, (_, value)| {
            let sam_events: HashMap<String, Value> =
                serde_yaml::from_value(value["Properties"]["Events"].to_owned())
                    .expect("no SAM Events in template");

            let sam_container_name = value["Properties"]["ImageUri"]
                .as_str()
                .expect("no SAM ImageUri in template");

            sam_events
                .iter()
                .filter(|(_, event_data)| event_data["Type"] == "Api")
                .for_each(|(_, event_data)| {
                    let sam_path = event_data["Properties"]["Path"]
                        .as_str()
                        .expect("no SAM Path in template");

                    let base_path = get_base_path(event_data, &sam_resources);

                    let final_path = trim_path_ending_slash(format!("{}{}", base_path, sam_path));

                    let sam_method = event_data["Properties"]["Method"]
                        .as_str()
                        .expect("no SAM Method in template");

                    let regex_replaced_path = replaced_regex_path(&final_path);

                    let sam_route_regex =
                        Regex::new(&regex_replaced_path).expect("invalid SAM path");
                    let sam_route = Route::create(
                        regex_replaced_path,
                        sam_method.to_owned(),
                        sam_container_name.to_owned(),
                        sam_route_regex,
                    );
                    acc.insert(format!("{}::{}", final_path, sam_method), sam_route);
                });
            acc
        })
}

fn get_base_path(
    event_data: &serde_yaml::Value,
    sam_resources: &HashMap<String, serde_yaml::Value>,
) -> String {
    let sam_event_api_gateway = &event_data["Properties"]["RestApiId"];
    if sam_event_api_gateway.is_string() {
        let sam_event_api_gateway = sam_event_api_gateway.as_str().unwrap_or("");
        let base_path_mapping = sam_resources.iter().find(|(_, sub_resource)| {
            sub_resource["Type"] == "AWS::ApiGateway::BasePathMapping"
                && sub_resource["Properties"]["RestApiId"].is_string()
                && sub_resource["Properties"]["RestApiId"]
                    .as_str()
                    .unwrap_or("")
                    == sam_event_api_gateway
        });

        if let Some(base_path_mapping) = base_path_mapping {
            format!(
                "/{}",
                &base_path_mapping.1["Properties"]["BasePath"]
                    .as_str()
                    .unwrap_or("")
            )
        } else {
            "".to_owned()
        }
    } else {
        "".to_owned()
    }
}

fn trim_path_ending_slash(path: String) -> String {
    if let Some(last_character) = path.chars().nth(path.len() - 1) {
        if path.len() > 1 && last_character == '/' {
            path[0..path.len() - 1].to_owned()
        } else {
            path.to_owned()
        }
    } else {
        path.to_owned()
    }
}

fn replaced_regex_path(path: &str) -> String {
    // As SAM supports parameters in url with {param} syntax we need to replace them with usable regex
    let replace_matches: Regex = Regex::new("{.*?}").expect("invalid regex");

    let replaced_sam_path =
        replace_matches
            .find_iter(path)
            .fold(path.to_string(), |mut acc, current_match| {
                if let Ok(current_match) = current_match {
                    let current_match_name: &str =
                        &current_match.as_str()[1..current_match.as_str().len() - 1];

                    if current_match_name.ends_with('+') {
                        acc = acc.replace(
                            current_match.as_str(),
                            &format!(
                                r"(?P<{}>.*)",
                                &current_match_name[0..current_match_name.len() - 1]
                            ),
                        );
                    } else {
                        acc = acc.replace(
                            current_match.as_str(),
                            &format!(r"(?P<{}>[^\/]+)", &current_match_name),
                        );
                    }
                };
                acc
            });

    format!("^{}$", replaced_sam_path)
}
