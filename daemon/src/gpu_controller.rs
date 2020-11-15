use crate::config::{GpuConfig, GpuIdentifier};
use crate::hw_mon::{HWMon, HWMonError};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::{collections::BTreeMap, fs};
use vulkano::instance::{Instance, InstanceExtensions, PhysicalDevice};

#[derive(Serialize, Deserialize, Debug)]
pub struct GpuStats {
    pub mem_used: u64,
    pub mem_total: u64,
    pub mem_freq: i32,
    pub gpu_freq: i32,
    pub gpu_temp: i32,
    pub power_avg: i32,
    pub power_cap: i32,
    pub power_cap_max: i32,
    pub fan_speed: i32,
    pub max_fan_speed: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FanControlInfo {
    pub enabled: bool,
    pub curve: BTreeMap<i32, f64>,
}

#[derive(Deserialize, Serialize)]
pub struct GpuController {
    pub hw_path: PathBuf,
    hw_mon: Option<HWMon>,
    pub gpu_info: GpuInfo,
    config: GpuConfig,
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
    pub pci_slot: String,
}

impl GpuController {
    pub fn new(hw_path: PathBuf, config: GpuConfig) -> Self {
        let hw_mon = match fs::read_dir(&hw_path.join("hwmon")) {
            Ok(mut path) => {
                let path = path.next().unwrap().unwrap().path();
                let hw_mon = HWMon::new(
                    &path,
                    config.fan_control_enabled,
                    config.fan_curve.clone(),
                    config.power_cap,
                );
                Some(hw_mon)
            },
            _ => None,
        };

        let mut controller = GpuController {
            hw_path: hw_path.clone(),
            hw_mon,
            gpu_info: Default::default(),
            config,
        };
        controller.gpu_info = controller.get_info();
        log::trace!("{:?}", controller.gpu_info);
        controller
    }

    pub fn load_config(&mut self, config: GpuConfig) {
        self.hw_mon = match fs::read_dir(self.hw_path.join("hwmon")) {
            Ok(mut path) => {
                let path = path.next().unwrap().unwrap().path();
                let hw_mon = HWMon::new(
                    &path,
                    config.fan_control_enabled,
                    config.fan_curve.clone(),
                    config.power_cap,
                );
                Some(hw_mon)
            },
            _ => None,
        };

    }

    pub fn get_config(&self) -> GpuConfig {
        self.config.clone()
    }

    pub fn get_identifier(&self) -> GpuIdentifier {
        GpuIdentifier { pci_id: self.gpu_info.pci_slot.clone(),
                        card_model: self.gpu_info.card_model.clone(),
                        gpu_model: self.gpu_info.gpu_model.clone(), 
                        path: self.hw_path.clone() }

    }

