use crate::bindings::intel::{
    drm_xe_device_query, drm_xe_query_mem_regions, DRM_COMMAND_BASE, DRM_IOCTL_BASE,
    DRM_XE_DEVICE_QUERY, DRM_XE_DEVICE_QUERY_MEM_REGIONS,
};
use nix::{errno::Errno, ioctl_readwrite};
use std::{alloc, fs::File, mem, os::fd::AsRawFd};

use super::DrmBox;

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
    xe_device_query(fd, &raw mut query)?;

    if query.size == 0 {
        return Ok(None);
    }

    let layout = alloc::Layout::from_size_align(query.size as usize, mem::align_of::<T>()).unwrap();
    #[allow(clippy::cast_ptr_alignment)]
    let data = alloc::alloc_zeroed(layout) as *const T;

    query.data = data as u64;

    xe_device_query(fd, &raw mut query)?;

    Ok(Some(DrmBox { data, layout }))
}

pub fn query_mem_regions(fd: &File) -> Result<Option<DrmBox<drm_xe_query_mem_regions>>, Errno> {
    unsafe { query_item(fd.as_raw_fd(), DRM_XE_DEVICE_QUERY_MEM_REGIONS) }
}
