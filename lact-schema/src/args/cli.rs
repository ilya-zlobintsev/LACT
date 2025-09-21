use clap::{Parser, Subcommand};

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
    #[clap(alias = "list-gpus")]
    List,
    /// Show GPU info
    Info,
    /// Generate debug snapshot
    Snapshot,
    /// Manage GPU power limit
    PowerLimit {
        #[command(subcommand)]
        cmd: Option<PowerLimitCmd>,
    },
}

#[derive(Parser, Clone, Copy)]
pub enum PowerLimitCmd {
    /// Get current power limit and allowed range
    Get,
    /// Set power limit
    Set { limit: u32 },
}
