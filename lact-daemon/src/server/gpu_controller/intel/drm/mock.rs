use super::{DrmProvider, VramInfo};
use lact_schema::{DeviceStats, DrmInfo, IntelDrmInfo};
use serde::de::DeserializeOwned;
use std::path::Path;

pub struct MockDrmProvider {
    info: DrmInfo,
    stats: DeviceStats,
}

impl MockDrmProvider {
    pub fn new(sysfs: &Path) -> Option<Self> {
        Some(Self {
            info: read_json(sysfs, "drm_info.json")?,
            stats: read_json(sysfs, "stats.json")?,
        })
    }
}

fn read_json<T: DeserializeOwned>(sysfs: &Path, name: &str) -> Option<T> {
    std::fs::read_to_string(sysfs.join(name))
        .ok()
        .and_then(|contents| serde_json::from_str::<T>(&contents).ok())
}

impl DrmProvider for MockDrmProvider {
    fn get_intel_info(&self) -> IntelDrmInfo {
        self.info.intel.clone()
    }

    fn get_vram_info(&self) -> VramInfo {
        VramInfo {
            total: self.stats.vram.total.unwrap_or(0),
            used: self.stats.vram.used.unwrap_or(0),
            mem_info: self.info.memory_info.clone().unwrap_or_default(),
        }
    }
}
