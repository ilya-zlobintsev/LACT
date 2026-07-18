pub mod cli;

pub use clap;

use crate::{NvidiaInitMode, args::cli::CliArgs};
use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Run the daemon
    Daemon(DaemonArgs),
    /// Run the GUI
    Gui(GuiArgs),
    /// Run the CLI
    Cli(CliArgs),
}

#[derive(Default, Parser)]
pub struct DaemonArgs {
    /// Select how the daemon initializes the Nvidia management library
    #[arg(long, value_enum, default_value = "normal")]
    pub nvidia_init_mode: NvidiaInitMode,
}

#[derive(Default, Parser)]
pub struct GuiArgs {
    #[arg(long)]
    pub log_level: Option<String>,
    /// Remote TCP address to connect to
    #[arg(long)]
    pub tcp_address: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daemon_uses_normal_nvidia_init_by_default() {
        let args = Args::try_parse_from(["lact", "daemon"]).unwrap();
        let Some(Command::Daemon(daemon_args)) = args.command else {
            panic!("daemon command was not parsed");
        };

        assert_eq!(daemon_args.nvidia_init_mode, NvidiaInitMode::Normal);
    }

    #[test]
    fn parses_allow_no_gpus_nvidia_init_mode() {
        let args =
            Args::try_parse_from(["lact", "daemon", "--nvidia-init-mode=allow-no-gpus"]).unwrap();
        let Some(Command::Daemon(daemon_args)) = args.command else {
            panic!("daemon command was not parsed");
        };

        assert_eq!(daemon_args.nvidia_init_mode, NvidiaInitMode::AllowNoGpus);
    }
}
