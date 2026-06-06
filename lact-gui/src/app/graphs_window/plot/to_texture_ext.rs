use std::ffi::c_void;

use cairo::ImageSurface;
use gtk::gdk;
use gtk::gdk::MemoryTexture;
use gtk::glib;
use gtk::glib::ffi::g_bytes_new_with_free_func;
use gtk::glib::translate::FromGlibPtrFull;

pub(super) trait ToTextureExt {
    fn to_texture(&mut self) -> Option<MemoryTexture>;
}

impl ToTextureExt for ImageSurface {
    fn to_texture(&mut self) -> Option<MemoryTexture> {
        // Ensure the surface is of type image
        if self.type_() != cairo::SurfaceType::Image {
            return None;
        }

        let width = self.width();
        let height = self.height();

        // Check if the surface has valid dimensions
        if width <= 0 || height <= 0 {
            return None;
        }

        let stride = self.stride();
        let format = self.format();

        // Use with_data to get mutable access to surface data
        let mut bytes = None;
        self.with_data(|data| {
            // Reference the surface to be passed to the free function
            let surface_ref = self.clone();

            // Use g_bytes_new_with_free_func to manage memory
            unsafe {
                let ptr = g_bytes_new_with_free_func(
                    data.as_ptr() as *const c_void,
                    (height * stride) as usize,
                    Some(c_surface_destroy_notify),
                    Box::into_raw(Box::new(surface_ref)) as *mut c_void,
                );

                bytes = Some(glib::Bytes::from_glib_full(ptr));
            };
        })
        .expect("Failed to get surface data");

        // Create the GdkTexture
        let texture = MemoryTexture::new(
            width,
            height,
            cairo_format_to_memory_format(format),
            &bytes.unwrap(),
            stride as usize,
        );

        Some(texture)
    }
}

// Function that will act as GDestroyNotify to free the cairo surface
extern "C" fn c_surface_destroy_notify(surface_ptr: *mut c_void) {
    if !surface_ptr.is_null() {
        // SAFETY: We know this is a valid ImageSurface as we passed it in Box::into_raw
        let surface: Box<ImageSurface> = unsafe { Box::from_raw(surface_ptr as *mut ImageSurface) };
        drop(surface); // Automatically handles the cleanup
    }
}

// Convert cairo format to gdk::MemoryFormat
fn cairo_format_to_memory_format(format: cairo::Format) -> gdk::MemoryFormat {
    match format {
        cairo::Format::Rgb24 => gdk::MemoryFormat::R8g8b8,
        cairo::Format::ARgb32 => gdk::MemoryFormat::R8g8b8a8,
        _ => panic!("Unsupported cairo format"),
    }
}
