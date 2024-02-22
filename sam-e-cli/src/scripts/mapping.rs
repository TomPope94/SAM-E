use crate::{
    data::cli::Command,
    scripts::{build, init, rebuild, start},
};
use tracing::debug;

pub async fn get_command_script(command: Command) -> anyhow::Result<()> {
    debug!("Getting command script for command: {:?}", command);

    let command_match = match command {
        Command::Init => init::init(),
        Command::Build => build::build(),
        Command::Start => start::start().await,
        Command::Rebuild => rebuild::rebuild(),
    };

    command_match
}
