extern crate gdk;
extern crate gio;
extern crate gtk;

use gio::prelude::*;
use gtk::{prelude::*, ButtonsType, DialogFlags, MessageType};

use gtk::{Builder, MessageDialog, TextBuffer, Window};

use std::env::args;

fn build_ui(application: &gtk::Application) {
    let glade_src = include_str!("main_window.glade");
    let builder = Builder::from_string(glade_src);

    let main_window: Window = builder
        .get_object("main_window")
        .expect("Couldn't get main_window");

    let gpu_model_text_buffer: TextBuffer = builder
        .get_object("gpu_model_text_buffer")
        .expect("Couldn't get textbuffer");

    let vbios_version_text_buffer: TextBuffer = builder
        .get_object("vbios_version_text_buffer")
        .expect("Couldn't get textbuffer");

    let driver_text_buffer: TextBuffer = builder
        .get_object("driver_text_buffer")
        .expect("Couldn't get textbuffer");

    let manufacturer_text_buffer: TextBuffer = builder
        .get_object("manufacturer_text_buffer")
        .expect("Couldn't get textbuffer");

    let vram_size_text_buffer: TextBuffer = builder
        .get_object("vram_size_text_buffer")
        .expect("Couldn't get textbuffer");

    let link_speed_text_buffer: TextBuffer = builder
        .get_object("link_speed_text_buffer")
        .expect("Couldn't get textbuffer");

    let vulkan_device_name_text_buffer: TextBuffer = builder  
        .get_object("vulkan_device_name_text_buffer")
        .expect("Couldn't get textbuffer");

    let vulkan_version_text_buffer: TextBuffer = builder
        .get_object("vulkan_version_text_buffer")
        .expect("Couldn't get textbuffer");

    let vulkan_features_text_buffer: TextBuffer = builder
        .get_object("vulkan_features_text_buffer")
        .expect("Couldn't get textbuffer");


    match daemon::get_gpu_info() {
        Ok(gpu_info) => {
            gpu_model_text_buffer.set_text(&gpu_info.card_model);
            manufacturer_text_buffer.set_text(&gpu_info.card_vendor);
            vbios_version_text_buffer.set_text(&gpu_info.vbios_version);
            driver_text_buffer.set_text(&gpu_info.driver);
            vram_size_text_buffer.set_text(&format!("{} MiB", &gpu_info.vram_size));
            link_speed_text_buffer.set_text(&format!("{} x{}", &gpu_info.link_speed, &gpu_info.link_width));

            vulkan_device_name_text_buffer.set_text(&gpu_info.vulkan_info.device_name);
            vulkan_version_text_buffer.set_text(&gpu_info.vulkan_info.api_version);
            vulkan_features_text_buffer.set_text(&gpu_info.vulkan_info.features);
        }
        Err(_) => {
            MessageDialog::new(
                None::<&Window>,
                DialogFlags::empty(),
                MessageType::Error,
                ButtonsType::Ok,
                "Unable to connect to service",
            )
            .run();
            application.quit();
        }
    }

    main_window.set_application(Some(application));

    main_window.show_all();
}

fn main() {
    let application = gtk::Application::new(Some("com.ilyaz.yagc"), Default::default())
        .expect("failed to initialize");

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}
