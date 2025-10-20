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
    /// Show GPU stats
    Stats,
    /// Generate debug snapshot
    Snapshot,
    /// Manage GPU power limit
    PowerLimit {
        #[command(subcommand)]
        cmd: Option<PowerLimitCmd>,
    },
    /// Manage profiles
    Profile(ProfileArgs),
}

#[derive(Parser, Clone, Copy)]
pub enum PowerLimitCmd {
    /// Get current power limit and allowed range
    Get,
    /// Set power limit
    Set { limit: u32 },
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
    /// Manage profile auto switching
    AutoSwitch(ProfileAutoSwitchArgs),
}

#[derive(Parser)]
pub struct SetProfileArgs {
    pub name: String,
}

#[derive(Parser)]
pub struct ProfileAutoSwitchArgs {
    #[command(subcommand)]
    pub subcommand: Option<ProfileAutoSwitchCommand>,
}

#[derive(Subcommand)]
pub enum ProfileAutoSwitchCommand {
    /// Current auto switch state
    Get,
    /// Enable auto switching
    Enable,
    /// Disable auto switching
    Disable,
}
