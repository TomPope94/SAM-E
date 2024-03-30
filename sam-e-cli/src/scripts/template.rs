mod update;
pub mod utils;
mod validate;
mod add;

use crate::data::cli::TemplateCommand;
use tracing::debug;

pub fn get_template_script(command: TemplateCommand) -> anyhow::Result<()> {
    debug!("Getting template script for command: {:?}", command);

    match command {
        TemplateCommand::Add => add::add()?,
        TemplateCommand::Validate => validate::validate()?,
        TemplateCommand::Update => update::update()?,
    }
    Ok(())
}
