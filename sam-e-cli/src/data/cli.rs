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
    #[clap(subcommand)]
    Function(FunctionCommand),
    #[clap(subcommand)]
    Environment(EnvironmentCommand),
    #[clap(subcommand)]
    Template(TemplateCommand),
}

#[derive(Debug, Args)]
pub struct StartArgs {
    /// A flag to run the docker files detached
    #[arg(short, long)]
    pub detached: bool,
}

#[derive(Debug, Subcommand)]
pub enum TemplateCommand {
    #[command(about = "TODO: Validate the SAM-E template.yaml file")]
    Validate,

    #[command(about = "Update the SAM-E template.yaml file")]
    Update,
}

pub enum FunctionCommand {
    Build,
}

#[derive(Debug, Subcommand)]
pub enum EnvironmentCommand {
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
