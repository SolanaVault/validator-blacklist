mod cli;
mod commands;

use clap::Parser;
use cli::Cli;
use commands::run_command;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    run_command(cli)
}
