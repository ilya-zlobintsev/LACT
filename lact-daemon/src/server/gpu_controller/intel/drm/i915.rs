#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
use super::{DrmBox, DrmProvider, VramInfo};
use crate::bindings::intel::{
    DRM_COMMAND_BASE, DRM_I915_QUERY_MEMORY_REGIONS, DRM_IOCTL_BASE,
    drm_i915_gem_memory_class_I915_MEMORY_CLASS_DEVICE, drm_i915_query,
    drm_i915_query_memory_regions,
};
use crate::bindings::intel::{IntelDrm, drm_i915_query_item};
use lact_schema::{DrmMemoryInfo, IntelDrmInfo};
use nix::{errno::Errno, ioctl_readwrite};
use std::ffi::c_int;
use std::os::fd::OwnedFd;
use std::{alloc, mem, os::fd::AsRawFd, ptr};

const DRM_I915_QUERY: u32 = 0x39;

pub struct I915DrmProvider {
    drm: &'static IntelDrm,
    fd: OwnedFd,
}

impl I915DrmProvider {
    pub fn new(drm: &'static IntelDrm, fd: OwnedFd) -> Self {
        Self { drm, fd }
    }

    fn drm_try<T: Default>(&self, f: unsafe fn(&IntelDrm, c_int, *mut T) -> c_int) -> Option<T> {
        unsafe {
            let mut out = T::default();
            let result = f(self.drm, self.fd.as_raw_fd(), &raw mut out);
            if result == 0 { Some(out) } else { None }
        }
    }
}

impl DrmProvider for I915DrmProvider {
    fn get_intel_info(&self) -> IntelDrmInfo {
        IntelDrmInfo {
            execution_units: self.drm_try(IntelDrm::drm_intel_get_eu_total),
            subslices: self.drm_try(IntelDrm::drm_intel_get_subslice_total),
        }
    }

    fn get_vram_info(&self) -> VramInfo {
        let mut total = 0;
        let mut used = 0;
        let mut cpu_accessible_total = 0;
        let mut cpu_accessible_used = 0;

        let result = unsafe {
            query_item::<drm_i915_query_memory_regions>(
                self.fd.as_raw_fd(),
                DRM_I915_QUERY_MEMORY_REGIONS,
            )
        };

        if let Ok(Some(query)) = result {
            let mut i915_unallocated = 0;
            let mut cpu_unallocated = 0;

            unsafe {
                let regions = query.regions.as_slice(query.num_regions as usize);
                for region_info in regions {
                    if u32::from(region_info.region.memory_class)
                        == drm_i915_gem_memory_class_I915_MEMORY_CLASS_DEVICE
                    {
                        total += region_info.probed_size;
                        i915_unallocated += region_info.unallocated_size;

                        let cpu_region_info = region_info.__bindgen_anon_1.__bindgen_anon_1;
                        if cpu_region_info.probed_cpu_visible_size > 0 {
                            cpu_accessible_total += cpu_region_info.probed_cpu_visible_size;
                            cpu_unallocated += cpu_region_info.unallocated_cpu_visible_size;
                        }
                    }
                }
            }

            if total > 0 {
                used = total - i915_unallocated;
            }

            if cpu_accessible_total > 0 {
                cpu_accessible_used = cpu_accessible_total - cpu_unallocated;
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
    i915_query,
    DRM_IOCTL_BASE,
    DRM_COMMAND_BASE + DRM_I915_QUERY,
    drm_i915_query
);

unsafe fn query_item<T>(fd: i32, query_id: u32) -> Result<Option<DrmBox<T>>, Errno> {
    let mut query_item = drm_i915_query_item {
        query_id: query_id as u64,
        length: 0,
        flags: 0,
        data_ptr: 0,
    };

    let mut query = drm_i915_query {
        num_items: 1,
        flags: 0,
        items_ptr: ptr::from_mut(&mut query_item) as u64,
    };

    unsafe {
        i915_query(fd, &raw mut query)?;

        if (*(query.items_ptr as *mut drm_i915_query_item)).length <= 0 {
            return Ok(None);
        }

        let layout =
            alloc::Layout::from_size_align(query_item.length as usize, mem::align_of::<T>())
                .unwrap();
        #[allow(clippy::cast_ptr_alignment)]
        let data = alloc::alloc_zeroed(layout) as *const T;

        (*(query.items_ptr as *mut drm_i915_query_item)).data_ptr = data as u64;

        i915_query(fd, &raw mut query)?;

        Ok(Some(DrmBox { data, layout }))
    }
}
