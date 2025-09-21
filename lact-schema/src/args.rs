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
    /// Manage profiles or show current profile
    Profile(ProfileArgs),
}

#[derive(Parser)]
pub struct ProfileArgs {
    #[command(subcommand)]
    pub subcommand: Option<ProfileCommand>,
}


#[derive(Subcommand)]
pub enum ProfileCommand {
    /// List profiles
    List,
    /// Current profile
    Get,
    /// Set profile
    Set(SetProfileArgs),
}

#[derive(Parser)]
pub struct SetProfileArgs {
    #[arg(short, long, required = true)]
    pub name: Option<String>,
    #[arg(short, long, required = true)]
    pub auto_switch: Option<bool>,
}
