pub mod build;
pub mod deploy;
pub mod init;
pub mod rebuild;
pub mod start;
pub mod stop;

pub use build::build;
pub use init::init;
pub use rebuild::rebuild;
pub use start::start;
pub use stop::stop;

use crate::data::cli::EnvironmentCommand;

pub async fn get_environment_script(subcommand: EnvironmentCommand) -> anyhow::Result<()> {
    match subcommand {
        EnvironmentCommand::Init => init::init(),
        EnvironmentCommand::Build => build::build(),
        EnvironmentCommand::Start(args) => start::start(args).await,
        EnvironmentCommand::Rebuild => rebuild::rebuild(),
        EnvironmentCommand::Stop => stop::stop(),
        EnvironmentCommand::Deploy => deploy::deploy(),
    }
}
