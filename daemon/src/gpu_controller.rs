use crate::hw_mon::{HWMon, HWMonError};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs};
use std::path::PathBuf;
use vulkano::instance::{Instance, InstanceExtensions, PhysicalDevice};
use crate::config::Config;

#[derive(Serialize, Deserialize, Debug)]
pub struct GpuStats {
    pub mem_used: u64,
    pub mem_total: u64,
    pub mem_freq: i32,
    pub gpu_freq: i32,
    pub gpu_temp: i32,
    pub power_avg: i32,
    pub power_max: i32,
    pub fan_speed: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FanControlInfo {
    pub enabled: bool,
    pub curve: BTreeMap<i32, f64>,
}

#[derive(Clone)]
pub struct GpuController {
    hw_path: PathBuf,
    hw_mon: HWMon,
    pub gpu_info: GpuInfo,
    config: Config,
    config_path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct VulkanInfo {
    pub device_name: String,
    pub api_version: String,
    pub features: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GpuInfo {
    pub gpu_vendor: String,
    pub gpu_model: String,
    pub card_model: String,
    pub card_vendor: String,
    pub model_id: String,
    pub vendor_id: String,
    pub driver: String,
    pub vbios_version: String,
    pub vram_size: u64, //in MiB
    pub link_speed: String,
    pub link_width: u8,
    pub vulkan_info: VulkanInfo,
    pub max_fan_speed: i32,
}

impl GpuController {
    pub fn new(hw_path: PathBuf, config: Config, config_path: PathBuf) -> Self {
        let hwmon_path = fs::read_dir(&hw_path.join("hwmon")).unwrap().next().unwrap().unwrap().path();

        let hw_mon = HWMon::new(&hwmon_path, config.fan_control_enabled, config.fan_curve.clone());

        let mut controller = GpuController {
            hw_path: hw_path.clone(),
            hw_mon,
            gpu_info: Default::default(),
            config,
            config_path,
        };
        controller.gpu_info = controller.get_info();
        log::trace!("{:?}", controller.gpu_info);
        controller
    }

    fn get_info(&self) -> GpuInfo {
        let uevent =
            fs::read_to_string(self.hw_path.join("uevent")).expect("Failed to read uevent");

        let mut driver = String::new();
        let mut vendor_id = String::new();
        let mut model_id = String::new();
        let mut card_vendor_id = String::new();
        let mut card_model_id = String::new();

        for line in uevent.split('\n') {
            let split = line.split('=').collect::<Vec<&str>>();
            match split[0] {
                "DRIVER" => driver = split[1].to_string(),
                "PCI_ID" => {
                    let ids = split[1].split(':').collect::<Vec<&str>>();
                    vendor_id = ids[0].to_string();
                    model_id = ids[1].to_string();
                }
                "PCI_SUBSYS_ID" => {
                    let ids = split[1].split(':').collect::<Vec<&str>>();
                    card_vendor_id = ids[0].to_string();
                    card_model_id = ids[1].to_string();
                }
                _ => (),
            }
        }

        let vendor = "AMD".to_string();
        let mut model = String::new();
        let mut card_vendor = String::new();
        let mut card_model = String::new();

        let full_hwid_list = fs::read_to_string("/usr/share/hwdata/pci.ids")
            .expect("Could not read pci.ids. Perhaps the \"hwids\" package is not installed?");

        //some weird space character, don't touch
        let pci_id_line = format!("	{}", model_id.to_lowercase());
        let card_ids_line = format!(
            "		{} {}",
            card_vendor_id.to_lowercase(),
            card_model_id.to_lowercase()
        );

        for line in full_hwid_list.split('\n') {
            if line.len() > card_vendor_id.len() {
                if line[0..card_vendor_id.len()] == card_vendor_id.to_lowercase() {
                    card_vendor = line.splitn(2, ' ').collect::<Vec<&str>>()[1]
                        .trim_start()
                        .to_string();
                }
            }
            if line.contains(&pci_id_line) {
                model = line[pci_id_line.len()..].trim_start().to_string();
            }
            if line.contains(&card_ids_line) {
                card_model = line[card_ids_line.len()..].trim_start().to_string();
            }
        }

        let vbios_version = match fs::read_to_string(self.hw_path.join("vbios_version")) {
            Ok(v) => v,
            Err(_) => "".to_string(),
        }
            .trim()
            .to_string();

        let vram_size = match fs::read_to_string(self.hw_path.join("mem_info_vram_total")) {
            Ok(a) => a.trim().parse::<u64>().unwrap() / 1024 / 1024,
            Err(_) => 0,
        };

        let link_speed = match fs::read_to_string(self.hw_path.join("current_link_speed")) {
            Ok(a) => a.trim().to_string(),
            Err(_) => "".to_string(),
        };

        let link_width = match fs::read_to_string(self.hw_path.join("current_link_width")) {
            Ok(a) => a.trim().parse::<u8>().unwrap(),
            Err(_) => 0,
        };


        let vulkan_info = GpuController::get_vulkan_info(&model_id);
        let max_fan_speed = self.hw_mon.fan_max_speed;

        GpuInfo {
            gpu_vendor: vendor,
            gpu_model: model,
            card_vendor,
            card_model,
            model_id,
            vendor_id,
            driver,
            vbios_version,
            vram_size,
            link_speed,
            link_width,
            vulkan_info,
            max_fan_speed,
        }
    }

    pub fn get_stats(&self) -> GpuStats {
        let mem_total = fs::read_to_string(self.hw_path.join("mem_info_vram_total"))
            .expect("Could not read device file")
            .trim()
            .parse::<u64>()
            .unwrap()
            / 1024
            / 1024;
        let mem_used = fs::read_to_string(self.hw_path.join("mem_info_vram_used"))
            .expect("Could not read device file")
            .trim()
            .parse::<u64>()
            .unwrap()
            / 1024
            / 1024;

        let (mem_freq, gpu_freq) = (self.hw_mon.get_mem_freq(), self.hw_mon.get_gpu_freq());
        let gpu_temp = self.hw_mon.get_gpu_temp();
        let (power_avg, power_max) = (self.hw_mon.get_power_avg(), self.hw_mon.get_power_cap());
        let fan_speed = self.hw_mon.get_fan_speed();

        GpuStats {
            mem_total,
            mem_used,
            mem_freq,
            gpu_freq,
            gpu_temp,
            power_avg,
            power_max,
            fan_speed,
        }
    }

    pub fn start_fan_control(&mut self) -> Result<(), HWMonError> {
        match self.hw_mon.start_fan_control() {
            Ok(_) => {
                self.config.fan_control_enabled = true;
                self.config.save(&self.config_path).expect("Failed to save config");
                Ok(())
            },
            Err(e) => Err(e),
        }
    }

    pub fn stop_fan_control(&mut self) -> Result<(), HWMonError> {
        match self.hw_mon.stop_fan_control() {
            Ok(_) => {
                self.config.fan_control_enabled = false;
                self.config.save(&self.config_path).expect("Failed to save config");
                Ok(())
            },
            Err(e) => Err(e),
        }
    }

    pub fn get_fan_control(&self) -> FanControlInfo {
        let control = self.hw_mon.get_fan_control();

        FanControlInfo {
            enabled: control.0,
            curve: control.1,
        }
    }

    pub fn set_fan_curve(&mut self, curve: BTreeMap<i32, f64>) {
        self.hw_mon.set_fan_curve(curve.clone());
        self.config.fan_curve = curve;
        self.config.save(&self.config_path).expect("Failed to save config");
    }

    fn get_vulkan_info(pci_id: &str) -> VulkanInfo {
        let instance = Instance::new(None, &InstanceExtensions::none(), None)
            .expect("failed to create instance");

        for physical in PhysicalDevice::enumerate(&instance) {
            if format!("{:x}", physical.pci_device_id()) == pci_id.to_lowercase() {
                let api_version = physical.api_version().to_string();
                let device_name = physical.name().to_string();
                let features = format!("{:?}", physical.supported_features());

                return VulkanInfo {
                    device_name,
                    api_version,
                    features,
                };
            }
        }

        VulkanInfo {
            device_name: "Not supported".to_string(),
            api_version: "".to_string(),
            features: "".to_string(),
        }
    }
}
