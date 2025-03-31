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
    /// Generate a command that runs the daemon from flatpak
    FlatpakDaemonCmd,
}

#[derive(Default, Parser)]
pub struct GuiArgs {
    #[arg(long)]
    pub log_level: Option<String>,
    /// Remote TCP address to connect to
    #[arg(long)]
    pub tcp_address: Option<String>,
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
    /// Generate debug snapshot
    Snapshot,
}
