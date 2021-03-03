use crate::config::{GpuConfig, GpuIdentifier};
use crate::hw_mon::{HWMon, HWMonError};
use pciid_parser::{PciDatabase, VendorData};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::num::ParseIntError;
use std::path::PathBuf;
use vulkano::instance::{Instance, InstanceExtensions, PhysicalDevice};

#[derive(Serialize, Deserialize, Debug)]
pub enum GpuControllerError {
    NotSupported,
    PermissionDenied,
    UnknownError,
    ParseError(String),
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
    fn from(err: ParseIntError) -> GpuControllerError {
        GpuControllerError::ParseError(err.to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PowerProfile {
    Auto,
    Low,
    High,
}

impl Default for PowerProfile {
    fn default() -> Self {
        PowerProfile::Auto
    }
}

impl PowerProfile {
    pub fn from_str(profile: &str) -> Result<Self, GpuControllerError> {
        match profile {
            "auto" | "Automatic" => Ok(PowerProfile::Auto),
            "high" | "Highest Clocks" => Ok(PowerProfile::High),
            "low" | "Lowest Clocks" => Ok(PowerProfile::Low),
            _ => Err(GpuControllerError::ParseError(
                "unrecognized GPU power profile".to_string(),
            )),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClocksTable {
    Old(ClocksTableOld),
    New(ClocksTableNew),
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ClocksTableOld {
    pub gpu_power_levels: BTreeMap<u32, (i64, i64)>, //<power level, (clockspeed, voltage)>
    pub mem_power_levels: BTreeMap<u32, (i64, i64)>,
    pub gpu_clocks_range: (i64, i64),
    pub mem_clocks_range: (i64, i64),
    pub voltage_range: (i64, i64), //IN MILLIVOLTS
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ClocksTableNew {
    pub current_gpu_clocks: (i64, i64),
    pub current_max_mem_clock: i64,
    // pub vddc_curve: [(i64, i64); 3],
    pub gpu_clocks_range: (i64, i64),
    pub mem_clocks_range: (i64, i64),
    // pub voltage_range: (i64, i64), //IN MILLIVOLTS
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GpuStats {
    pub mem_used: Option<u64>,
    pub mem_total: Option<u64>,
    pub mem_freq: Option<i64>,
    pub gpu_freq: Option<i64>,
    pub gpu_temp: Option<i64>,
    pub power_avg: Option<i64>,
    pub power_cap: Option<i64>,
    pub power_cap_max: Option<i64>,
    pub fan_speed: Option<i64>,
    pub max_fan_speed: Option<i64>,
    pub voltage: Option<i64>,
    pub gpu_usage: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FanControlInfo {
    pub enabled: bool,
    pub curve: BTreeMap<i64, f64>,
}
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct VulkanInfo {
    pub device_name: String,
    pub api_version: String,
    pub features: HashMap<String, bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GpuInfo {
    pub vendor_data: VendorData,
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
    pub power_cap: Option<i64>,
    pub power_cap_max: Option<i64>,
}

#[derive(Deserialize, Serialize)]
pub struct GpuController {
    pub hw_path: PathBuf,
    hw_mon: Option<HWMon>,
    gpu_info: GpuInfo,
    config: GpuConfig,
}

impl GpuController {
    pub fn new(hw_path: PathBuf, config: GpuConfig, pci_db: &Option<PciDatabase>) -> Self {
        let mut controller = GpuController {
            hw_path: hw_path.clone(),
            hw_mon: None,
            config: GpuConfig::new(),
            gpu_info: GpuInfo::default(),
        };

        controller.gpu_info = controller.get_info_initial(pci_db);

        controller.load_config(&config);

        controller
    }

    pub fn load_config(&mut self, config: &GpuConfig) {
        self.hw_mon = match fs::read_dir(self.hw_path.join("hwmon")) {
            Ok(mut path) => {
                let path = path.next().unwrap().unwrap().path();
                let hw_mon = HWMon::new(
                    &path,
                    config.fan_control_enabled,
                    config.fan_curve.clone(),
                    Some(config.power_cap),
                );
                Some(hw_mon)
            }
            _ => None,
        };

        #[allow(unused_must_use)]
        {
            self.set_power_profile(config.power_profile.clone());

            self.set_gpu_max_power_state(config.gpu_max_clock, config.gpu_max_voltage);

            self.set_vram_max_clockspeed(config.vram_max_clock);

            self.commit_gpu_power_states();
        }
    }

    pub fn get_config(&self) -> GpuConfig {
        self.config.clone()
    }

    pub fn get_identifier(&self) -> GpuIdentifier {
        let gpu_info = self.get_info();
        GpuIdentifier {
            pci_id: gpu_info.pci_slot.clone(),
            card_model: gpu_info.vendor_data.card_model.clone(),
            gpu_model: gpu_info.vendor_data.gpu_model.clone(),
            path: self.hw_path.clone(),
        }
    }

    pub fn get_info(&self) -> GpuInfo {
        let mut info = self.gpu_info.clone();

        info.power_profile = match self.get_power_profile() {
            Ok(p) => Some(p),
            Err(_) => None,
        };

        info.clocks_table = match self.get_clocks_table() {
            Ok(t) => Some(t),
            Err(_) => None,
        };

        if let Some(hw_mon) = &self.hw_mon {
            info.power_cap = hw_mon.get_power_cap();
            info.power_cap_max = hw_mon.get_power_cap_max();
        }

        info
    }

    fn get_info_initial(&self, pci_db: &Option<PciDatabase>) -> GpuInfo {
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
                    let ids = split
                        .last()
                        .expect("failed to get split")
                        .split(':')
                        .collect::<Vec<&str>>();
                    vendor_id = ids.get(0).unwrap().to_string();
                    model_id = ids.get(1).unwrap().to_string();
                }
                &"PCI_SUBSYS_ID" => {
                    let ids = split
                        .last()
                        .expect("failed to get split")
                        .split(':')
                        .collect::<Vec<&str>>();
                    card_vendor_id = ids.get(0).unwrap().to_string();
                    card_model_id = ids.get(1).unwrap().to_string();
                }
                &"PCI_SLOT_NAME" => pci_slot = split.get(1).unwrap().to_string(),
                _ => (),
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

        let vendor_data = match pci_db {
            Some(db) => {
                match db.get_by_ids(&vendor_id, &model_id, &card_vendor_id, &card_model_id) {
                    Ok(data) => data,
                    Err(_) => VendorData::default(),
                }
            }
            None => match PciDatabase::read() {
                Ok(db) => {
                    match db.get_by_ids(&vendor_id, &model_id, &card_vendor_id, &card_model_id) {
                        Ok(data) => data,
                        Err(_) => VendorData::default(),
                    }
                }
                Err(err) => {
                    println!(
                        "{:?} pci.ids not found! Make sure you have 'hwdata' installed",
                        err
                    );
                    VendorData::default()
                }
            },
        };

        log::info!("Vendor data: {:?}", vendor_data);

        GpuInfo {
            vendor_data,
            model_id,
            vendor_id,
            driver,
            vbios_version,
            vram_size,
            link_speed,
            link_width,
            vulkan_info,
            pci_slot,
            power_profile: None,
            clocks_table: None,
            power_cap: None,
            power_cap_max: None,
        }
    }

    pub fn get_stats(&self) -> Result<GpuStats, HWMonError> {
        let mem_total = match fs::read_to_string(self.hw_path.join("mem_info_vram_total")) {
            Ok(a) => Some(a.trim().parse::<u64>().unwrap() / 1024 / 1024),
            Err(_) => None,
        };

        let mem_used = match fs::read_to_string(self.hw_path.join("mem_info_vram_used")) {
            Ok(a) => Some(a.trim().parse::<u64>().unwrap() / 1024 / 1024),
            Err(_) => None,
        };

        let gpu_usage = match fs::read_to_string(self.hw_path.join("gpu_busy_percent")) {
            Ok(a) => Some(a.trim().parse::<u8>().unwrap()),
            Err(_) => None,
        };

        let (
            mem_freq,
            gpu_freq,
            gpu_temp,
            power_avg,
            power_cap,
            power_cap_max,
            fan_speed,
            max_fan_speed,
            voltage,
        ) = match &self.hw_mon {
            Some(hw_mon) => (
                hw_mon.get_mem_freq(),
                hw_mon.get_gpu_freq(),
                hw_mon.get_gpu_temp(),
                hw_mon.get_power_avg(),
                hw_mon.get_power_cap(),
                hw_mon.get_power_cap_max(),
                hw_mon.get_fan_speed(),
                hw_mon.get_fan_max_speed(),
                hw_mon.get_voltage(),
            ),
            None => return Err(HWMonError::NoHWMon),
        };

        Ok(GpuStats {
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
            gpu_usage,
        })
    }

    pub fn start_fan_control(&mut self) -> Result<(), HWMonError> {
        match &self.hw_mon {
            Some(hw_mon) => match hw_mon.start_fan_control() {
                Ok(_) => {
                    self.config.fan_control_enabled = true;
                    Ok(())
                }
                Err(e) => Err(e),
            },
            None => Err(HWMonError::NoHWMon),
        }
    }

    pub fn stop_fan_control(&mut self) -> Result<(), HWMonError> {
        match &self.hw_mon {
            Some(hw_mon) => match hw_mon.stop_fan_control() {
                Ok(_) => {
                    self.config.fan_control_enabled = false;
                    Ok(())
                }
                Err(e) => Err(e),
            },
            None => Err(HWMonError::NoHWMon),
        }
    }

    pub fn get_fan_control(&self) -> Result<FanControlInfo, HWMonError> {
        match &self.hw_mon {
            Some(hw_mon) => match hw_mon.get_fan_speed() {
                Some(_) => {
                    let control = hw_mon.get_fan_control();
                    Ok(FanControlInfo {
                        enabled: control.0,
                        curve: control.1,
                    })
                }
                None => Err(HWMonError::Unsupported),
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
            }
            None => Err(HWMonError::NoHWMon),
        }
    }

    pub fn set_power_cap(&mut self, cap: i64) -> Result<(), HWMonError> {
        match &mut self.hw_mon {
            Some(hw_mon) => {
                hw_mon.set_power_cap(cap).unwrap();
                self.config.power_cap = cap;
                Ok(())
            }
            None => Err(HWMonError::NoHWMon),
        }
    }

    pub fn get_power_cap(&self) -> Result<(i64, i64), HWMonError> {
        match &self.hw_mon {
            Some(hw_mon) => {
                let min = hw_mon
                    .get_power_cap()
                    .ok_or_else(|| HWMonError::Unsupported)?;
                let max = hw_mon
                    .get_power_cap_max()
                    .ok_or_else(|| HWMonError::Unsupported)?;

                Ok((min, max))
            }
            None => Err(HWMonError::NoHWMon),
        }
    }

    fn get_power_profile(&self) -> Result<PowerProfile, GpuControllerError> {
        match fs::read_to_string(self.hw_path.join("power_dpm_force_performance_level")) {
            Ok(s) => Ok(PowerProfile::from_str(&s.trim()).unwrap()),
            Err(_) => Err(GpuControllerError::NotSupported),
        }
    }

    pub fn set_power_profile(&mut self, profile: PowerProfile) -> Result<(), GpuControllerError> {
        match fs::write(
            self.hw_path.join("power_dpm_force_performance_level"),
            profile.to_string(),
        ) {
            Ok(_) => {
                self.config.power_profile = profile;
                Ok(())
            }
            Err(_) => Err(GpuControllerError::NotSupported),
        }
    }

    fn get_clocks_table(&self) -> Result<ClocksTable, GpuControllerError> {
        match fs::read_to_string(self.hw_path.join("pp_od_clk_voltage")) {
            Ok(table) => Self::parse_clocks_table(&table),
            Err(_) => Err(GpuControllerError::NotSupported),
        }
    }

    fn parse_clocks_table(table: &str) -> Result<ClocksTable, GpuControllerError> {
        if table.contains("CURVE") {
            Ok(ClocksTable::New(Self::parse_clocks_table_new(table)?))
        } else {
            Ok(ClocksTable::Old(Self::parse_clocks_table_old(table)?))
        }
    }

    fn parse_clocks_table_old(table: &str) -> Result<ClocksTableOld, GpuControllerError> {
        let mut clocks_table = ClocksTableOld::default();

        let mut lines_iter = table.trim().split("\n").into_iter();

        log::trace!("Reading clocks table");

        while let Some(line) = lines_iter.next() {
            let line = line.trim();
            log::trace!("Parsing line {}", line);

            match line {
                "OD_SCLK:" | "OD_MCLK:" => {
                    let is_vram = match line {
                        "OD_SCLK:" => false,
                        "OD_MCLK:" => true,
                        _ => unreachable!(),
                    };

                    log::trace!("Parsing clock levels");

                    // If `next()` is used on the main iterator directly, it will consume the `OD_MCLK:` aswell,
                    // which means the outer loop won't recognize that the next lines are of a different clock type.
                    // Thus, it is better to count how many lines were of the clock levels and then substract that amount from the main iterator.
                    let mut i = 0;
                    let mut lines = lines_iter.clone();

                    while let Some(line) = lines.next() {
                        let line = line.trim();
                        log::trace!("Parsing power level line {}", line);

                        // Probably shouldn't unwrap, will fail on empty lines in clocks table
                        if let Some(_) = line.chars().next().unwrap().to_digit(10) {
                            let (num, clock, voltage) =
                                GpuController::parse_clock_voltage_line(line)?;

                            log::trace!("Power level {}: {}MHz {}mV", num, clock, voltage);

                            if is_vram {
                                clocks_table.mem_power_levels.insert(num, (clock, voltage));
                            } else {
                                clocks_table.gpu_power_levels.insert(num, (clock, voltage));
                            }

                            i += 1;
                        } else {
                            // Probably a better way to do this
                            for _ in 0..i {
                                lines_iter.next().unwrap();
                            }
                            log::trace!("Finished reading clock levels");
                            break;
                        }
                    }
                }
                "OD_RANGE:" => {
                    log::trace!("Parsing clock and voltage ranges");

                    while let Some(line) = lines_iter.next() {
                        let mut split = line.split_whitespace();

                        let name = split.next().ok_or_else(|| {
                            GpuControllerError::ParseError("failed to get range name".to_string())
                        })?;
                        let min = split.next().ok_or_else(|| {
                            GpuControllerError::ParseError(
                                "failed to get range minimal value".to_string(),
                            )
                        })?;
                        let max = split.next().ok_or_else(|| {
                            GpuControllerError::ParseError(
                                "failed to get range maximum value".to_string(),
                            )
                        })?;

                        match name {
                            "SCLK:" => {
                                let min_clock: i64 = min.replace("MHz", "").parse()?;
                                let max_clock: i64 = max.replace("MHz", "").parse()?;

                                clocks_table.gpu_clocks_range = (min_clock, max_clock);
                            }
                            "MCLK:" => {
                                let min_clock: i64 = min.replace("MHz", "").parse()?;
                                let max_clock: i64 = max.replace("MHz", "").parse()?;

                                clocks_table.mem_clocks_range = (min_clock, max_clock);
                            }
                            "VDDC:" => {
                                let min_voltage: i64 = min.replace("mV", "").parse()?;
                                let max_voltage: i64 = max.replace("mV", "").parse()?;

                                clocks_table.voltage_range = (min_voltage, max_voltage);
                            }
                            _ => {
                                return Err(GpuControllerError::ParseError(
                                    "unrecognized voltage range type".to_string(),
                                ))
                            }
                        }
                    }
                }
                _ => {
                    return Err(GpuControllerError::ParseError(
                        "unrecognized line type".to_string(),
                    ))
                }
            }
        }

        log::trace!("Successfully parsed the clocks table");
        Ok(clocks_table)
    }

    fn parse_clocks_table_new(table: &str) -> Result<ClocksTableNew, GpuControllerError> {
        log::trace!("Detected clocks table format for Vega20 or newer");

        let mut clocks_table = ClocksTableNew::default();

        let mut lines_iter = table.trim().split("\n").into_iter();

        log::trace!("Reading clocks table");

        while let Some(line) = &lines_iter.next() {
            let line = line.trim();
            log::trace!("Parsing line {}", line);

            match line {
                "OD_SCLK:" => {
                    let min_clock_line = lines_iter
                        .next()
                        .ok_or_else(|| {
                            GpuControllerError::ParseError(
                                "unexpeceted clocks file end".to_string(),
                            )
                        })?
                        .trim()
                        .to_lowercase();

                    let min_clock: i64 = min_clock_line
                        .strip_prefix("0:")
                        .ok_or_else(|| {
                            GpuControllerError::ParseError(format!(
                                "invalid clock line prefix in {}",
                                min_clock_line
                            ))
                        })?
                        .strip_suffix("mhz")
                        .ok_or_else(|| {
                            GpuControllerError::ParseError(format!(
                                "invalid clock line suffix in {}",
                                min_clock_line
                            ))
                        })?
                        .trim()
                        .parse()?;

                    let max_clock_line = lines_iter
                        .next()
                        .ok_or_else(|| {
                            GpuControllerError::ParseError(
                                "unexpeceted clocks file end".to_string(),
                            )
                        })?
                        .trim()
                        .to_lowercase();

                    let max_clock: i64 = max_clock_line
                        .strip_prefix("1:")
                        .ok_or_else(|| {
                            GpuControllerError::ParseError(format!(
                                "invalid clock line prefix in {}",
                                min_clock_line
                            ))
                        })?
                        .strip_suffix("mhz")
                        .ok_or_else(|| {
                            GpuControllerError::ParseError(format!(
                                "invalid clock line suffix in {}",
                                min_clock_line
                            ))
                        })?
                        .trim()
                        .parse()?;

                    clocks_table.current_gpu_clocks = (min_clock, max_clock);
                }
                "OD_MCLK:" => {
                    let max_clock_line = lines_iter
                        .next()
                        .ok_or_else(|| {
                            GpuControllerError::ParseError("unexpected clocks file end".to_string())
                        })?
                        .trim()
                        .to_lowercase();

                    let max_clock = max_clock_line
                        .strip_prefix("1:")
                        .ok_or_else(|| {
                            GpuControllerError::ParseError(format!(
                                "invalid clock line prefix in {}",
                                max_clock_line
                            ))
                        })?
                        .strip_suffix("mhz")
                        .ok_or_else(|| {
                            GpuControllerError::ParseError(format!(
                                "invalid clock line suffix in {}",
                                max_clock_line
                            ))
                        })?
                        .trim()
                        .parse()?;

                    clocks_table.current_max_mem_clock = max_clock;
                }
                "OD_RANGE:" => {
                    while let Some(line) = &lines_iter.next() {
                        let line = line.trim();
                        log::trace!("Parsing OD_RANGE line {}", &line);

                        match &line[..5] {
                            "SCLK:" => {
                                let mut split = line.split_whitespace();

                                // Skips the 'SCLK'
                                split.next().unwrap();

                                let min_clock = split
                                    .next()
                                    .unwrap()
                                    .strip_suffix("Mhz")
                                    .ok_or_else(|| {
                                        GpuControllerError::ParseError("missing suffix".to_string())
                                    })?
                                    .parse()?;

                                let max_clock = split
                                    .next()
                                    .unwrap()
                                    .strip_suffix("Mhz")
                                    .ok_or_else(|| {
                                        GpuControllerError::ParseError("missing suffix".to_string())
                                    })?
                                    .parse()?;

                                clocks_table.gpu_clocks_range = (min_clock, max_clock);
                            }
                            "MCLK:" => {
                                let mut split = line.split_whitespace();

                                // Skips the 'SCLK'
                                split.next().unwrap();

                                let min_clock = split
                                    .next()
                                    .unwrap()
                                    .strip_suffix("Mhz")
                                    .ok_or_else(|| {
                                        GpuControllerError::ParseError("missing suffix".to_string())
                                    })?
                                    .parse()?;

                                let max_clock = split
                                    .next()
                                    .unwrap()
                                    .strip_suffix("Mhz")
                                    .ok_or_else(|| {
                                        GpuControllerError::ParseError("missing suffix".to_string())
                                    })?
                                    .parse()?;

                                clocks_table.mem_clocks_range = (min_clock, max_clock);
                            }
                            _ => {
                                log::trace!("OD_RANGE ended");
                                break;
                            }
                        }
                    }
                }
                _ => {
                    log::trace!("Skipping line");
                    continue;
                }
            }
        }

        Ok(clocks_table)
    }

    /*pub fn set_gpu_power_state(
        &mut self,
        num: u32,
        clockspeed: i64,
        voltage: Option<i64>,
    ) -> Result<(), GpuControllerError> {
        let mut line = format!("s {} {}", num, clockspeed);

        if let Some(voltage) = voltage {
            line.push_str(&format!(" {}", voltage));
        }
        line.push_str("\n");

        log::info!("Setting gpu power state {}", line);
        log::info!("Writing {} to pp_od_clk_voltage", line);

        fs::write(self.hw_path.join("pp_od_clk_voltage"), line)?;

        self.config
            .gpu_power_states
            .insert(num, (clockspeed, voltage.unwrap()));

        Ok(())
    }*/

    pub fn set_gpu_max_power_state(
        &mut self,
        clockspeed: i64,
        voltage: Option<i64>,
    ) -> Result<(), GpuControllerError> {
        match self.get_clocks_table()? {
            ClocksTable::Old(clocks_table) => {
                let profile = { clocks_table.gpu_power_levels.iter().next_back().unwrap().0 };

                let mut line = format!("s {} {}", profile, clockspeed);

                if let Some(voltage) = voltage {
                    line.push_str(&format!(" {}", voltage));
                }
                line.push_str("\n");

                log::info!("Writing {} to pp_od_clk_voltage", line);

                fs::write(self.hw_path.join("pp_od_clk_voltage"), line)?;

                self.config.gpu_max_clock = clockspeed;
                self.config.gpu_max_voltage = voltage;
            }
            ClocksTable::New(_) => {
                let s_line = format!("s 1 {}\n", clockspeed);

                fs::write(self.hw_path.join("pp_od_clk_voltage"), s_line)?;

                if let Some(voltage) = voltage {
                    let vc_line = format!("vc 2 {} {}\n", clockspeed, voltage);

                    fs::write(self.hw_path.join("pp_od_clk_voltage"), vc_line)?;
                }
            }
        }

        Ok(())
    }

    pub fn set_vram_max_clockspeed(&mut self, clockspeed: i64) -> Result<(), GpuControllerError> {
        match self.get_clocks_table()? {
            ClocksTable::Old(clocks_table) => {
                let (profile, voltage) = {
                    let power_level = clocks_table.mem_power_levels.iter().next_back().unwrap();
                    log::info!("Using mem power level {:?}", power_level);
                    (power_level.0, power_level.1 .1)
                };

                let line = format!("m {} {} {}\n", profile, clockspeed, voltage);

                log::info!("Writing {} to pp_od_clk_voltage", line);

                fs::write(self.hw_path.join("pp_od_clk_voltage"), line)?;

                self.config.vram_max_clock = clockspeed;
            }
            ClocksTable::New(_) => {
                let s_line = format!("m 1 {}\n", clockspeed);

                fs::write(self.hw_path.join("pp_od_clk_voltage"), s_line)?;
            }
        }

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
        let mut features = HashMap::new();

        match Instance::new(None, &InstanceExtensions::none(), None) {
            Ok(instance) => {
                for physical in PhysicalDevice::enumerate(&instance) {
                    if format!("{:x}", physical.pci_device_id()) == pci_id.to_lowercase() {
                        api_version = physical.api_version().to_string();
                        device_name = physical.name().to_string();

                        let features_string = format!("{:?}", physical.supported_features());
                        let features_string = features_string
                            .replace("Features", "")
                            .replace("{", "")
                            .replace("}", "");

                        for feature in features_string.split(',') {
                            // let (name, supported) = feature.split_once(':').unwrap(); Use this once it's in stable
                            let mut split = feature.split(':');
                            let name = split.next().unwrap().trim();
                            let supported = split.next().unwrap().trim();

                            let supported: bool = supported.parse().unwrap();

                            features.insert(name.to_string(), supported);
                        }

                        break;
                    }
                }
            }
            Err(_) => (),
        }

        VulkanInfo {
            device_name,
            api_version,
            features,
        }
    }

    fn parse_clock_voltage_line(line: &str) -> Result<(u32, i64, i64), GpuControllerError> {
        log::trace!("Parsing line {}", line);

        let line = line.to_uppercase();
        let line_parts: Vec<&str> = line.split_whitespace().collect();

        let num: u32 = line_parts
            .get(0)
            .ok_or_else(|| {
                GpuControllerError::ParseError("failed to read the power level number".to_string())
            })?
            .chars()
            .nth(0)
            .unwrap()
            .to_digit(10)
            .unwrap();
        let clock: i64 = line_parts
            .get(1)
            .ok_or_else(|| {
                GpuControllerError::ParseError("failed to read the clockspeed".to_string())
            })?
            .strip_suffix("MHZ")
            .ok_or_else(|| GpuControllerError::ParseError("failed to strip \"MHZ\"".to_string()))?
            .parse()?;
        let voltage: i64 = line_parts
            .get(2)
            .ok_or_else(|| {
                GpuControllerError::ParseError("failed to read the voltage".to_string())
            })?
            .strip_suffix("MV")
            .ok_or_else(|| GpuControllerError::ParseError("failed to strip \"mV\"".to_string()))?
            .parse()?;

        Ok((num, clock, voltage))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    // pp_od_clk_voltage taken from an RX 580
    #[test]
    fn parse_clocks_table_polaris() {
        init();

        let pp_od_clk_voltage = r#"
            OD_SCLK:
            0:        300MHz        750mV
            1:        600MHz        769mV
            2:        900MHz        912mV
            3:       1145MHz       1125mV
            4:       1215MHz       1150mV
            5:       1257MHz       1150mV
            6:       1300MHz       1150mV
            7:       1366MHz       1150mV
            OD_MCLK:
            0:        300MHz        750mV
            1:       1000MHz        825mV
            2:       1750MHz        975mV
            OD_RANGE:
            SCLK:     300MHz       2000MHz
            MCLK:     300MHz       2250MHz
            VDDC:     750mV        1200mV"#;

        let clocks_table = GpuController::parse_clocks_table(pp_od_clk_voltage).unwrap();

        log::trace!("{:?}", clocks_table);
    }

    // pp_od_clk_voltage taken from a Vega 56
    #[test]
    fn parse_clocks_table_vega() {
        init();

        let pp_od_clk_voltage = r#"
            OD_SCLK:
            0:        852Mhz        800mV
            1:        991Mhz        900mV
            2:       1138Mhz        950mV
            3:       1269Mhz       1000mV
            4:       1312Mhz       1050mV
            5:       1474Mhz       1100mV
            6:       1538Mhz       1150mV
            7:       1590Mhz       1157mV
            OD_MCLK:
            0:        167Mhz        800mV
            1:        500Mhz        800mV
            2:        700Mhz        900mV
            3:        900Mhz        950mV
            OD_RANGE:
            SCLK:     852MHz       2400MHz
            MCLK:     167MHz       1500MHz
            VDDC:     800mV        1200mV"#;

        let clocks_table = GpuController::parse_clocks_table(pp_od_clk_voltage).unwrap();

        log::trace!("{:?}", clocks_table);
    }

    // pp_od_clk_voltage taken from an RX 5700 XT
    #[test]
    fn parse_clocks_table_navi() {
        init();

        let pp_od_clk_voltage = r#"
            OD_SCLK:
            0: 800Mhz
            1: 2100Mhz
            OD_MCLK:
            1: 875MHz
            OD_VDDC_CURVE:
            0: 800MHz 711mV
            1: 1450MHz 801mV
            2: 2100MHz 1191mV
            OD_RANGE:
            SCLK:     800Mhz       2150Mhz
            MCLK:     625Mhz        950Mhz
            VDDC_CURVE_SCLK[0]:     800Mhz       2150Mhz
            VDDC_CURVE_VOLT[0]:     750mV        1200mV
            VDDC_CURVE_SCLK[1]:     800Mhz       2150Mhz
            VDDC_CURVE_VOLT[1]:     750mV        1200mV
            VDDC_CURVE_SCLK[2]:     800Mhz       2150Mhz
            VDDC_CURVE_VOLT[2]:     750mV        1200mV
        "#;

        let clocks_table = GpuController::parse_clocks_table(pp_od_clk_voltage).unwrap();

        log::info!("{:?}", clocks_table);
    }
}
