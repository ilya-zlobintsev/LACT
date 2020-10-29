use daemon::Daemon;

fn main() {
    env_logger::init();
    let d = Daemon::new();
    d.listen();
}
