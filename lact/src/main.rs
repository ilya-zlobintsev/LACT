mod args;

use args::{Args, Command};
use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let command = args.command.unwrap_or(Command::Gui);

    match command {
        Command::Daemon => lact_daemon::run(),
        Command::Gui => run_gui(),
        Command::Cli(cli_args) => lact_cli::run(cli_args),
    }
}

#[cfg(feature = "lact-gui")]
fn run_gui() -> anyhow::Result<()> {
    lact_gui::run()
}

#[cfg(not(feature = "lact-gui"))]
fn run_gui() -> anyhow::Result<()> {
    use anyhow::anyhow;

    Err(anyhow!("LACT was built without GUI support"))
}
