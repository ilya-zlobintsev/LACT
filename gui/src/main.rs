extern crate gdk;
extern crate gio;
extern crate gtk;

use daemon::{Daemon, daemon_connection::DaemonConnection};
use gio::prelude::*;
use gtk::{Adjustment, Button, ButtonsType, ComboBoxText, DialogFlags, Frame, Label, LevelBar, MessageType, Switch, prelude::*};

use gtk::{Builder, MessageDialog, TextBuffer, Window};
use pango::EllipsizeMode;

use std::{collections::BTreeMap, env::args, fs, sync::{Arc, RwLock}, thread, time::Duration};

fn build_ui(application: &gtk::Application) {
    let glade_src = include_str!("main_window.glade");
    let builder = Builder::from_string(glade_src);
    println!("Getting elements");

    let main_window: Window = builder
        .get_object("main_window")
        .expect("Couldn't get main_window");

    let vram_usage_level_bar: LevelBar = builder
        .get_object("vram_usage_level_bar")
        .expect("Couldnt get levelbar");

    let vram_usage_label: Label = builder
        .get_object("vram_usage_label")
        .expect("Couldn't get label");

    let gpu_select_comboboxtext: ComboBoxText = builder.get_object("gpu_select_comboboxtext").unwrap();

    let gpu_clock_text_buffer: TextBuffer = builder.get_object("gpu_clock_text_buffer").unwrap();

    let vram_clock_text_buffer: TextBuffer = builder.get_object("vram_clock_text_buffer").unwrap();

    let gpu_temp_text_buffer: TextBuffer = builder.get_object("gpu_temp_text_buffer").unwrap();

    let gpu_power_text_buffer: TextBuffer = builder.get_object("gpu_power_text_buffer").unwrap();

    let fan_speed_text_buffer: TextBuffer = builder.get_object("fan_speed_text_buffer").unwrap();

    let power_cap_label: Label = builder.get_object("power_cap_label").unwrap();

    let apply_button: Button = builder.get_object("apply_button").unwrap();
    
    let automatic_fan_control_switch: Switch =
        builder.get_object("automatic_fan_control_switch").unwrap();

    let fan_curve_frame: Frame = builder.get_object("fan_curve_frame").unwrap();
    
    let gpu_power_adjustment: Adjustment = builder.get_object("gpu_power_adjustment").unwrap();


    let mut unpriviliged: bool = false;

    let d = match DaemonConnection::new() {
        Ok(a) => a,
        Err(_) => {
            unpriviliged = true;

            let daemon = Daemon::new(unpriviliged);
            thread::spawn(move || {
                daemon.listen();
            });

            let diag = MessageDialog::new(
                None::<&Window>,
                DialogFlags::empty(),
                MessageType::Error,
                ButtonsType::Ok,
                "Running in unpriviliged mode",
            );
            diag.run();
            diag.hide();

            DaemonConnection::new().unwrap()
        }
    };
    println!("Connected");

    let gpus = d.get_gpus().unwrap();

    for gpu in &gpus {
        gpu_select_comboboxtext.append(Some(&gpu.0.to_string()), &gpu.1);
    }

    //limits the length of gpu names in combobox
    for cell in gpu_select_comboboxtext.get_cells() {
        cell.set_property("width-chars", &10).unwrap();
        cell.set_property("ellipsize", &EllipsizeMode::End).unwrap();
    }

    let current_gpu_id  = Arc::new(RwLock::new(0u32));

    let build = builder.clone();

    let b = apply_button.clone();
    gpu_power_adjustment.connect_value_changed(move |adjustment| {
        println!("changed adjustment value to {}/{}", adjustment.get_value(), adjustment.get_upper());
        b.set_sensitive(true);
        power_cap_label.set_text(&format!("{}/{}", adjustment.get_value().floor(), adjustment.get_upper()));
    });
    

    let cur_id = current_gpu_id.clone();
    let b = apply_button.clone();
    
    gpu_select_comboboxtext.connect_changed(move |combobox| {
        let mut current_gpu_id = cur_id.write().unwrap();
        *current_gpu_id = combobox.get_active_id().unwrap().parse::<u32>().expect("invalid id");
        println!("Set current gpu id to {}", current_gpu_id);

        set_info(&build, d, current_gpu_id.clone());

        b.set_sensitive(false);

    });

    //gpu_select_comboboxtext.set_active_id(Some(&current_gpu_id.to_string()));
    gpu_select_comboboxtext.set_active(Some(0));

    if unpriviliged {
        automatic_fan_control_switch.set_sensitive(false);
        fan_curve_frame.set_visible(false);
        automatic_fan_control_switch.set_tooltip_text(Some("Unavailable in unprivileged mode"));
    }

    let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    let cur_gpu_id = current_gpu_id.clone();
    thread::spawn(move || loop {
        let current_gpu_id = *cur_gpu_id.clone().read().unwrap();
        println!("Getting stats for {}", current_gpu_id);
        let gpu_stats = d.get_gpu_stats(current_gpu_id).unwrap();

        tx.send(gpu_stats).expect("Couldn't send text");
        thread::sleep(Duration::from_millis(500));
    });

    rx.attach(None, move |gpu_stats| {
        vram_usage_level_bar.set_max_value(gpu_stats.mem_total as f64);
        vram_usage_level_bar.set_value(gpu_stats.mem_used as f64);

        let text = format!("{}/{} MiB", gpu_stats.mem_used, gpu_stats.mem_total);
        vram_usage_label.set_text(&text);

        gpu_clock_text_buffer.set_text(&format!("{}MHz", gpu_stats.gpu_freq));
        vram_clock_text_buffer.set_text(&format!("{}MHz", gpu_stats.mem_freq));

        gpu_temp_text_buffer.set_text(&format!("{}°C", gpu_stats.gpu_temp));

        gpu_power_text_buffer
            .set_text(&format!("{}/{}W", gpu_stats.power_avg, gpu_stats.power_cap));

        fan_speed_text_buffer.set_text(&format!(
            "{}RPM({}%)",
            gpu_stats.fan_speed,
            (gpu_stats.fan_speed as f64 / gpu_stats.max_fan_speed as f64 * 100 as f64) as i32
        ));


        glib::Continue(true)
    });

    let b = apply_button.clone();

    let switch = automatic_fan_control_switch.clone();
    automatic_fan_control_switch.connect_changed_active(move |_| {
        match switch.get_active() {
            true => {
                fan_curve_frame.set_visible(false);
            }
            false => {
                fan_curve_frame.set_visible(true);
            }
        }

        b.set_sensitive(true);
    });



    main_window.set_application(Some(application));

    main_window.show();
}

