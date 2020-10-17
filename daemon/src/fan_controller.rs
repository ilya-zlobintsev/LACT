#[derive(Clone)]
pub struct FanController {
    hwmon_path: String,
}

impl FanController {
    pub fn new(hwmon_path: &str) -> FanController {
        FanController { hwmon_path: hwmon_path.to_string() }
    }
}