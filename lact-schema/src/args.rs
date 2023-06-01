pub use clap;

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
}

#[derive(Parser)]
#[command(author, version, about)]
pub struct CliArgs {
    #[arg(short, long)]
    pub gpu_id: Option<String>,
    #[command(subcommand)]
    pub subcommand: CliCommand,
}

#[derive(Subcommand)]
pub enum CliCommand {
    /// List GPUs
    ListGpus,
    /// Show GPU info
    Info,
}
