use daemon::Daemon;

fn main() {
    let d = Daemon::new();
    d.listen();
}
