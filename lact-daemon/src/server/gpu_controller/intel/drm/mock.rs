use super::{DrmProvider, VramInfo};
use crate::server::handler::SnapshotDeviceInfo;
use lact_schema::IntelDrmInfo;
use std::path::Path;

pub struct MockDrmProvider {
    snapshot: SnapshotDeviceInfo,
}

impl MockDrmProvider {
    pub fn new(sysfs: &Path) -> Option<Self> {
        let info_path = sysfs.parent()?.parent()?.join("info.json");
        let raw_snapshot = std::fs::read_to_string(info_path).ok()?;
        let snapshot = serde_json::from_str(&raw_snapshot).expect("could not parse snapshot");

        Some(Self { snapshot })
    }
}

impl DrmProvider for MockDrmProvider {
    fn get_intel_info(&self) -> IntelDrmInfo {
        self.snapshot
            .info
            .drm_info
            .as_ref()
            .map(|info| info.intel.clone())
            .unwrap_or_default()
    }

    fn get_vram_info(&self) -> VramInfo {
        VramInfo {
            total: self.snapshot.stats.vram.total.unwrap_or(0),
            used: self.snapshot.stats.vram.used.unwrap_or(0),
            mem_info: self
                .snapshot
                .info
                .drm_info
                .as_ref()
                .and_then(|info| info.memory_info.clone())
                .unwrap_or_default(),
        }
    }
}
