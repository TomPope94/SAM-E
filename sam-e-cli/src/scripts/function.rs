mod add;

use crate::data::cli::FunctionCommand;

pub async fn get_function_script(subcommand: FunctionCommand) -> anyhow::Result<()> {
    match subcommand {
        FunctionCommand::Add => add::add(),
    }
}