fn set_info(builder: &Builder, d: DaemonConnection, gpu_id: u32) {
    let gpu_model_text_buffer: TextBuffer = builder
        .get_object("gpu_model_text_buffer")
        .expect("Couldn't get textbuffer");
let vbios_version_text_buffer: TextBuffer = builder .get_object("vbios_version_text_buffer") .expect("Couldn't get textbuffer");
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

    let automatic_fan_control_switch: Switch =
        builder.get_object("automatic_fan_control_switch").unwrap();

    let fan_curve_frame: Frame = builder.get_object("fan_curve_frame").unwrap();
    
    let gpu_power_adjustment: Adjustment = builder.get_object("gpu_power_adjustment").unwrap();

    let apply_button: Button = builder.get_object("apply_button").unwrap();

    let overclocking_info_frame: Frame = builder.get_object("overclocking_info_frame").unwrap();

    match fs::read_to_string("/proc/cmdline") {
        Ok(cmdline) => {
            if cmdline.contains("amdgpu.ppfeaturemask=") {
                overclocking_info_frame.set_visible(false);
            }
        },
        Err(_) => (),
    }

    let gpu_info = d.get_gpu_info(gpu_id).unwrap();

    gpu_model_text_buffer.set_text(&gpu_info.card_model);
    manufacturer_text_buffer.set_text(&gpu_info.card_vendor);
    vbios_version_text_buffer.set_text(&gpu_info.vbios_version);
    driver_text_buffer.set_text(&gpu_info.driver);
    vram_size_text_buffer.set_text(&format!("{} MiB", &gpu_info.vram_size));
    link_speed_text_buffer.set_text(&format!(
        "{} x{}",
        &gpu_info.link_speed, &gpu_info.link_width
    ));

    let vulkan_features = gpu_info.vulkan_info.features.replace(',', "\n").replace("Features", "").replace("{", "").replace("}", "").replace(" ", "").replace(":", ": ");

    vulkan_device_name_text_buffer.set_text(&gpu_info.vulkan_info.device_name);
    vulkan_version_text_buffer.set_text(&gpu_info.vulkan_info.api_version);
    vulkan_features_text_buffer.set_text(&vulkan_features);

    let (power_cap, power_cap_max) = d.get_power_cap(gpu_id).unwrap();

    gpu_power_adjustment.set_upper(power_cap_max as f64);
    gpu_power_adjustment.set_value(power_cap as f64);

    
    let fan_control = d.get_fan_control(gpu_id);
    
    match fan_control {
        Ok(ref fan_control) => {
            if fan_control.enabled {
                println!("Automatic fan control disabled!");
                automatic_fan_control_switch.set_active(false);
                fan_curve_frame.set_visible(true);
            } else {
                println!("Automatic fan control enabled");
                automatic_fan_control_switch.set_active(true);
                fan_curve_frame.set_visible(false);
            }
        },
        Err(_) => {
            automatic_fan_control_switch.set_sensitive(false);
            automatic_fan_control_switch.set_tooltip_text(Some("Unavailable"));
    
            fan_curve_frame.set_visible(false);
        }
    }
    
    match fan_control {
        Ok(fan_control) => {

            let curve: Arc<RwLock<BTreeMap<i32, f64>>> = Arc::new(RwLock::new(fan_control.curve));

            for i in 1..6 {
                let curve_temperature_adjustment: Adjustment = builder
                    .get_object(&format!("curve_temperature_adjustment_{}", i))
                    .unwrap();

                let value = *curve
                    .read()
                    .unwrap()
                    .get(&(i * 20))
                    .expect("Could not get by index");
                println!("Setting value {} on adjustment {}", value, i);
                curve_temperature_adjustment.set_value(value);

                let c = curve.clone();
                let b = apply_button.clone();

                curve_temperature_adjustment.connect_value_changed(move |adj| {
                    c.write().unwrap().insert(20 * i, adj.get_value());
                    b.set_sensitive(true);
                });
            }

            apply_button.connect_clicked(move |b| {
                //let current_gpu_id = *current_gpu_id.read().unwrap();

                let curve = curve.read().unwrap().clone();
                println!("setting curve to {:?}", curve);
                d.set_fan_curve(gpu_id, curve).unwrap();
                b.set_sensitive(false);

                match automatic_fan_control_switch.get_active() {
                    true => {
                        d.stop_fan_control(gpu_id).unwrap();
                        
                        let diag = MessageDialog::new(
                            None::<&Window>,
                            DialogFlags::empty(),
                            MessageType::Error,
                            ButtonsType::Ok,
                            "WARNING: Due to a driver bug, the GPU fan may misbehave after switching to automatic control. You may need to reboot your system to avoid issues.",
                        );
                        diag.run();
                        diag.hide();
                    }
                    false => {
                        d.start_fan_control(gpu_id).unwrap();
                    }
                }

                let power_cap = gpu_power_adjustment.get_value().floor() as i32;
                d.set_power_cap(gpu_id, power_cap).unwrap();
            });
        },
        Err(_) => (),
    }
}

fn main() {
    println!("Initializing gtk");
    let application = gtk::Application::new(Some("com.ilyaz.yagc"), Default::default())
        .expect("failed to initialize");

    application.connect_activate(|app| {
        println!("Activating");
        build_ui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}
