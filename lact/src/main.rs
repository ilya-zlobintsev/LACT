use lact_schema::args::{Args, Command, GuiArgs, clap::Parser};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let command = args
        .command
        .unwrap_or_else(|| Command::Gui(GuiArgs::default()));

    match command {
        Command::Daemon(daemon_args) => lact_daemon::run_with_options(lact_daemon::DaemonOptions {
            nvidia_init_mode: daemon_args.nvidia_init_mode,
        }),
        Command::Gui(gui_args) => run_gui(gui_args),
        Command::Cli(cli_args) => lact_cli::run(cli_args),
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
