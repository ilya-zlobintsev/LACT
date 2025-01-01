/*use super::bindings::xe::{drm_xe_device_query, DRM_XE_DEVICE_QUERY, DRM_XE_DEVICE_QUERY_ENGINES};
use crate::server::gpu_controller::intel::bindings::xe::{
    drm_xe_query_engines, DRM_COMMAND_BASE, DRM_IOCTL_BASE, DRM_XE_DEVICE_QUERY_HWCONFIG,
};
use nix::{errno::Errno, ioctl_readwrite};
use std::{
    alloc::{self, dealloc},
    fs::File,
    mem,
    os::fd::AsRawFd,
};

ioctl_readwrite!(
    xe_device_query,
    DRM_IOCTL_BASE,
    DRM_COMMAND_BASE + DRM_XE_DEVICE_QUERY,
    drm_xe_device_query
);

pub fn query_engines(fd: &File) -> Result<(), Errno> {
    unsafe {
        let mut query = drm_xe_device_query {
            extensions: 0,
            query: DRM_XE_DEVICE_QUERY_ENGINES,
            size: 0,
            data: 0,
            reserved: mem::zeroed(),
        };

        xe_device_query(fd.as_raw_fd(), &mut query)?;

        let layout = alloc::Layout::from_size_align(
            query.size as usize,
            mem::align_of::<drm_xe_query_engines>(),
        )
        .unwrap();

        #[allow(clippy::cast_ptr_alignment)]
        let query_engines = alloc::alloc(layout) as *const drm_xe_query_engines;
        query.data = query_engines as u64;

        xe_device_query(fd.as_raw_fd(), &mut query)?;

        println!("query data: {query:?}");

        for engine in (*query_engines)
            .engines
            .as_slice((*query_engines).num_engines as usize)
        {
            println!("Engine {engine:?}");
        }

        dealloc(query_engines as *mut u8, layout);

        Ok(())
    }
}

pub fn query_hwconfig(fd: &File) -> Result<(), Errno> {
    unsafe {
        let mut query = drm_xe_device_query {
            extensions: 0,
            query: DRM_XE_DEVICE_QUERY_HWCONFIG,
            size: 0,
            data: 0,
            reserved: mem::zeroed(),
        };

        xe_device_query(fd.as_raw_fd(), &mut query)?;

        println!("{query:?}");

        Ok(())
    }
}*/
