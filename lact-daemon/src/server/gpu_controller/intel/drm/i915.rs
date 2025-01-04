/*#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
use crate::server::gpu_controller::intel::drm::bindings::i915::{
    drm_i915_query, DRM_COMMAND_BASE, DRM_I915_QUERY_TOPOLOGY_INFO, DRM_IOCTL_BASE,
};
use nix::{errno::Errno, ioctl_readwrite};
use std::{alloc, fs::File, mem, os::fd::AsRawFd};

use super::bindings::i915::{
    drm_i915_query_item, drm_i915_query_topology_info, DRM_I915_QUERY_HWCONFIG_BLOB,
};

const DRM_I915_QUERY: u32 = 0x39;

ioctl_readwrite!(
    i915_device_query,
    DRM_IOCTL_BASE,
    DRM_COMMAND_BASE + DRM_I915_QUERY,
    drm_i915_query
);

unsafe fn query_item(fd: i32, item: &mut drm_i915_query_item) -> Result<(), Errno> {
    let query_items = [*item];

    let mut query = drm_i915_query {
        num_items: query_items.len() as u32,
        flags: 0,
        items_ptr: query_items.as_ptr() as u64,
    };

    i915_device_query(fd, &mut query)?;
    *item = query_items[0];
    Ok(())
}

pub fn query_hwconfig(fd: &File) -> Result<(), Errno> {
    let fd = fd.as_raw_fd();

    unsafe {
        let mut item = drm_i915_query_item {
            query_id: DRM_I915_QUERY_HWCONFIG_BLOB as u64,
            length: 0,
            flags: 0,
            data_ptr: 0,
        };
        query_item(fd, &mut item)?;

        if item.length <= 0 {
            println!("Nothing found");
            return Ok(());
        }
        println!("asd");

        // let layout = alloc::Layout::from_size_align(
        //     item.length as usize,
        //     mem::align_of::<drm_i915_query_topology_info>(),
        // )
        // .unwrap();

        // #[allow(clippy::cast_ptr_alignment)]
        // let topology_info = alloc::alloc(layout) as *const drm_i915_query_topology_info;
        // item.data_ptr = topology_info as u64;

        // item.data_ptr = println!("query result: {item:?}");
    }

    Ok(())
}*/
