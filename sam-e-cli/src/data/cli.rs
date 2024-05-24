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
    #[clap(subcommand)]
    Frontend(FrontendCommand),
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
    #[command(about = "Add a new template to the list of templates")]
    Add,
}

#[derive(Debug, Subcommand)]
pub enum FrontendCommand {
    #[command(
        about = "Add a new frontend to the local environment. Note: this currently sits disconnected from the template files."
    )]
    Add,
    #[command(about = "Remove a frontend from the local environment")]
    Remove,
    #[command(about = "Start the frontend")]
    Start,
    #[command(about = "Stop the frontend")]
    Stop,
}

#[derive(Debug, Subcommand)]
pub enum FunctionCommand {
    #[command(about = "Add a new function from the template list to the local environment")]
    Add,

    #[clap(subcommand)]
    Group(FunctionGroupCommand),
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

#[derive(Debug, Subcommand)]
pub enum FunctionGroupCommand {
    #[command(about = "Create a new function group to the local environment")]
    Create,
    #[command(about = "Delete a function group from the local environment (won't delete the functions)")]
    Delete,
    #[command(about = "Add a function to a function group")]
    Add,
    #[command(about = "Remove a function from a function group")]
    Remove,
}
