use crate::{
    data::cli::Command,
    scripts::{build, init, start, rebuild},
};
use tracing::debug;

pub async fn get_command_script(command: Command) -> anyhow::Result<()> {
    debug!("Getting command script for command: {:?}", command);

    let command_match = match command {
        Command::Init => init::init(),
        Command::Build(args) => build::build(args),
        Command::Start => start::start().await,
        Command::Rebuild => rebuild::rebuild(),
    };

    command_match
}
