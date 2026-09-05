pub mod i915;
#[cfg(feature = "mock")]
pub mod mock;
pub mod xe;

use lact_schema::{DrmMemoryInfo, IntelDrmInfo};
use std::{alloc, ops::Deref};

pub struct DrmBox<T> {
    data: *const T,
    layout: alloc::Layout,
}

impl<T> Deref for DrmBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.data) }
    }
}

impl<T> Drop for DrmBox<T> {
    fn drop(&mut self) {
        unsafe {
            alloc::dealloc(self.data as *mut u8, self.layout);
        }
    }
}

pub trait DrmProvider {
    fn get_intel_info(&self) -> IntelDrmInfo;

    fn get_vram_info(&self) -> VramInfo;
}

pub struct VramInfo {
    pub total: u64,
    pub used: u64,
    pub mem_info: DrmMemoryInfo,
}
