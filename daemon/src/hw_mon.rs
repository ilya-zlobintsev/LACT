use std:: {fs, path::PathBuf, sync::{Arc, atomic::{AtomicBool, Ordering}}, thread, time::Duration};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum HWMonError {
    PermissionDenied,
}

#[derive(Debug, Clone, Default)]
pub struct HWMon {
    hwmon_path: PathBuf,
    fan_max_speed: i32,
    fan_control: Arc<AtomicBool>,
}

impl HWMon {
    pub fn new(hwmon_path: &PathBuf) -> HWMon {
        let fan_max_speed = fs::read_to_string(hwmon_path.join("fan1_max")).unwrap().trim().parse::<i32>().unwrap();

        HWMon {
            hwmon_path: hwmon_path.clone(),
            fan_max_speed,
            fan_control: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn get_fan_speed(&self) -> i32 {
        fs::read_to_string(self.hwmon_path.join("fan1_input"))
            .expect("Couldn't read fan speed")
            .trim()
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

    pub fn start_fan_control(&self) -> Result<(), HWMonError> {
        self.fan_control.store(true, Ordering::SeqCst);

        match fs::write(self.hwmon_path.join("pwm1_enable"), "1") {
            Ok(_) => {
                let s = self.clone();

                thread::spawn(move || {
                    while s.fan_control.load(Ordering::SeqCst) {
                            let temp = s.get_gpu_temp();
                            println!("{}", temp);
                            thread::sleep(Duration::from_millis(1000));
                        }
                    });
                Ok(())
            },
            Err(_) => Err(HWMonError::PermissionDenied)
        }
    }

    pub fn stop_fan_control(&self) -> Result<(), HWMonError> {
        match fs::write(self.hwmon_path.join("pwm1_enable"), "2") {
            Ok(_) => {
                self.fan_control.store(false, Ordering::SeqCst);
                println!("Stopping fan control");
                Ok(())
            },
            Err(_) => Err(HWMonError::PermissionDenied)
        }
        
    }
}
