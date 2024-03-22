use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    /// A flag to enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// A flag to enable quiet logging
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// A flag to enable trace logging
    #[arg(short, long, global = true)]
    pub trace: bool,

    /// Specify what action you would like SAM-E to perform
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(
        about = "Initiate the SAM-E environment config. Run if first time using SAM-E in this project."
    )]
    Init,

    #[command(about = "Build the SAM-E environment using a SAM template.yaml file")]
    Build,

    #[command(
        about = "Rebuild the SAM-E environment using just the Config (allowing for manual changes). Will not rebuild from the SAM template file"
    )]
    Rebuild,

    #[command(
        about = "Start the SAM-E environment. Will prompt you to choose which part of the environment to start."
    )]
    Start(StartArgs),

    #[command(about = "Stop the SAM-E environment")]
    Stop,
}

#[derive(Debug, Args)]
pub struct StartArgs {
    /// A flag to run the docker files detached
    #[arg(short, long)]
    pub detached: bool,
}
