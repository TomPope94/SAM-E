use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use fancy_regex::Regex;
use tracing::{debug, trace};

use sam_e_types::{
    cloudformation::Template,
    config::runtime::template::Template as ConfigTemplate,
};
use crate::scripts::build::ResourceWithTemplate;

/// Takes the vec of template locations (i.e. file paths to the YAML files) and returns a hashmap
/// of the resources section of the CloudFormation template.
pub fn parse_templates_into_resources(
    templates: &Vec<ConfigTemplate>,
) -> Result<HashMap<String, ResourceWithTemplate>> {
    let mut resources: HashMap<String, ResourceWithTemplate> = HashMap::new();

    for config_template in templates {
        let temp_resources = build_template(config_template)?;
        temp_resources.into_iter().for_each(|(k, v)| {
            resources.insert(k.to_string(), v);
        });
    }

    Ok(resources)
}

/// Builds the template for an individual CloudFormation template returning a hashmap of just
/// the resources section. Starts by reading the file to a string before passing to serde_yaml to
/// be parsed into the HashMap.
fn build_template(template: &ConfigTemplate) -> anyhow::Result<HashMap<String, ResourceWithTemplate>> {
    debug!("Building template: {:?}", template.get_name());
    let template_path = Path::new(template.get_location());

    let path_as_str = template_path.to_str().unwrap();
    debug!("Template path: {}", path_as_str);

    let yaml_file = fs::read_to_string(template_path)?;
    debug!("YAML file read successfully");

    let template_value: Template = serde_yaml::from_str(&yaml_file)?;
    debug!("Template value: {:#?}", template_value);

    let template_resources = template_value.resources;
    let mut resources_with_template: HashMap<String, ResourceWithTemplate> = HashMap::new();
    template_resources.into_iter().for_each(|(k, v)| {
        resources_with_template.insert(k.to_string(), ResourceWithTemplate::new(v, template.get_name()));
    });

    Ok(resources_with_template)
}

/// Recursively goes through directories to find all files that match a specific regex pattern.
pub fn find_all_files(path: &impl AsRef<Path>, to_find: &str) -> std::io::Result<Vec<PathBuf>> {
    let mut buf = vec![];
    let regex = Regex::new(to_find).expect("Invalid regex pattern");

    trace!("Reading entries in {:?}", path.as_ref());
    let entries = fs::read_dir(path)?;

    for entry in entries {
        let entry = entry?;
        let meta = entry.metadata()?;
        trace!("Found entry: {:?}", entry.path());

        if meta.is_dir() {
            trace!("Entry recognized as directory, recursing...");
            let mut subdir = find_all_files(&entry.path(), to_find)?;
            buf.append(&mut subdir);
        }

        if meta.is_file() && regex.is_match(entry.file_name().to_str().unwrap()).unwrap() {
            trace!("Entry recognized as file, adding to buffer...");
            trace!("Found file: {:?}", entry.path());
            buf.push(entry.path());
        }
    }

    Ok(buf)
}

// Takes a yaml value and returns a string. If the value is a reference using a yaml tag (i.e.
// !Ref) it will return a string of what's being referenced. If it's already a string it will
// simply return that. Anything else will error.
// pub fn handle_value_reference(reference: &Value) -> Result<String> {
//     match reference {
//         Value::String(s) => Ok(s.to_string()),
//         Value::Tagged(tagged_value) => handle_yaml_tag(tagged_value.to_owned()),
//         _ => Err(Error::msg("Value is not a string or reference")),
//     }
// }
//
// fn handle_yaml_tag(value: TaggedValue) -> Result<String> {
//     if value.tag() == "!GetAtt"
//
// }
