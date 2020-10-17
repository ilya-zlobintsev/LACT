use daemon::daemon::Daemon;
use daemon::gpu_controller::GpuController;

fn main() {
    let gpu_controller = GpuController::new("/sys/class/drm/card0/device");
    let d = Daemon::new(gpu_controller);
    d.run();
}

