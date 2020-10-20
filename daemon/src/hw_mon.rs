use glob::glob;
use std::{collections::HashMap, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct HWMon {
    hwmon_path: PathBuf,
    freqs: HashMap<String, u8>,
}

impl HWMon {
    pub fn new(hwmon_path: &PathBuf) -> HWMon {
        let mut freqs: HashMap<String, u8> = HashMap::new();
        for entry in glob(&format!("{}/freq*_label", hwmon_path.to_str().unwrap())).expect("Couldnt read glob pattern")
        {
            let entry = entry.unwrap();
            let filename = entry.file_name().unwrap().to_str().unwrap();
            let index = filename.chars().nth(4).unwrap().to_digit(10).unwrap() as u8;
            let label = fs::read_to_string(hwmon_path.join(&format!("freq{}_label", index)))
                .unwrap()
                .trim()
                .to_string();
            freqs.insert(label, index);
        }
        println!("{:?}", freqs);

        HWMon {
            hwmon_path: hwmon_path.clone(),
            freqs,
        }
    }

    pub fn get_fan_speed(&self) -> i32 {
        fs::read_to_string(self.hwmon_path.join("fan1_input"))
            .expect("Couldn't read fan speed")
            .parse::<i32>()
            .unwrap()
    }

    pub fn get_mem_freq(&self) -> i32 {
        let filename = self.hwmon_path.join(format!("freq{}_input", self.freqs.get("mclk").unwrap()));

        fs::read_to_string(filename).unwrap().trim().parse::<i32>().unwrap() / 1000 / 1000
    }

    pub fn get_gpu_freq(&self) -> i32 {
        let filename = self.hwmon_path.join(format!("freq{}_input", self.freqs.get("sclk").unwrap()));

        fs::read_to_string(filename).unwrap().trim().parse::<i32>().unwrap() / 1000 / 1000
    }
}
