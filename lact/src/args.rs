use clap::Parser;

#[derive(Parser)]
pub enum Command {
    /// Run the daemon
    Daemon,
    /// Run the GUI
    Gui,
}
