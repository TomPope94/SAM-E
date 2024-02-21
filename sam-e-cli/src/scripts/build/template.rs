use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Error, Result};
use serde_yaml::{value::TaggedValue, Value};
use tracing::{debug, trace, warn};

/// Gets the raw CloudFormation template file from directory and returns resources specified as
/// hashmap. If multi set to true, will return all files found collated (i.e. all resources across
/// all the files).
pub fn collect_template_to_resource(
    template_name: &str,
    multi: &bool,
    current_dir: &PathBuf,
) -> anyhow::Result<HashMap<String, Value>> {
    let template_files = find_all_files(&current_dir, template_name)?;
    if template_files.is_empty() {
        return Err(Error::msg("No template files found"));
    }

    let resources = match multi {
        true => {
            if template_files.len() == 1 {
                warn!("The multi flag was set to true but only one template file found. If this is correct we recommend removing the multi flag to avoid unexpected behaviour.")
            }
            let mut resources = HashMap::new();
            for file in template_files {
                let temp_resources = build_template(&file)?;
                temp_resources.iter().for_each(|(k, v)| {
                    resources.insert(k.to_string(), v.to_owned());
                });
            }
            Ok(resources)
        }
        false => {
            if template_files.len() > 1 {
                warn!("Multiple template files found but multi flag set to false (either by default or manually), will use the first one vs collating together");
            }
            build_template(&template_files[0])
        }
    };

    resources
}

/// Builds the template for an individual CloudFormation template returning a hashmap of just
/// the resources section. Starts by reading the file to a string before passing to serde_yaml to
/// be parsed into the HashMap.
fn build_template(template: &PathBuf) -> anyhow::Result<HashMap<String, Value>> {
    debug!("Building template: {:?}", template);

    let template_path = template.to_str().unwrap();
    debug!("Template path: {}", template_path);

    let yaml_file = fs::read_to_string(template_path)?;
    debug!("YAML file read successfully");

    let template_value: Value = serde_yaml::from_str(&yaml_file)?;
    let resources = serde_yaml::from_value(template_value["Resources"].to_owned())?;

    Ok(resources)
}

/// Recursively goes through directories to find all files of a specific name
fn find_all_files(path: &impl AsRef<Path>, filename: &str) -> std::io::Result<Vec<PathBuf>> {
    let mut buf = vec![];

    trace!("Reading entries in {:?}", path.as_ref());
    let entries = fs::read_dir(path)?;

    for entry in entries {
        let entry = entry?;
        let meta = entry.metadata()?;
        trace!("Found entry: {:?}", entry.path());

        if meta.is_dir() {
            trace!("Entry recognized as directory, recursing...");
            let mut subdir = find_all_files(&entry.path(), filename)?;
            buf.append(&mut subdir);
        }

        if meta.is_file() && entry.file_name().to_str().unwrap() == filename {
            trace!("Entry recognized as file, adding to buffer...");
            debug!("Found file: {:?}", entry.path());
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
