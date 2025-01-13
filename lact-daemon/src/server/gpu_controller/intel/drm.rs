pub mod i915;
pub mod xe;

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
