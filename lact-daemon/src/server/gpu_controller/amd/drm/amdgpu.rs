use super::DrmProvider;
use amdgpu_sysfs::gpu_handle::GpuHandle;
use anyhow::{Context as _, anyhow};
use lact_schema::{AmdIpInfo, CacheInfo, DeviceType, DrmInfo, DrmMemoryInfo, RopInfo};
use libdrm_amdgpu_sys::{
    AMDGPU::{
        GPU_INFO as _, HW_IP::HW_IP_TYPE, SENSOR_INFO::SENSOR_TYPE, VBIOS::VbiosInfo, VRAM_TYPE,
    },
    LibDrmAmdgpu,
};
use std::fs;
use tracing::error;

const AMDGPU_IDS_FLAGS_FUSION: u64 = 0x1;

const ALL_HW_IP: &[HW_IP_TYPE] = &[
    HW_IP_TYPE::GFX,
    HW_IP_TYPE::COMPUTE,
    HW_IP_TYPE::DMA,
    HW_IP_TYPE::UVD,
    HW_IP_TYPE::VCE,
    HW_IP_TYPE::UVD_ENC,
    HW_IP_TYPE::VCN_DEC,
    HW_IP_TYPE::VCN_ENC,
    HW_IP_TYPE::VCN_JPEG,
    HW_IP_TYPE::VPE,
];

use crate::server::gpu_controller::CommonControllerInfo;

pub struct AmdGpuDrmProvider(libdrm_amdgpu_sys::AMDGPU::DeviceHandle);

impl AmdGpuDrmProvider {
    pub fn new(
        common: &CommonControllerInfo,
        libdrm_amdgpu: &LibDrmAmdgpu,
    ) -> anyhow::Result<Self> {
        let path = common.get_drm_render()?;
        let drm_file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .with_context(|| format!("Could not open drm file at {}", path.display()))?;
        let (handle, _, _) = libdrm_amdgpu
            .init_device_handle_with_fd(drm_file)
            .map_err(|err| anyhow!("Could not open drm handle, error code {err}"))?;

        Ok(Self(handle))
    }
}

impl DrmProvider for AmdGpuDrmProvider {
    fn get_drm_info(&self, handle: &GpuHandle, cache_info: Option<CacheInfo>) -> Option<DrmInfo> {
        let drm_memory_info = self.0.memory_info().ok().map(|memory_info| DrmMemoryInfo {
            resizeable_bar: Some(memory_info.check_resizable_bar()),
            cpu_accessible_used: memory_info.cpu_accessible_vram.heap_usage,
            cpu_accessible_total: memory_info.cpu_accessible_vram.total_heap_size,
        });
        let drm_info = self
            .0
            .device_info()
            .inspect_err(|err| error!("could not fetch DRM info: {err}"))
            .ok()?;

        Some(DrmInfo {
            device_name: drm_info.find_device_name(),
            pci_revision_id: Some(drm_info.pci_rev_id()),
            family_name: Some(drm_info.get_family_name().to_string()),
            family_id: Some(drm_info.family_id()),
            asic_name: Some(drm_info.get_asic_name().to_string()),
            chip_class: Some(drm_info.get_chip_class().to_string()),
            compute_units: Some(drm_info.cu_active_number),
            isa: drm_info
                .get_gfx_target_version()
                .map(|version| version.to_string()),
            streaming_multiprocessors: None,
            cuda_cores: None,
            vram_type: Some(drm_info.get_vram_type().to_string()),
            vram_vendor: handle.get_vram_vendor().ok(),
            vram_clock_ratio: match drm_info.get_vram_type() {
                VRAM_TYPE::GDDR6 => 2.0,
                _ => 1.0,
            },
            amd_ip_info: ALL_HW_IP
                .iter()
                .filter_map(|ip_type| {
                    let ip_info = self.0.get_hw_ip_info(*ip_type).ok()?;

                    Some(AmdIpInfo {
                        ip_type: ip_info.ip_type.to_string(),
                        version_major: ip_info.info.hw_ip_version_major,
                        version_minor: ip_info.info.hw_ip_version_minor,
                        queues: ip_info.info.num_queues(),
                        count: ip_info.count,
                    })
                })
                .collect(),
            vram_bit_width: Some(drm_info.vram_bit_width),
            vram_max_bw: Some(drm_info.peak_memory_bw_gb().to_string()),
            cache_info,
            memory_info: drm_memory_info,
            rop_info: Some(RopInfo {
                unit_count: drm_info.rb_pipes(),
                operations_factor: if drm_info.get_asic_name().rbplus_allowed() {
                    8
                } else {
                    4
                },
                operations_count: drm_info.calc_rop_count(),
            }),
            ..Default::default()
        })
    }

    fn get_gtt_size(&self) -> Result<u64, i32> {
        self.0.vram_gtt_info().map(|info| info.gtt_size)
    }

    fn get_gtt_used(&self) -> Result<u64, i32> {
        self.0.gtt_usage_info()
    }

    fn get_vram_clock(&self) -> Result<u64, i32> {
        self.0.sensor_info(SENSOR_TYPE::GFX_MCLK).map(u64::from)
    }

    fn get_vbios_info(&self) -> Result<VbiosInfo, i32> {
        self.0.get_vbios_info()
    }

    fn get_device_name(&self) -> Option<String> {
        self.0
            .device_info()
            .ok()
            .and_then(|info| info.find_device_name())
    }

    fn get_device_type(&self) -> Option<DeviceType> {
        self.0.device_info().ok().map(|info| {
            if (info.ids_flags & AMDGPU_IDS_FLAGS_FUSION) > 0 {
                DeviceType::Integrated
            } else {
                DeviceType::Dedicated
            }
        })
    }
}
