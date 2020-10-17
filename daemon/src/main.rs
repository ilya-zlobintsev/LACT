use daemon::daemon::Daemon;
use daemon::fan_controller::FanController;

fn main() {
    let fan_controller = FanController::new("afsadfasdfa");
    let d = Daemon::new(fan_controller);
    d.run();
}

