mod args;

use args::Command;
use clap::Parser;

fn main() -> anyhow::Result<()> {
    match Command::try_parse() {
        Ok(Command::Daemon) => lact_daemon::run(),
        Ok(Command::Gui) => run_gui(),
        Err(err) => {
            println!("{err}");
            Ok(())
        }
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
