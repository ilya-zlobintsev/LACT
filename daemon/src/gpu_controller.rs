use crate::config::{GpuConfig, GpuIdentifier};
use crate::hw_mon::{HWMon, HWMonError};
use serde::{Deserialize, Serialize};
use std::{num::ParseIntError, path::{Path, PathBuf}};
use std::{collections::BTreeMap, fs};
use vulkano::instance::{Instance, InstanceExtensions, PhysicalDevice};

#[derive(Serialize, Deserialize, Debug)]
pub enum GpuControllerError {
    NotSupported,
    PermissionDenied,
    UnknownError,
    ParseError,
}

impl From<std::io::Error> for GpuControllerError {
    fn from(err: std::io::Error) -> GpuControllerError {
        match err.kind() {
            std::io::ErrorKind::PermissionDenied => GpuControllerError::PermissionDenied,
            std::io::ErrorKind::NotFound => GpuControllerError::NotSupported,
            _ => GpuControllerError::UnknownError,
        }
    }
}

impl From<ParseIntError> for GpuControllerError {
    fn from(_err: ParseIntError) -> GpuControllerError {
        GpuControllerError::ParseError
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PowerProfile {
    Auto,
    Low,
    High,
}

impl Default for PowerProfile {
    fn default() -> Self { PowerProfile::Auto }
}

impl PowerProfile {
    pub fn from_str(profile: &str) -> Result<Self, GpuControllerError> {
        match profile {
            "auto" | "Automatic" => Ok(PowerProfile::Auto),
            "high" | "Highest Clocks" => Ok(PowerProfile::High),
            "low" | "Lowest Clocks" => Ok(PowerProfile::Low),
            _ => Err(GpuControllerError::ParseError),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            PowerProfile::Auto => "auto".to_string(),
            PowerProfile::High => "high".to_string(),
            PowerProfile::Low => "low".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ClocksTable {
    pub gpu_power_levels: BTreeMap<u32, (i64, i64)>, //<power level, (clockspeed, voltage)>
    pub mem_power_levels: BTreeMap<u32, (i64, i64)>,
    pub gpu_clocks_range: (i64, i64),
    pub mem_clocks_range: (i64, i64),
    pub voltage_range: (i64, i64), //IN MILLIVOLTS
}

impl ClocksTable {
    fn new() -> Self {
        ClocksTable {
            gpu_power_levels: BTreeMap::new(),
            mem_power_levels: BTreeMap::new(),
            gpu_clocks_range: (0, 0),
            mem_clocks_range: (0, 0),
            voltage_range: (0, 0),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GpuStats {
    pub mem_used: u64,
    pub mem_total: u64,
    pub mem_freq: i64,
    pub gpu_freq: i64,
    pub gpu_temp: i64,
    pub power_avg: i64,
    pub power_cap: i64,
    pub power_cap_max: i64,
    pub fan_speed: i64,
    pub max_fan_speed: i64,
    pub voltage: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FanControlInfo {
    pub enabled: bool,
    pub curve: BTreeMap<i64, f64>,
}

#[derive(Deserialize, Serialize)]
pub struct GpuController {
    pub hw_path: PathBuf,
    hw_mon: Option<HWMon>,
    //pub gpu_info: GpuInfo,
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
    pub power_profile: Option<PowerProfile>,
    pub clocks_table: Option<ClocksTable>,
}

impl GpuController {
    pub fn new(hw_path: PathBuf, config: GpuConfig) -> Self {
        let mut controller = GpuController {
            hw_path: hw_path.clone(),
            hw_mon: None,
            config: GpuConfig::new(),
        };

        controller.load_config(config);

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

        #[allow(unused_must_use)]
        {
            self.set_power_profile(config.power_profile.clone());

            for (num, (clockspeed, voltage)) in &config.gpu_power_states {
                self.set_gpu_power_state(*num, *clockspeed, Some(*voltage));
            }
            
            for (num, (clockspeed, voltage)) in &config.vram_power_states {
                self.set_vram_power_state(*num, *clockspeed, Some(*voltage));
            }
        }
    }

    pub fn get_config(&self) -> GpuConfig {
        self.config.clone()
    }

    pub fn get_identifier(&self) -> GpuIdentifier {
        let gpu_info = self.get_info();
        GpuIdentifier { pci_id: gpu_info.pci_slot.clone(),
                        card_model: gpu_info.card_model.clone(),
                        gpu_model: gpu_info.gpu_model.clone(), 
                        path: self.hw_path.clone() }

    }

    pub fn get_info(&self) -> GpuInfo {
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

        let power_profile = match self.get_power_profile() {
            Ok(p) => Some(p),
            Err(_) => None,
        };

        let clocks_table = match self.get_clocks_table() {
            Ok(t) => Some(t),
            Err(_) => None,
        };

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
            power_profile,
            clocks_table,
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

        let (mem_freq, gpu_freq, gpu_temp, power_avg, power_cap, power_cap_max, fan_speed, max_fan_speed, voltage) = match &self.hw_mon {
            Some(hw_mon) => (hw_mon.get_mem_freq(), hw_mon.get_gpu_freq(), hw_mon.get_gpu_temp(), hw_mon.get_power_avg(), hw_mon.get_power_cap(), hw_mon.get_power_cap_max(), hw_mon.get_fan_speed(), hw_mon.fan_max_speed, hw_mon.get_voltage()),
            None => (0, 0, 0, 0, 0, 0, 0, 0, 0),
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
            voltage,
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

    pub fn set_fan_curve(&mut self, curve: BTreeMap<i64, f64>) -> Result<(), HWMonError> {
        match &self.hw_mon {
            Some(hw_mon) => {
                hw_mon.set_fan_curve(curve.clone());
                self.config.fan_curve = curve;
                Ok(())
            },
            None => Err(HWMonError::NoHWMon),
        }
    }

    pub fn set_power_cap(&mut self, cap: i64) -> Result<(), HWMonError> {
        match &mut self.hw_mon {
            Some(hw_mon) => {
                hw_mon.set_power_cap(cap).unwrap();
                self.config.power_cap = cap;
                Ok(())
            },
            None => Err(HWMonError::NoHWMon),
        }
    }

    pub fn get_power_cap(&self) -> Result<(i64, i64), HWMonError> {
        match &self.hw_mon {
            Some(hw_mon) => {
                Ok((hw_mon.get_power_cap(), hw_mon.get_power_cap_max()))
            },
            None => Err(HWMonError::NoHWMon),
        }
    }

    fn get_power_profile(&self) -> Result<PowerProfile, GpuControllerError> {
        match fs::read_to_string(self.hw_path.join("power_dpm_force_performance_level")) {
            Ok(s) => {
                Ok(PowerProfile::from_str(&s.trim()).unwrap())
            },
            Err(_) => Err(GpuControllerError::NotSupported),
        }
    }

    pub fn set_power_profile(&mut self, profile: PowerProfile) -> Result<(), GpuControllerError> {
        match fs::write(self.hw_path.join("power_dpm_force_performance_level"), profile.to_string()) {
            Ok(_) => { 
                self.config.power_profile = profile;
                Ok(())
            },
            Err(_) => Err(GpuControllerError::NotSupported),
        }
    }

    fn get_clocks_table(&self) -> Result<ClocksTable, GpuControllerError> {
        match fs::read_to_string(self.hw_path.join("pp_od_clk_voltage")) {
            Ok(s) => {
                let mut clocks_table = ClocksTable::new();
                let lines: Vec<&str> = s.trim().split("\n").collect();

                log::trace!("Reading clocks table");

                let mut i = 0;
                while i < lines.len() {
                    log::trace!("matching {}", lines[i]);
                    match lines[i] {
                        "OD_SCLK:" => {
                            i += 1;
                            while (lines[i].split_at(2).0 != "OD") && i < lines.len() {
                                let (num, clock, voltage) = GpuController::parse_clock_voltage_line(lines[i])?;

                                clocks_table.gpu_power_levels.insert(num, (clock, voltage));
                                log::trace!("Adding gpu power level {}MHz {}mv", clock, voltage);
                                i += 1;
                            }
                        },
                        "OD_MCLK:" => {
                            i += 1;
                            while (lines[i].split_at(2).0 != "OD") && i < lines.len() {
                                let (num, clock, voltage) = GpuController::parse_clock_voltage_line(lines[i])?;
                                
                                clocks_table.mem_power_levels.insert(num, (clock, voltage));
                                log::trace!("Adding vram power level {}MHz {}mv", clock, voltage);
                                i += 1;
                            }
                        },
                        "OD_RANGE:" => {
                            i += 1;
                            while lines[i].split_at(2).0 != "OD" {
                                let split: Vec<&str> = lines[i].split_whitespace().collect();
                                let name = split[0].replace(":", "");

                                match name.as_ref() {
                                    "SCLK" => {
                                        let min_clock = split[1].replace("MHz", "").parse::<i64>().unwrap();
                                        let max_clock = split[2].replace("MHz", "").parse::<i64>().unwrap();
                                        clocks_table.gpu_clocks_range = (min_clock, max_clock);
                                        log::trace!("Maximum gpu clock: {}", max_clock);
                                    },
                                    "MCLK" => {
                                        let min_clock = split[1].replace("MHz", "").parse::<i64>().unwrap();
                                        let max_clock = split[2].replace("MHz", "").parse::<i64>().unwrap();
                                        clocks_table.mem_clocks_range = (min_clock, max_clock);
                                        log::trace!("Maximum vram clock: {}", max_clock);
                                    },
                                    "VDDC" => {
                                        let min_voltage = split[1].replace("mV", "").parse::<i64>().unwrap();
                                        let max_voltage = split[2].replace("mV", "").parse::<i64>().unwrap();
                                        clocks_table.voltage_range = (min_voltage, max_voltage);
                                        log::trace!("Maximum voltage: {}", max_voltage);
                                    },
                                    _ => (),
                                }

                                i += 1;
                                if i >= lines.len() {
                                    break
                                }
                            }
                        },
                        _ => i += 1,
                    }
                }
                Ok(clocks_table)
            },
            Err(_) => Err(GpuControllerError::NotSupported),
        }
    }

    pub fn set_gpu_power_state(&mut self, num: u32, clockspeed: i64, voltage: Option<i64>) -> Result<(), GpuControllerError> {
        let mut line = format!("s {} {}", num, clockspeed);

        if let Some(voltage) = voltage {
            line.push_str(&format!(" {}", voltage));
        }
        line.push_str("\n");

        log::trace!("Setting gpu power state {}", line);
        log::trace!("Writing {} to pp_od_clk_voltage", line);

        fs::write(self.hw_path.join("pp_od_clk_voltage"), line)?;
        
        self.config.gpu_power_states.insert(num, (clockspeed, voltage.unwrap()));

        Ok(())
    }

    pub fn set_vram_power_state(&mut self, num: u32, clockspeed: i64, voltage: Option<i64>) -> Result<(), GpuControllerError> {
        let mut line = format!("m {} {}", num, clockspeed);

        if let Some(voltage) = voltage {
            line.push_str(&format!(" {}", voltage));
        }
        line.push_str("\n");

        log::trace!("Setting vram power state {}", line);
        log::trace!("Writing {} to pp_od_clk_voltage", line);

        fs::write(self.hw_path.join("pp_od_clk_voltage"), line)?;
        
        self.config.vram_power_states.insert(num, (clockspeed, voltage.unwrap()));

        Ok(())
    }

    pub fn commit_gpu_power_states(&mut self) -> Result<(), GpuControllerError> {
        fs::write(self.hw_path.join("pp_od_clk_voltage"), b"c\n")?;
        Ok(())
    }

    pub fn reset_gpu_power_states(&mut self) -> Result<(), GpuControllerError> {
        fs::write(self.hw_path.join("pp_od_clk_voltage"), b"r\n")?;
        Ok(())
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

    fn parse_clock_voltage_line(line: &str) -> Result<(u32, i64, i64), GpuControllerError> {
        let line = line.to_uppercase();
        let line_parts: Vec<&str> = line.split_whitespace().collect();

        let num: u32 = line_parts.get(0).ok_or_else(|| GpuControllerError::ParseError)?.chars().nth(0).unwrap().to_digit(10).unwrap();
        let clock: i64 = line_parts.get(1).ok_or_else(|| GpuControllerError::ParseError)?.strip_suffix("MHZ").ok_or_else(|| GpuControllerError::ParseError)?.parse()?;
        let voltage: i64 = line_parts.get(2).ok_or_else(|| GpuControllerError::ParseError)?.strip_suffix("MV").ok_or_else(|| GpuControllerError::ParseError)?.parse()?;

        Ok((num, clock, voltage))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_pstate() -> Result<(), GpuControllerError> {
        let mut c = GpuController::new(PathBuf::from("/sys/class/drm/card0/device"), GpuConfig::new());
        c.set_gpu_power_state(7, 1360, None)
    }
}
