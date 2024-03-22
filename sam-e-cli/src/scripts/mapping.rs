use crate::{
    data::cli::Command,
    scripts::{build, init, rebuild, start, stop},
};
use tracing::debug;

pub async fn get_command_script(command: Command) -> anyhow::Result<()> {
    debug!("Getting command script for command: {:?}", command);

    match command {
        Command::Init => init::init(),
        Command::Build => build::build(),
        Command::Start(args) => start::start(args).await,
        Command::Rebuild => rebuild::rebuild(),
        Command::Stop => stop::stop(),
    }
}