    fn get_info(&self) -> GpuInfo {
        let uevent =
            fs::read_to_string(self.hw_path.join("uevent")).expect("Failed to read uevent");

        let mut driver = String::new();
        let mut vendor_id = String::new();
        let mut model_id = String::new();
        let mut card_vendor_id = String::new();
        let mut card_model_id = String::new();
        let mut pci_slot = String::new();

        for line in uevent.split('\n') {
            let split = line.split('=').collect::<Vec<&str>>();
            match split.get(0).unwrap() {
                &"DRIVER" => driver = split.get(1).unwrap().to_string(),
                &"PCI_ID" => {
                    let ids = split.last().expect("failed to get split").split(':').collect::<Vec<&str>>();
                    vendor_id = ids.get(0).unwrap().to_string();
                    model_id = ids.get(1).unwrap().to_string();
                },
                &"PCI_SUBSYS_ID" => {
                    let ids = split.last().expect("failed to get split").split(':').collect::<Vec<&str>>();
                    card_vendor_id = ids.get(0).unwrap().to_string();
                    card_model_id = ids.get(1).unwrap().to_string();
                },
                &"PCI_SLOT_NAME" => pci_slot = split.get(1).unwrap().to_string(),
                _ => (),
            }
        }

        let vendor = "AMD".to_string();
        let mut model = String::new();
        let mut card_vendor = String::new();
        let mut card_model = String::new();

        let mut full_hwid_list = String::new(); 
        if Path::exists(&PathBuf::from("/usr/share/hwdata/pci.ids")) {
            full_hwid_list = fs::read_to_string("/usr/share/hwdata/pci.ids").unwrap();
        } else if Path::exists(&PathBuf::from("/usr/share/misc/pci.ids")) {
            full_hwid_list = fs::read_to_string("/usr/share/misc/pci.ids").unwrap();
        }

        if !full_hwid_list.is_empty() {
            //some weird space character, don't touch
            let pci_id_line = format!("	{}", model_id.to_lowercase());
            let card_ids_line = format!(
                "		{} {}",
                card_vendor_id.to_lowercase(),
                card_model_id.to_lowercase()
            );
            log::trace!("identifying {} \n {}", pci_id_line, card_ids_line);

            let lines: Vec<&str> = full_hwid_list.split('\n').collect();

            //for line in full_hwid_list.split('\n') {
            for i in 0..lines.len() {
                let line = lines[i];

                if line.len() > card_vendor_id.len() {
                    if line[0..card_vendor_id.len()] == card_vendor_id.to_lowercase() {
                        card_vendor = line.splitn(2, ' ').collect::<Vec<&str>>().last().unwrap()
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
            pci_slot,
        }
    }

    pub fn get_stats(&self) -> GpuStats {
        let mem_total = match fs::read_to_string(self.hw_path.join("mem_info_vram_total")) {
            Ok(a) => a.trim().parse::<u64>().unwrap() / 1024 / 1024,
            Err(_) => 0,
        };

        let mem_used = match fs::read_to_string(self.hw_path.join("mem_info_vram_used")) {
            Ok(a) => a.trim().parse::<u64>().unwrap() / 1024 / 1024,
            Err(_) => 0,
        };

        let (mem_freq, gpu_freq, gpu_temp, power_avg, power_cap, power_cap_max, fan_speed, max_fan_speed) = match &self.hw_mon {
            Some(hw_mon) => (hw_mon.get_mem_freq(), hw_mon.get_gpu_freq(), hw_mon.get_gpu_temp(), hw_mon.get_power_avg(), hw_mon.get_power_cap(), hw_mon.get_power_cap_max(), hw_mon.get_fan_speed(), hw_mon.fan_max_speed),
            None => (0, 0, 0, 0, 0, 0, 0, 0),
        };

        GpuStats {
            mem_total,
            mem_used,
            mem_freq,
            gpu_freq,
            gpu_temp,
            power_avg,
            power_cap,
            power_cap_max,
            fan_speed,
            max_fan_speed,
        }
    }

    pub fn start_fan_control(&mut self) -> Result<(), HWMonError> {
        match &self.hw_mon {
            Some(hw_mon) => {

                match hw_mon.start_fan_control() {
                    Ok(_) => {
                        self.config.fan_control_enabled = true;
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            },
            None => Err(HWMonError::NoHWMon),
        }
    }

    pub fn stop_fan_control(&mut self) -> Result<(), HWMonError> {
        match &self.hw_mon {
            Some(hw_mon) => {
                match hw_mon.stop_fan_control() {
                    Ok(_) => {
                        self.config.fan_control_enabled = false;
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            },
            None => Err(HWMonError::NoHWMon),
        }
    }

    pub fn get_fan_control(&self) -> Result<FanControlInfo, HWMonError> {
        match &self.hw_mon {
            Some(hw_mon) => {
                let control = hw_mon.get_fan_control();
                Ok(FanControlInfo {
                    enabled: control.0,
                    curve: control.1,
                })
            },
            None => Err(HWMonError::NoHWMon),
        }

    }

    pub fn set_fan_curve(&mut self, curve: BTreeMap<i32, f64>) -> Result<(), HWMonError> {
        match &self.hw_mon {
            Some(hw_mon) => {
                hw_mon.set_fan_curve(curve.clone());
                self.config.fan_curve = curve;
                Ok(())
            },
            None => Err(HWMonError::NoHWMon),
        }
    }

    pub fn set_power_cap(&mut self, cap: i32) -> Result<(), HWMonError> {
        match &mut self.hw_mon {
            Some(hw_mon) => {
                hw_mon.set_power_cap(cap).unwrap();
                self.config.power_cap = cap;
                Ok(())
            },
            None => Err(HWMonError::NoHWMon),
        }
    }

    pub fn get_power_cap(&self) -> Result<(i32, i32), HWMonError> {
        match &self.hw_mon {
            Some(hw_mon) => {
                Ok((hw_mon.get_power_cap(), hw_mon.get_power_cap_max()))
            },
            None => Err(HWMonError::NoHWMon),
        }
    }

    fn get_vulkan_info(pci_id: &str) -> VulkanInfo {
        let mut device_name = String::from("Not supported");
        let mut api_version = String::new();
        let mut features = String::new();

        match Instance::new(None, &InstanceExtensions::none(), None) {
            Ok(instance) => {
                for physical in PhysicalDevice::enumerate(&instance) {
                    if format!("{:x}", physical.pci_device_id()) == pci_id.to_lowercase() {
                        api_version = physical.api_version().to_string();
                        device_name = physical.name().to_string();
                        features = format!("{:?}", physical.supported_features());

                    }
                }
            },
            Err(_) => (),
        }
            

        VulkanInfo {
            device_name,
            api_version,
            features,
        }

    }
}
