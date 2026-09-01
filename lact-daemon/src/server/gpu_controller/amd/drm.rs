pub mod amdgpu;
#[cfg(feature = "mock")]
pub mod mock;

use amdgpu_sysfs::gpu_handle::GpuHandle;
use lact_schema::{CacheInfo, DeviceType, DrmInfo};
use libdrm_amdgpu_sys::AMDGPU::VBIOS::VbiosInfo;

pub trait DrmProvider {
    fn get_drm_info(&self, handle: &GpuHandle, cache_info: Option<CacheInfo>) -> Option<DrmInfo>;

    fn get_device_type(&self) -> Option<DeviceType>;

    fn get_gtt_size(&self) -> Result<u64, i32>;

    fn get_gtt_used(&self) -> Result<u64, i32>;

    fn get_vram_clock(&self) -> Result<u64, i32>;

    fn get_vbios_info(&self) -> Result<VbiosInfo, i32>;

    fn get_device_name(&self) -> Option<String>;
}
