use std::{env, fs};
use tracing::{debug, info};

use crate::scripts::{
    environment::build::template::find_all_files,
    utils::{check_init, get_config, get_sam_e_directory_path},
};

pub fn add() -> anyhow::Result<()> {
    info!("Adding a new template to the list of templates");

    check_init()?;
    let mut config = get_config()?;
    let mut runtime = config.get_runtime().clone();
    let templates = runtime.get_templates();
    let template_locations = templates
        .iter()
        .map(|template| template.get_location())
        .collect::<Vec<&str>>();

    let current_directory = env::current_dir()?;

    let all_yaml_files = find_all_files(&current_directory, "(.*?)\\.(yaml)$")?;
    let non_used_yaml_files = all_yaml_files
        .iter()
        .map(|file| file.to_str().unwrap())
        .filter(|file| !template_locations.contains(&file))
        .collect::<Vec<&str>>();

    let selection = dialoguer::MultiSelect::new()
        .with_prompt("Please select the YAML files you would like to add to the templates. Use space to select. Press enter when done.")
        .items(&non_used_yaml_files)
        .interact()?;
    debug!("Selected YAML files: {:?}", selection);

    for location in selection {
        runtime.add_template_str(non_used_yaml_files[location]);
    }

    let sam_e_directory_path = get_sam_e_directory_path()?;
    config.set_runtime(runtime);
    let config_string = serde_yaml::to_string(&config)?;

    let sam_e_config_path = format!("{}/sam-e-config.yaml", sam_e_directory_path);
    info!("Updating SAM-E config file at: {:?}", sam_e_config_path);

    fs::write(&sam_e_config_path, config_string)?;
    debug!("SAM-E config file updated successfully");

    Ok(())
}
