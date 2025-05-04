#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
use super::DrmBox;
use crate::bindings::intel::drm_i915_query_item;
use crate::bindings::intel::{
    drm_i915_query, drm_i915_query_memory_regions, drm_i915_query_topology_info, DRM_COMMAND_BASE,
    DRM_I915_QUERY_MEMORY_REGIONS, DRM_I915_QUERY_TOPOLOGY_INFO, DRM_IOCTL_BASE,
};
use nix::{errno::Errno, ioctl_readwrite};
use std::{alloc, fs::File, mem, os::fd::AsRawFd, ptr};

const DRM_I915_QUERY: u32 = 0x39;

ioctl_readwrite!(
    i915_query,
    DRM_IOCTL_BASE,
    DRM_COMMAND_BASE + DRM_I915_QUERY,
    drm_i915_query
);

unsafe fn query_item<T>(fd: i32, query_id: u32) -> Result<(Option<DrmBox<T>>, i32), Errno> {
    let mut query_item = drm_i915_query_item {
        query_id: query_id as u64,
        length: 0,
        flags: 0,
        data_ptr: 0,
    };

    let mut query = drm_i915_query {
        num_items: 1,
        flags: 0,
        items_ptr: ptr::from_ref(&query_item) as u64,
    };

    i915_query(fd, &mut query)?;

    if query_item.length <= 0 {
        return Ok((None, 0));
    }

    let layout =
        alloc::Layout::from_size_align(query_item.length as usize, mem::align_of::<T>()).unwrap();
    #[allow(clippy::cast_ptr_alignment)]
    let data = alloc::alloc_zeroed(layout) as *const T;

    query_item.data_ptr = data as u64;

    i915_query(fd, &mut query)?;

    Ok((Some(DrmBox { data, layout }), query_item.length))
}

pub fn query_memory_regions(
    fd: &File,
) -> Result<Option<DrmBox<drm_i915_query_memory_regions>>, Errno> {
    unsafe { Ok(query_item(fd.as_raw_fd(), DRM_I915_QUERY_MEMORY_REGIONS)?.0) }
}

pub fn query_topology_info(
    fd: &File,
) -> Result<Option<(DrmBox<drm_i915_query_topology_info>, i32)>, Errno> {
    unsafe {
        let (option, length) = query_item(fd.as_raw_fd(), DRM_I915_QUERY_TOPOLOGY_INFO)?;
        Ok(option.map(|item| (item, length)))
    }
}
