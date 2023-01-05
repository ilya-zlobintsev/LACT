mod args;

use args::Command;
use clap::Parser;

fn main() -> anyhow::Result<()> {
    let command = Command::parse();
    match command {
        Command::Daemon => lact_daemon::run(),
        Command::Gui => lact_gui::run(),
    }
}
