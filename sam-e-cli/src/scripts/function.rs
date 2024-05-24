mod add;
mod group;

use crate::data::cli::FunctionCommand;

pub async fn get_function_script(subcommand: FunctionCommand) -> anyhow::Result<()> {
    match subcommand {
        FunctionCommand::Add => add::add(),
        FunctionCommand::Group(subcommand) => group::get_group_script(subcommand).await,
    }
}
