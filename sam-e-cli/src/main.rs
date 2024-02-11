pub mod data;
pub mod scripts;

use std::env;

use clap::Parser;
use tracing_subscriber::EnvFilter;

use scripts::get_command_script;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = data::Cli::parse();

    if args.verbose {
        env::set_var("RUST_LOG", "trace");
    } else {
        env::set_var("RUST_LOG", "info");
    }

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .with_ansi(false)
        .without_time()
        .init();

    get_command_script(args.command).await
}
