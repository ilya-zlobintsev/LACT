use crate::hw_mon::HWMon;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs};
use std::path::PathBuf;
use vulkano::instance::{Instance, InstanceExtensions, PhysicalDevice};

#[derive(Serialize, Deserialize, Debug)]
pub struct GpuStats {
    pub mem_used: u64,
    pub mem_total: u64,
    pub mem_freq: i32,
    pub gpu_freq: i32,
    pub gpu_temp: i32,
}

#[derive(Clone)]
pub struct GpuController {
    hw_path: PathBuf,
    hw_mon: HWMon,
    pub gpu_info: GpuInfo,
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
}

impl GpuController {
    pub fn new(hw_path: PathBuf) -> Self {
        let mut controller = GpuController {
            hw_path: hw_path.clone(),
            hw_mon: HWMon::new(&hw_path.join("hwmon/hwmon0")),
            gpu_info: Default::default(),
        };
        controller.gpu_info = controller.get_info();
        println!("{:?}", controller.gpu_info);
        controller
    }

    fn get_info(&self) -> GpuInfo {
        let uevent =
            fs::read_to_string(self.hw_path.join("uevent")).expect("Failed to read uevent");

        let mut driver = String::new();
        let mut vendor_id = String::new();
        let mut model_id = String::new();
        let mut CARD_VENDOR_ID = String::new();
        let mut CARD_MODEL_ID = String::new();

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
                    CARD_VENDOR_ID = ids[0].to_string();
                    CARD_MODEL_ID = ids[1].to_string();
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
            CARD_VENDOR_ID.to_lowercase(),
            CARD_MODEL_ID.to_lowercase()
        );

        for line in full_hwid_list.split('\n') {
            if line.len() > CARD_VENDOR_ID.len() {
                if line[0..CARD_VENDOR_ID.len()] == CARD_VENDOR_ID.to_lowercase() {
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

        let vbios_version = fs::read_to_string(self.hw_path.join("vbios_version"))
            .expect("Failed to read vbios_info")
            .trim()
            .to_string();

        let vram_size = fs::read_to_string(self.hw_path.join("mem_info_vram_total"))
            .expect("Failed to read mem size")
            .trim()
            .parse::<u64>()
            .unwrap()
            / 1024
            / 1024;

        let link_speed = fs::read_to_string(self.hw_path.join("current_link_speed"))
            .expect("Failed to read link speed")
            .trim()
            .to_string();

        let link_width = fs::read_to_string(self.hw_path.join("current_link_width"))
            .expect("Failed to read link width")
            .trim()
            .parse::<u8>()
            .unwrap();

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
        }
    }

    pub fn get_stats(self) -> GpuStats {
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


        GpuStats {
            mem_total,
            mem_used,
            mem_freq,
            gpu_freq,
            gpu_temp,
        }
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
