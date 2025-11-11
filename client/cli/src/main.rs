mod cli;
mod commands;
mod validator_parser;

use clap::Parser;
use cli::Cli;
use commands::run_command;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    run_command(cli)
}
