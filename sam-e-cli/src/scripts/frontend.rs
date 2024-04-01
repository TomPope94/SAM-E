mod add;
mod remove;
mod start;
mod stop;

use add::add_frontend;
use remove::remove_frontend;
use start::start_frontend;
use stop::stop_frontend;

use crate::data::cli::FrontendCommand;

use tracing::debug;

pub fn get_frontend_script(subcommand: FrontendCommand) -> anyhow::Result<()> {
    debug!("Getting frontend script for subcommand: {:?}", subcommand);

    match subcommand {
        FrontendCommand::Add => add_frontend(),
        FrontendCommand::Remove => remove_frontend(),
        FrontendCommand::Start => start_frontend(),
        FrontendCommand::Stop => stop_frontend(),
    }
}
