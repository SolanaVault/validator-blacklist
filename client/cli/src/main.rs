mod cli;
mod commands;

use clap::Parser;
use cli::Cli;
use commands::run_command;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    run_command(cli).await
}
