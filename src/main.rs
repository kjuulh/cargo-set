use std::env::Args;

use clap::{Parser, Subcommand};

fn main() -> anyhow::Result<()> {
    let args = std::env::args();

    cli_execute(args)
}

fn cli_execute(args: Args) -> anyhow::Result<()> {
    let cli = Cli::parse_from(args);

    match &cli.command {
        Some(Commands::Set {}) => {}
        None => {}
    }

    Ok(())
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
#[command(subcommand_required = true)]
struct Cli {
    name: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Set {},
}
