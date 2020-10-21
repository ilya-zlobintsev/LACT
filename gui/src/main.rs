extern crate gdk;
extern crate gio;
extern crate gtk;

use daemon::{Daemon, daemon_connection::DaemonConnection};
use gio::prelude::*;
use gtk::{ButtonsType, DialogFlags, Label, LevelBar, MessageType, prelude::*};

use gtk::{Builder, MessageDialog, TextBuffer, Window};

use std::{process::Command, env::args, thread, time::Duration};

fn build_ui(application: &gtk::Application) {
    let glade_src = include_str!("main_window.glade");
    let builder = Builder::from_string(glade_src);
    println!("Getting elements");

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

    let vram_usage_level_bar: LevelBar = builder
        .get_object("vram_usage_level_bar")
        .expect("Couldnt get levelbar");

    let vram_usage_label: Label = builder
        .get_object("vram_usage_label")
        .expect("Couldn't get label");
    
    let gpu_clock_text_buffer: TextBuffer = builder
        .get_object("gpu_clock_text_buffer").unwrap();    

    let vram_clock_text_buffer: TextBuffer = builder
        .get_object("vram_clock_text_buffer").unwrap();    

    let gpu_temp_text_buffer: TextBuffer = builder
        .get_object("gpu_temp_text_buffer").unwrap();

    let gpu_power_text_buffer: TextBuffer = builder
        .get_object("gpu_power_text_buffer").unwrap();

    let d = match DaemonConnection::new() {
        Ok(a) => a,
        Err(_) => {
            let daemon = Daemon::new();
            thread::spawn(move || {
                daemon.listen();
            });

            let diag = MessageDialog::new(
                None::<&Window>,
                DialogFlags::empty(),
                MessageType::Error,
                ButtonsType::Ok,
                "Running in unpriveleged mode",
            );
            diag.run();
            diag.hide();

            DaemonConnection::new().unwrap()
        }
    };
    println!("Connected");

    let gpu_info = d.get_gpu_info();

    gpu_model_text_buffer.set_text(&gpu_info.card_model);
    manufacturer_text_buffer.set_text(&gpu_info.card_vendor);
    vbios_version_text_buffer.set_text(&gpu_info.vbios_version);
    driver_text_buffer.set_text(&gpu_info.driver);
    vram_size_text_buffer.set_text(&format!("{} MiB", &gpu_info.vram_size));
    link_speed_text_buffer.set_text(&format!(
        "{} x{}",
        &gpu_info.link_speed, &gpu_info.link_width
    ));

    vulkan_device_name_text_buffer.set_text(&gpu_info.vulkan_info.device_name);
    vulkan_version_text_buffer.set_text(&gpu_info.vulkan_info.api_version);
    vulkan_features_text_buffer.set_text(&gpu_info.vulkan_info.features);

    let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    thread::spawn(move || {
        loop {
            let gpu_stats = d.get_gpu_stats();

            tx.send(gpu_stats).expect("Couldn't send text");
            thread::sleep(Duration::from_millis(500));
        }
    });

    rx.attach(None, move |gpu_stats| {
        vram_usage_level_bar.set_max_value(gpu_stats.mem_total as f64);
        vram_usage_level_bar.set_value(gpu_stats.mem_used as f64);
        
        let text = format!("{}/{} MiB", gpu_stats.mem_used, gpu_stats.mem_total);
        vram_usage_label.set_text(&text);

        gpu_clock_text_buffer.set_text(&format!("{}MHz", gpu_stats.gpu_freq));
        vram_clock_text_buffer.set_text(&format!("{}MHz", gpu_stats.mem_freq));

        gpu_temp_text_buffer.set_text(&format!("{}°C", gpu_stats.gpu_temp));

        gpu_power_text_buffer.set_text(&format!("{}/{}W", gpu_stats.power_avg, gpu_stats.power_max));

        glib::Continue(true)
    });
   

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
