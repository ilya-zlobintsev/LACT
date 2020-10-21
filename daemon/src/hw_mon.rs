use std:: {fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct HWMon {
    hwmon_path: PathBuf,
}

impl HWMon {
    pub fn new(hwmon_path: &PathBuf) -> HWMon {

        HWMon {
            hwmon_path: hwmon_path.clone(),
        }
    }

    pub fn get_fan_speed(&self) -> i32 {
        fs::read_to_string(self.hwmon_path.join("fan1_input"))
            .expect("Couldn't read fan speed")
            .parse::<i32>()
            .unwrap()
    }

    pub fn get_mem_freq(&self) -> i32 {
        let filename = self.hwmon_path.join("freq2_input");

        fs::read_to_string(filename).unwrap().trim().parse::<i32>().unwrap() / 1000 / 1000
    }

    pub fn get_gpu_freq(&self) -> i32 {
        let filename = self.hwmon_path.join("freq1_input");

        fs::read_to_string(filename).unwrap().trim().parse::<i32>().unwrap() / 1000 / 1000
    }

    pub fn get_gpu_temp(&self) -> i32 {
        let filename = self.hwmon_path.join("temp1_input");

        fs::read_to_string(filename).unwrap().trim().parse::<i32>().unwrap() / 1000
    }

    pub fn get_power_cap(&self) -> i32 {
        let filename = self.hwmon_path.join("power1_cap");

        fs::read_to_string(filename).unwrap().trim().parse::<i32>().unwrap() / 1000000
    }

    pub fn get_power_avg(&self) -> i32 {
        let filename = self.hwmon_path.join("power1_average");

        fs::read_to_string(filename).unwrap().trim().parse::<i32>().unwrap() / 1000000
    }
}
