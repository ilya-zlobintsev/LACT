use super::{DrmBox, DrmProvider, VramInfo};
use crate::bindings::intel::{
    DRM_COMMAND_BASE, DRM_IOCTL_BASE, DRM_XE_DEVICE_QUERY, DRM_XE_DEVICE_QUERY_MEM_REGIONS,
    drm_xe_device_query, drm_xe_memory_class_DRM_XE_MEM_REGION_CLASS_VRAM,
    drm_xe_query_mem_regions,
};
use lact_schema::{DrmMemoryInfo, IntelDrmInfo};
use nix::{errno::Errno, ioctl_readwrite};
use std::{
    alloc, mem,
    os::fd::{AsRawFd, OwnedFd},
};

pub struct XeDrmProvider {
    fd: OwnedFd,
}

impl XeDrmProvider {
    pub fn new(fd: OwnedFd) -> Self {
        Self { fd }
    }
}

impl DrmProvider for XeDrmProvider {
    fn get_intel_info(&self) -> IntelDrmInfo {
        IntelDrmInfo::default()
    }

    fn get_vram_info(&self) -> VramInfo {
        let mut total = 0;
        let mut used = 0;
        let mut cpu_accessible_total = 0;
        let mut cpu_accessible_used = 0;

        let result = unsafe {
            query_item::<drm_xe_query_mem_regions>(
                self.fd.as_raw_fd(),
                DRM_XE_DEVICE_QUERY_MEM_REGIONS,
            )
        };

        if let Ok(Some(query)) = result {
            unsafe {
                let regions = query.mem_regions.as_slice(query.num_mem_regions as usize);
                for region_info in regions {
                    if u32::from(region_info.mem_class)
                        == drm_xe_memory_class_DRM_XE_MEM_REGION_CLASS_VRAM
                    {
                        total += region_info.total_size;
                        used += region_info.used;

                        if region_info.cpu_visible_size > 0 {
                            cpu_accessible_total += region_info.cpu_visible_size;
                            cpu_accessible_used += region_info.cpu_visible_used;
                        }
                    }
                }
            }
        }

        VramInfo {
            total,
            used,
            mem_info: DrmMemoryInfo {
                cpu_accessible_used,
                cpu_accessible_total,
                resizeable_bar: Some(cpu_accessible_total == total),
            },
        }
    }
}

ioctl_readwrite!(
    xe_device_query,
    DRM_IOCTL_BASE,
    DRM_COMMAND_BASE + DRM_XE_DEVICE_QUERY,
    drm_xe_device_query
);

unsafe fn query_item<T>(fd: i32, query_id: u32) -> Result<Option<DrmBox<T>>, Errno> {
    let mut query = drm_xe_device_query {
        query: query_id,
        size: 0,
        data: 0,
        extensions: 0,
        reserved: [0, 0],
    };
    unsafe {
        xe_device_query(fd, &raw mut query)?;
    }

    if query.size == 0 {
        return Ok(None);
    }

    let layout = alloc::Layout::from_size_align(query.size as usize, mem::align_of::<T>()).unwrap();

    unsafe {
        #[allow(clippy::cast_ptr_alignment)]
        let data = alloc::alloc_zeroed(layout) as *const T;

        query.data = data as u64;

        xe_device_query(fd, &raw mut query)?;

        Ok(Some(DrmBox { data, layout }))
    }
}
