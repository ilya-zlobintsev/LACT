use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum VendorDataError {
    MissingIdsFile,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct VendorData {
    pub gpu_vendor: Option<String>,
    pub gpu_model: Option<String>,
    pub card_vendor: Option<String>,
    pub card_model: Option<String>,
}

impl VendorData {
    pub fn from_ids(
        vendor_id: &str,
        model_id: &str,
        subsys_vendor_id: &str,
        subsys_model_id: &str,
    ) -> Result<VendorData, VendorDataError> {
        let vendor_id = vendor_id.to_lowercase();
        let model_id = model_id.to_lowercase();
        let subsys_vendor_id = subsys_vendor_id.to_lowercase();
        let subsys_model_id = subsys_model_id.to_lowercase();

        match PciDatabase::read() {
            Ok(db) => {
                let mut gpu_vendor = None;
                let mut gpu_model = None;
                let mut card_vendor = None;
                let mut card_model = None;

                log::trace!("Seacrhing vendor {}", vendor_id);
                if let Some(vendor) = db.vendors.get(&vendor_id) {
                    log::trace!("Found vendor {}", vendor.name);
                    gpu_vendor = Some(vendor.name.clone());
                    
                    log::trace!("Searching device {}", model_id);
                    if let Some(model) = vendor.devices.get(&model_id) {
                        log::trace!("Found device {}", model.name);
                        gpu_model = Some(model.name.clone());
                        
                        log::trace!("Searching subdevice {} {}", subsys_vendor_id, subsys_model_id);
                        if let Some(subvendor) = db.vendors.get(&subsys_vendor_id) {
                            log::trace!("Found subvendor {}", subvendor.name);
                            card_vendor = Some(subvendor.name.clone());
                        }
                        if let Some(subdevice) = model.subdevices.get(&(subsys_vendor_id, subsys_model_id)) {
                            log::trace!("Found subdevice {}", subdevice);
                            card_model = Some(subdevice.to_owned());
                        }
                    }
                }
                
                Ok(VendorData { gpu_vendor, gpu_model, card_vendor, card_model })
            },
            Err(_) => Err(VendorDataError::MissingIdsFile)
        }
    }
}

#[derive(Debug)]
enum PciDatabaseError {
    FileNotFound,
}

struct PciDatabase {
    pub vendors: HashMap<String, PciVendor>,
}

impl PciDatabase {
    pub fn read<'a>() -> Result<Self, PciDatabaseError> {
        let _ = env_logger::builder().is_test(true).try_init();

        match Self::read_pci_db() {
            Some(pci_ids) => {
                log::trace!("Parsing pci.ids");

                let mut vendors: HashMap<String, PciVendor> = HashMap::new();

                let mut lines = pci_ids.split("\n").into_iter();

                let mut current_vendor_id: Option<String> = None;
                let mut current_device_id: Option<String> = None;

                while let Some(line) = lines.next() {
                    if line.starts_with("#") | line.is_empty() {
                        continue;
                    } else if line.starts_with("\t\t") {
                        let mut split = line.split_whitespace();

                        let vendor_id = split.next().unwrap().to_owned();
                        let device_id = split.next().unwrap().to_owned();
                        let name = split.collect::<Vec<&str>>().join(" ");

                        if let Some(current_vendor_id) = &current_vendor_id {
                            if let Some(current_device_id) = &current_device_id {
                                vendors
                                    .get_mut(current_vendor_id)
                                    .unwrap()
                                    .devices
                                    .get_mut(current_device_id)
                                    .unwrap()
                                    .subdevices
                                    .insert((vendor_id, device_id), name);
                            }
                        }
                    } else if line.starts_with("\t") {
                        let mut split = line.split_whitespace();

                        let id = split.next().unwrap().to_owned();
                        let name = split.collect::<Vec<&str>>().join(" ");

                        let device = PciDevice::new(name);

                        current_device_id = Some(id.clone());

                        if let Some(current_vendor_id) = &current_vendor_id {
                            vendors
                                .get_mut(current_vendor_id)
                                .unwrap()
                                .devices
                                .insert(id, device);
                        }
                    } else {
                        let mut split = line.split_whitespace();

                        let id = split.next().unwrap().to_owned();
                        let name = split.collect::<Vec<&str>>().join(" ");
                        
                        current_vendor_id = Some(id.clone());
                        
                        let vendor = PciVendor::new(name);
                        vendors.insert(id, vendor);
                    }
                }
                Ok(PciDatabase { vendors })
            }
            None => Err(PciDatabaseError::FileNotFound),
        }
    }

    fn read_pci_db() -> Option<String> {
        let paths = ["/usr/share/hwdata/pci.ids", "/usr/share/misc/pci.ids"];

        if let Some(path) = paths.iter().find(|path| Path::exists(&PathBuf::from(path))) {
            let all_ids = fs::read_to_string(path).unwrap();

            Some(all_ids)
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct PciVendor {
    pub name: String,
    pub devices: HashMap<String, PciDevice>,
}

impl PciVendor {
    pub fn new(name: String) -> Self {
        PciVendor {
            name,
            devices: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
struct PciDevice {
    pub name: String,
    pub subdevices: HashMap<(String, String), String>, // <(vendor_id, device_id), name>
}

impl PciDevice {
    pub fn new(name: String) -> Self {
        PciDevice {
            name,
            subdevices: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn parse_polaris() {
        init();
        let data = VendorData::from_ids("1002", "67DF", "1DA2", "E387").unwrap();
        
        assert_eq!(data.gpu_vendor, Some("Advanced Micro Devices, Inc. [AMD/ATI]".to_string()));
        assert_eq!(data.gpu_model, Some("Ellesmere [Radeon RX 470/480/570/570X/580/580X/590]".to_string()));
        assert_eq!(data.card_vendor, Some("Sapphire Technology Limited".to_string()));
        assert_eq!(data.card_model, Some("Radeon RX 570 Pulse 4GB".to_string()));
    }
    
    #[test]
    fn parse_vega() {
        let data = VendorData::from_ids("1002", "687F", "1043", "0555").unwrap();
        
        assert_eq!(data.gpu_vendor, Some("Advanced Micro Devices, Inc. [AMD/ATI]".to_string()));
        assert_eq!(data.gpu_model, Some("Vega 10 XL/XT [Radeon RX Vega 56/64]".to_string()));
        assert_eq!(data.card_vendor, Some("ASUSTeK Computer Inc.".to_string()));
        assert_eq!(data.card_model, None);
    }
}
