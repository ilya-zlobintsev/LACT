mod flatpak;

use lact_schema::args::{clap::Parser, Args, Command, GuiArgs};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let command = args
        .command
        .unwrap_or_else(|| Command::Gui(GuiArgs::default()));

    match command {
        Command::Daemon => lact_daemon::run(),
        Command::Gui(gui_args) => run_gui(gui_args),
        Command::Cli(cli_args) => lact_cli::run(cli_args),
        Command::FlatpakDaemonCmd => flatpak::generate_daemon_cmd(),
    }
}

#[cfg(feature = "lact-gui")]
fn run_gui(args: GuiArgs) -> anyhow::Result<()> {
    lact_gui::run(args)
}

#[cfg(not(feature = "lact-gui"))]
fn run_gui(_: GuiArgs) -> anyhow::Result<()> {
    use anyhow::anyhow;
    Err(anyhow!("LACT was built without GUI support"))
}
