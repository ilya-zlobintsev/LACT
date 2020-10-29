use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    thread,
    time::Duration,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum HWMonError {
    PermissionDenied,
}

#[derive(Debug, Clone)]
pub struct HWMon {
    hwmon_path: PathBuf,
    pub fan_max_speed: i32,
    fan_control: Arc<AtomicBool>,
    fan_curve: Arc<RwLock<BTreeMap<i32, f64>>>,
}

impl HWMon {
    pub fn new(hwmon_path: &PathBuf, fan_control_enabled: bool, fan_curve: BTreeMap<i32, f64>) -> HWMon {
        let fan_max_speed = fs::read_to_string(hwmon_path.join("fan1_max"))
            .unwrap()
            .trim()
            .parse::<i32>()
            .unwrap();

        let mon = HWMon {
            hwmon_path: hwmon_path.clone(),
            fan_max_speed,
            fan_control: Arc::new(AtomicBool::new(false)),
            fan_curve: Arc::new(RwLock::new(fan_curve)),
        };

        if fan_control_enabled {
            mon.start_fan_control().unwrap();
        }
        
        mon
    }

    pub fn get_fan_speed(&self) -> i32 {
        /*if self.fan_control.load(Ordering::SeqCst) {
            let pwm1 = fs::read_to_string(self.hwmon_path.join("pwm1"))
                .expect("Couldn't read pwm1")
                .trim()
                .parse::<i32>()
                .unwrap();

            self.fan_max_speed / 255 * pwm1
        }
        else {
            fs::read_to_string(self.hwmon_path.join("fan1_input"))
                .expect("Couldn't read fan speed")
                .trim()
                .parse::<i32>()
                .unwrap()
        }*/
        fs::read_to_string(self.hwmon_path.join("fan1_input"))
            .expect("Couldn't read fan speed")
            .trim()
            .parse::<i32>()
            .unwrap()
    }

    pub fn get_mem_freq(&self) -> i32 {
        let filename = self.hwmon_path.join("freq2_input");

        fs::read_to_string(filename)
            .unwrap()
            .trim()
            .parse::<i32>()
            .unwrap()
            / 1000
            / 1000
    }

    pub fn get_gpu_freq(&self) -> i32 {
        let filename = self.hwmon_path.join("freq1_input");

        fs::read_to_string(filename)
            .unwrap()
            .trim()
            .parse::<i32>()
            .unwrap()
            / 1000
            / 1000
    }

    pub fn get_gpu_temp(&self) -> i32 {
        let filename = self.hwmon_path.join("temp1_input");

        fs::read_to_string(filename)
            .unwrap()
            .trim()
            .parse::<i32>()
            .unwrap()
            / 1000
    }

    pub fn get_power_cap(&self) -> i32 {
        let filename = self.hwmon_path.join("power1_cap");

        fs::read_to_string(filename)
            .unwrap()
            .trim()
            .parse::<i32>()
            .unwrap()
            / 1000000
    }

    pub fn get_power_avg(&self) -> i32 {
        let filename = self.hwmon_path.join("power1_average");

        fs::read_to_string(filename)
            .unwrap()
            .trim()
            .parse::<i32>()
            .unwrap()
            / 1000000
    }

    pub fn set_fan_curve(&self, curve: BTreeMap<i32, f64>) {
        log::trace!("trying to set curve");
        let mut current = self.fan_curve.write().unwrap();
        current.clear();

        for (k, v) in curve.iter() {
            current.insert(k.clone(), v.clone());
        }
        log::trace!("set curve to {:?}", current);
    }

    pub fn start_fan_control(&self) -> Result<(), HWMonError> {
        if self.fan_control.load(Ordering::SeqCst) {
            return Ok(());
        }
        self.fan_control.store(true, Ordering::SeqCst);

        match fs::write(self.hwmon_path.join("pwm1_enable"), "1") {
            Ok(_) => {
                let s = self.clone();

                thread::spawn(move || {
                    while s.fan_control.load(Ordering::SeqCst) {
                        let curve = s.fan_curve.read().unwrap();

                        let temp = s.get_gpu_temp();
                        log::trace!("Current gpu temp: {}", temp);
                        for (t_low, s_low) in curve.iter() {
                            match curve.range(t_low..).nth(1) {
                                Some((t_high, s_high)) => {
                                    if (t_low..t_high).contains(&&temp) {
                                        let speed_ratio =
                                            (temp - t_low) as f64 / (t_high - t_low) as f64; //The ratio of which speed to choose within the range of current lower and upper speeds
                                        let speed_percent =
                                            s_low + ((s_high - s_low) * speed_ratio);
                                        let pwm = (255f64 * (speed_percent / 100f64)) as i32;
                                        log::trace!("pwm: {}", pwm);

                                        fs::write(s.hwmon_path.join("pwm1"), pwm.to_string())
                                            .expect("Failed to write to pwm1");

                                        log::trace!("In the range of {}..{}c {}..{}%, setting speed {}% ratio {}", t_low, t_high, s_low, s_high, speed_percent, speed_ratio);
                                        break;
                                    }
                                }
                                None => (),
                            }
                        }
                        drop(curve); //needed to release rwlock so that the curve can be changed

                        thread::sleep(Duration::from_millis(1000));
                    }
                });
                Ok(())
            }
            Err(_) => Err(HWMonError::PermissionDenied),
        }
    }

    pub fn stop_fan_control(&self) -> Result<(), HWMonError> {
        match fs::write(self.hwmon_path.join("pwm1_enable"), "2") {
            Ok(_) => {
                self.fan_control.store(false, Ordering::SeqCst);
                log::trace!("Stopping fan control");
                Ok(())
            }
            Err(_) => Err(HWMonError::PermissionDenied),
        }
    }

    pub fn get_fan_control(&self) -> (bool, BTreeMap<i32, f64>) {
        log::trace!("Fan control: {}", self.fan_control.load(Ordering::SeqCst));
        (
            self.fan_control.load(Ordering::SeqCst),
            self.fan_curve.read().unwrap().clone(),
        )
    }
}
