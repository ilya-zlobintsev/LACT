pub mod cli;

pub use clap;

use crate::args::cli::CliArgs;
use clap::{Parser, Subcommand};

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
    Gui(GuiArgs),
    /// Run the CLI
    Cli(CliArgs),
}

#[derive(Default, Parser)]
pub struct GuiArgs {
    #[arg(long)]
    pub log_level: Option<String>,
    /// Remote TCP address to connect to
    #[arg(long)]
    pub tcp_address: Option<String>,
}
