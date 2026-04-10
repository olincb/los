mod cli;
use clap::Parser;
use cli::{Cli, handle_main_cli};

fn main() -> anyhow::Result<()> {
    handle_main_cli(Cli::parse())
}
