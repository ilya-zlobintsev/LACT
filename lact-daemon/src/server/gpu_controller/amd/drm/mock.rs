use super::DrmProvider;
use crate::server::handler::SnapshotDeviceInfo;
use lact_schema::{DeviceType, DrmInfo};
use libdrm_amdgpu_sys::AMDGPU::VBIOS::VbiosInfo;
use std::path::Path;
use tracing::info;

pub struct MockDrmProvider {
    snapshot: SnapshotDeviceInfo,
}

impl MockDrmProvider {
    pub fn new(sysfs: &Path) -> Option<Self> {
        let info_path = sysfs.parent()?.parent()?.join("info.json");
        let raw_snapshot = std::fs::read_to_string(&info_path).ok()?;
        let snapshot = serde_json::from_str(&raw_snapshot).expect("could not parse snapshot");

        info!("using mock device info from {}", info_path.display());

        Some(Self { snapshot })
    }
}

impl DrmProvider for MockDrmProvider {
    fn get_drm_info(
        &self,
        _handle: &amdgpu_sysfs::gpu_handle::GpuHandle,
        cache_info: Option<lact_schema::CacheInfo>,
    ) -> Option<DrmInfo> {
        self.snapshot
            .info
            .drm_info
            .clone()
            .map(|info| DrmInfo { cache_info, ..info })
    }

    fn get_device_type(&self) -> Option<DeviceType> {
        Some(DeviceType::Dedicated)
    }

    fn get_gtt_size(&self) -> Result<u64, i32> {
        self.snapshot.stats.vram.gtt_total_usable.ok_or(-1)
    }

    fn get_gtt_used(&self) -> Result<u64, i32> {
        self.snapshot.stats.vram.gtt_used.ok_or(-1)
    }

    fn get_vram_clock(&self) -> Result<u64, i32> {
        self.snapshot.stats.clockspeed.vram_clockspeed.ok_or(-1)
    }

    fn get_vbios_info(&self) -> Result<VbiosInfo, i32> {
        Err(-1)
    }

    fn get_device_name(&self) -> Option<String> {
        self.snapshot
            .info
            .drm_info
            .as_ref()
            .and_then(|info| info.device_name.clone())
    }
}
