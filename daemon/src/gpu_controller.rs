use crate::fan_controller::FanController;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct GpuStats {
    pub mem_used: u64,
    pub mem_total: u64,
}

#[derive(Clone)]
pub struct GpuController {
    hw_path: PathBuf,
    fan_controller: FanController,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GpuInfo {
    pub gpu_vendor: String,
    pub gpu_model: String,
    pub card_model: String,
    pub card_vendor: String,
    pub driver: String,
    pub vbios_version: String,
}

impl GpuController {
    pub fn new(hw_path: &str) -> GpuController {
        GpuController {
            hw_path: PathBuf::from(hw_path),
            fan_controller: FanController::new(hw_path),
        }
    }

    pub fn get_info(self) -> GpuInfo {
        let uevent =
            fs::read_to_string(self.hw_path.join("uevent")).expect("Failed to read uevent");

        //caps for raw values, lowercase for parsed
        let mut driver = String::new();
        let mut VENDOR_ID = String::new();
        let mut MODEL_ID = String::new();
        let mut CARD_VENDOR_ID= String::new();
        let mut CARD_MODEL_ID = String::new();

        for line in uevent.split('\n') {
            let split = line.split('=').collect::<Vec<&str>>();
            match split[0] {
                "DRIVER" => driver = split[1].to_string(),
                "PCI_ID" => {
                    let ids = split[1].split(':').collect::<Vec<&str>>();
                    VENDOR_ID = ids[0].to_string();
                    MODEL_ID = ids[1].to_string();
                },
                "PCI_SUBSYS_ID" => {
                    let ids = split[1].split(':').collect::<Vec<&str>>();
                    CARD_VENDOR_ID = ids[0].to_string();
                    CARD_MODEL_ID = ids[1].to_string();
                },
                _ => (),
            }
        }

        let vendor = "AMD".to_string();
        let mut model = String::new();
        let mut card_vendor = String::new();
        let mut card_model = String::new();

        let full_hwid_list = fs::read_to_string("/usr/share/hwdata/pci.ids").expect("Could not read pci.ids. Perhaps the \"hwids\" package is not installed?");
        
        //some weird space character, don't touch
        let pci_id_line = format!("	{}", MODEL_ID.to_lowercase());
        let card_ids_line = format!("		{} {}", CARD_VENDOR_ID.to_lowercase(), CARD_MODEL_ID.to_lowercase());
        println!("looking for {}", pci_id_line);
        for line in full_hwid_list.split('\n') {
            if line.contains(&pci_id_line) {
                model = line[pci_id_line.len()..].trim_start().to_string();
            }
            if line.contains(&card_ids_line) {
                card_model = line[card_ids_line.len()..].trim_start().to_string();
            }
        }

        let vbios_version = fs::read_to_string(self.hw_path.join("vbios_version"))
            .expect("Failed to read vbios_info");

        GpuInfo {
            gpu_vendor: vendor,
            gpu_model: model,
            card_vendor: String::new(),
            card_model,
            driver,
            vbios_version,
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

        GpuStats {
            mem_total,
            mem_used,
        }
    }
}
