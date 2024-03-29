pub mod build;
pub mod deploy;
pub mod init;
pub mod rebuild;
pub mod start;
pub mod stop;
pub mod template;
pub mod utils;

use build::build;
use deploy::deploy;
use init::init;
use rebuild::rebuild;
use start::start;
use stop::stop;
use template::get_template_script;

use crate::data::cli::Command;
use tracing::debug;

pub async fn get_command_script(command: Command) -> anyhow::Result<()> {
    debug!("Getting command script for command: {:?}", command);

    match command {
        Command::Init => init(),
        Command::Build => build(),
        Command::Deploy => deploy(),
        Command::Start(args) => start(args).await,
        Command::Rebuild => rebuild(),
        Command::Stop => stop(),
        Command::Template(subcommand) => get_template_script(subcommand),
    }
}
