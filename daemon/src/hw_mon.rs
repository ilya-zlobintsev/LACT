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
    InvalidValue,
    NoHWMon,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HWMon {
    hwmon_path: PathBuf,
    pub fan_max_speed: i64,
    fan_control: Arc<AtomicBool>,
    fan_curve: Arc<RwLock<BTreeMap<i64, f64>>>,
    power_cap: i64,
}

impl HWMon {
    pub fn new(
        hwmon_path: &PathBuf,
        fan_control_enabled: bool,
        fan_curve: BTreeMap<i64, f64>,
        power_cap: i64,
    ) -> HWMon {
        let fan_max_speed = match fs::read_to_string(hwmon_path.join("fan1_max")) {
            Ok(s) => s.trim().parse::<i64>().unwrap(),
            Err(_) => 0,
        };

        let mut mon = HWMon {
            hwmon_path: hwmon_path.clone(),
            fan_max_speed,
            fan_control: Arc::new(AtomicBool::new(false)),
            fan_curve: Arc::new(RwLock::new(fan_curve)),
            power_cap: 0,
        };

        if fan_control_enabled {
            mon.start_fan_control().unwrap();
        }
        if power_cap == -1 {
            mon.power_cap = mon.get_power_cap();
        }
        else {
            mon.set_power_cap(power_cap).unwrap();
        }

        mon
    }

    pub fn get_fan_speed(&self) -> i64 {
        /*if self.fan_control.load(Ordering::SeqCst) {
            let pwm1 = fs::read_to_string(self.hwmon_path.join("pwm1"))
                .expect("Couldn't read pwm1")
                .trim()
                .parse::<i64>()
                .unwrap();

            self.fan_max_speed / 255 * pwm1
        }
        else {
            fs::read_to_string(self.hwmon_path.join("fan1_input"))
                .expect("Couldn't read fan speed")
                .trim()
                .parse::<i64>()
                .unwrap()
        }*/
        match fs::read_to_string(self.hwmon_path.join("fan1_input")) {
            Ok(a) => a.trim().parse::<i64>().unwrap(),
            _ => 0,
        }
    }

    pub fn get_mem_freq(&self) -> i64 {
        let filename = self.hwmon_path.join("freq2_input");

        match fs::read_to_string(filename) {
            Ok(a) => a
                .trim()
                .parse::<i64>()
                .unwrap()
                / 1000
                / 1000,
            _ => 0,
        }
    }

    pub fn get_gpu_freq(&self) -> i64 {
        let filename = self.hwmon_path.join("freq1_input");

        match fs::read_to_string(filename) {
            Ok(a) => a
                .trim()
                .parse::<i64>()
                .unwrap()
                / 1000
                / 1000,
            _ => 0,
        }
    }

    pub fn get_gpu_temp(&self) -> i64 {
        let filename = self.hwmon_path.join("temp1_input");

        match fs::read_to_string(filename) {
            Ok(a) => a
                .trim()
                .parse::<i64>()
                .unwrap()
                / 1000,
            _ => 0,
        }
    }

    pub fn get_voltage(&self) -> i64 {
        let filename = self.hwmon_path.join("in0_input");
        
        match fs::read_to_string(filename) {
            Ok(a) => a.trim().parse::<i64>().unwrap(),
            Err(_) => 0,
        }
    }

    pub fn get_power_cap_max(&self) -> i64 {
        let filename = self.hwmon_path.join("power1_cap_max");

        match fs::read_to_string(filename) {
            Ok(a) => a
                .trim()
                .parse::<i64>()
                .unwrap()
                / 1000000,
            _ => 0,
        }
    }

    pub fn get_power_cap(&self) -> i64 {
        let filename = self.hwmon_path.join("power1_cap");

        match fs::read_to_string(filename) {
            Ok(a) => a
                .trim()
                .parse::<i64>()
                .unwrap()
                / 1000000,
            _ => 0,
        }
    }

    pub fn set_power_cap(&mut self, cap: i64) -> Result<(), HWMonError> {
        if cap > self.get_power_cap_max() {
            return Err(HWMonError::InvalidValue);
        }
        
        let cap = cap * 1000000;
        log::trace!("setting power cap to {}", cap);

        match fs::write(self.hwmon_path.join("power1_cap"), cap.to_string()) {
            Ok(_) => {
                self.power_cap = cap;
                Ok(())
            },
            Err(_) => Err(HWMonError::PermissionDenied),
        }
    }

    pub fn get_power_avg(&self) -> i64 {
        let filename = self.hwmon_path.join("power1_average");

        match fs::read_to_string(filename) {
            Ok(a) => a
                .trim()
                .parse::<i64>()
                .unwrap()
                / 1000000,
            Err(_) => 0,
        }
    }

    pub fn set_fan_curve(&self, curve: BTreeMap<i64, f64>) {
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
                                        let pwm = (255f64 * (speed_percent / 100f64)) as i64;
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

    pub fn get_fan_control(&self) -> (bool, BTreeMap<i64, f64>) {
        log::trace!("Fan control: {}", self.fan_control.load(Ordering::SeqCst));
        (
            self.fan_control.load(Ordering::SeqCst),
            self.fan_curve.read().unwrap().clone(),
        )
    }
}
