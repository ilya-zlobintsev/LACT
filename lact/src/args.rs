use clap::{Parser, Subcommand};
use lact_cli::args::CliArgs;

#[derive(Parser)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Run the daemon
    Daemon,
    /// Run the GUI
    Gui,
    Cli(CliArgs),
}
