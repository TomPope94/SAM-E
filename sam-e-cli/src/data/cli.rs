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
    Build(BuildArgs),

    #[command(about = "Rebuild the SAM-E environment using just the Config (allowing for manual changes). Will not rebuild from the SAM template file")]
    Rebuild,

    #[command(
        about = "Start the SAM-E environment. Will run an Axum API server. Run the generated docker-compose file in separate terminal to complete setup."
    )]
    Start,
}

#[derive(Debug, Args)]
pub struct BuildArgs {
    /// The name of SAM template yaml file to search for at current location. If multi is true, will use this name for ALL files. Note: Will search in child directories
    #[arg(long)]
    pub template_name: Option<String>,

    /// Boolean for whether there is more than one SAM file. Will default to false
    #[arg(short, long)]
    pub multi: Option<bool>,

    /// Boolean for whether it should overwrite the current SAM-E environment or merge with it (if it exists). Will default to false
    #[arg(short, long)]
    pub overwrite: Option<bool>,
}
