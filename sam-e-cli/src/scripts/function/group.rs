pub mod add;
pub mod create;

use crate::data::cli::FunctionGroupCommand;

pub async fn get_group_script(subcommand: FunctionGroupCommand) -> anyhow::Result<()> {
    match subcommand {
        FunctionGroupCommand::Create => create::create().await,
        FunctionGroupCommand::Delete => todo!("Delete group"),
        FunctionGroupCommand::Add => add::add_function().await,
        FunctionGroupCommand::Remove => todo!("Remove function from group"),
    }
}
