pub mod environment;
pub mod function;
pub mod template;
pub mod utils;

pub use environment::get_environment_script;
pub use function::get_function_script;
use template::get_template_script;

use crate::data::cli::Command;
use tracing::debug;

pub async fn get_command_script(command: Command) -> anyhow::Result<()> {
    debug!("Getting command script for command: {:?}", command);

    match command {
        Command::Function(subcommand) => get_function_script(subcommand).await,
        Command::Environment(subcommand) => get_environment_script(subcommand).await,
        Command::Template(subcommand) => get_template_script(subcommand),
    }
}
