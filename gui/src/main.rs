extern crate gdk;
extern crate gio;
extern crate gtk;

use daemon::{daemon_connection::DaemonConnection, gpu_controller::PowerProfile, Daemon};
use gio::prelude::*;
use gtk::{Adjustment, Button, ButtonsType, ComboBoxText, DialogFlags, Frame, Label, LevelBar, MessageType, Notebook, Scale, Switch, prelude::*};

use gtk::{Builder, MessageDialog, TextBuffer, Window};
use pango::EllipsizeMode;

use std::{collections::BTreeMap, env::args, fs, sync::{Arc, Mutex, RwLock}, thread, time::Duration};

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

    let gpu_select_comboboxtext: ComboBoxText =
        builder.get_object("gpu_select_comboboxtext").unwrap();

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

    let gpu_voltage_text_buffer: TextBuffer = builder.get_object("gpu_voltage_text_buffer").unwrap();

    let power_profile_select_comboboxtext: ComboBoxText = builder
        .get_object("power_profile_select_comboboxtext")
        .unwrap();

    let power_profile_description_label: Label = builder.get_object("power_profile_description_label").unwrap();

    let gpu_clockspeed_adjustment: Adjustment = builder.get_object("gpu_clockspeed_adjustment").unwrap();

    let vram_clockspeed_adjustment: Adjustment = builder.get_object("vram_clockspeed_adjustment").unwrap();

    let gpu_voltage_adjustment: Adjustment = builder.get_object("gpu_voltage_adjustment").unwrap();

    let vram_voltage_adjustment: Adjustment = builder.get_object("vram_voltage_adjustment").unwrap();

    let reset_clocks_button: Button = builder.get_object("reset_clocks_button").unwrap();

    let power_cap_scale: Scale = builder.get_object("power_cap_scale").unwrap();

    let clocks_notebook: Notebook = builder.get_object("clocks_notebook").unwrap();

    let clocks_unsupported_label: Label = builder.get_object("clocks_unsupported_label").unwrap();

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

    let current_gpu_id = Arc::new(RwLock::new(0u32));

    { //Handle power limit adjustment change
        let apply_button = apply_button.clone();
        gpu_power_adjustment.connect_value_changed(move |adjustment| {
            println!(
                "changed adjustment value to {}/{}",
                adjustment.get_value(),
                adjustment.get_upper()
            );
            apply_button.set_sensitive(true);
            power_cap_label.set_text(&format!(
                "{}/{}",
                adjustment.get_value().floor(),
                adjustment.get_upper()
            ));
        });
    }

    let adjs = [gpu_clockspeed_adjustment.clone(), vram_clockspeed_adjustment.clone(), gpu_voltage_adjustment.clone(), vram_voltage_adjustment.clone()];

    for adjustment in adjs.iter() {
        let b = apply_button.clone();

        adjustment.connect_value_changed(move |_| {
            b.set_sensitive(true);
        });
    }

    { //Handle changing the GPU power profile
        let b = apply_button.clone();
        let description_label = power_profile_description_label.clone();
        
        power_profile_select_comboboxtext.connect_changed(move |combobox| {
            println!("power profile selection changed");
            b.set_sensitive(true);
            match combobox.get_active().unwrap() {
                0 => description_label.set_text("Automatically adjust core and VRAM clocks. (Default)"),
                1 => description_label.set_text("Always run the on the highest clocks."),
                2 => description_label.set_text("Always run the on the lowest clocks."),
                _ => unreachable!(),
            }
        });
    }


    let gpu_power_level: Arc<Mutex<Option<u32>>> = Arc::new(Mutex::new(None));
    let vram_power_level: Arc<Mutex<Option<u32>>> = Arc::new(Mutex::new(None));

    { // Handle when the GPU is chosen from the dropdown box (also triggered on initializtion)
        let (gpu_power_level, vram_power_level) = (gpu_power_level.clone(), vram_power_level.clone());
        let builder = builder.clone();
        let current_gpu_id = current_gpu_id.clone();

        gpu_select_comboboxtext.connect_changed(move |combobox| {
            let mut current_gpu_id = current_gpu_id.write().unwrap();
            *current_gpu_id = combobox
                .get_active_id()
                .unwrap()
                .parse::<u32>()
                .expect("invalid id");
            println!("Set current gpu id to {}", current_gpu_id);

            set_info(&builder, d, current_gpu_id.clone(), &gpu_power_level, &vram_power_level);
        });
    }

    { //Handle reset clocks button
        let current_gpu_id = current_gpu_id.clone();
        let (gpu_power_level, vram_power_level) = (gpu_power_level.clone(), vram_power_level.clone());
        let builder = builder.clone();
        let apply_button = apply_button.clone();

        reset_clocks_button.connect_clicked(move |_| {
            let current_gpu_id = *current_gpu_id.read().unwrap();
            d.reset_gpu_power_states(current_gpu_id).unwrap();

            set_info(&builder, d, current_gpu_id, &gpu_power_level, &vram_power_level);
            apply_button.set_sensitive(true);
        });
    }

    { //Apply button click
        let current_gpu_id = current_gpu_id.clone();
        let auto_fan_control_switch = automatic_fan_control_switch.clone();
        let power_profile_select_comboboxtext = power_profile_select_comboboxtext.clone();
        let (gpu_power_level, vram_power_level) = (gpu_power_level.clone(), vram_power_level.clone());
        let builder = builder.clone();

        apply_button.connect_clicked(move |_| {
            let gpu_id = *current_gpu_id.read().unwrap();
    
            let mut curve: BTreeMap<i32, f64> = BTreeMap::new();
            
            for i in 1..6 {
                let curve_temperature_adjustment: Adjustment = builder
                    .get_object(&format!("curve_temperature_adjustment_{}", i))
                    .unwrap();
                
                curve.insert(20 * i, curve_temperature_adjustment.get_value());
    
            }
        
            println!("setting curve to {:?}", curve);
            d.set_fan_curve(gpu_id, curve).unwrap();
        
            match auto_fan_control_switch.get_active() {
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
    
            d.set_power_profile(gpu_id, PowerProfile::from_str(&power_profile_select_comboboxtext.get_active_text().unwrap()).unwrap()).unwrap();
    
            if let Some(gpu_power_level) = *gpu_power_level.lock().unwrap() {
                d.set_gpu_power_state(gpu_id, gpu_power_level, gpu_clockspeed_adjustment.get_value() as i32, Some((gpu_voltage_adjustment.get_value() * 1000.0) as i32)).unwrap();
                if let Some(vram_power_level) = *vram_power_level.lock().unwrap() {
                    d.set_vram_power_state(gpu_id, vram_power_level, vram_clockspeed_adjustment.get_value() as i32, Some((vram_voltage_adjustment.get_value() * 1000.0) as i32)).unwrap();
                }
                d.commit_gpu_power_states(gpu_id).unwrap();
            }
    
            set_info(&builder, d, gpu_id, &gpu_power_level, &vram_power_level);
        });
    }

    //gpu_select_comboboxtext.set_active_id(Some(&current_gpu_id.to_string()));
    gpu_select_comboboxtext.set_active(Some(0));

    if unpriviliged {
        automatic_fan_control_switch.set_sensitive(false);
        fan_curve_frame.set_visible(false);
        power_profile_select_comboboxtext.set_sensitive(false);
        power_cap_scale.set_sensitive(false);
        clocks_notebook.set_visible(false);
        clocks_unsupported_label.set_visible(true);
        
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

        gpu_voltage_text_buffer.set_text(&format!("{}V", gpu_stats.voltage as f64 / 1000.0));

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

fn set_info(builder: &Builder, d: DaemonConnection, gpu_id: u32, gpu_power_level: &Arc<Mutex<Option<u32>>>, vram_power_level: &Arc<Mutex<Option<u32>>>) {
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

    let automatic_fan_control_switch: Switch =
        builder.get_object("automatic_fan_control_switch").unwrap();

    let fan_curve_frame: Frame = builder.get_object("fan_curve_frame").unwrap();

    let gpu_power_adjustment: Adjustment = builder.get_object("gpu_power_adjustment").unwrap();

    let apply_button: Button = builder.get_object("apply_button").unwrap();

    let overclocking_info_frame: Frame = builder.get_object("overclocking_info_frame").unwrap();

    let gpu_clockspeed_adjustment: Adjustment = builder.get_object("gpu_clockspeed_adjustment").unwrap();

    let vram_clockspeed_adjustment: Adjustment = builder.get_object("vram_clockspeed_adjustment").unwrap();

    let gpu_voltage_adjustment: Adjustment = builder.get_object("gpu_voltage_adjustment").unwrap();

    let vram_voltage_adjustment: Adjustment = builder.get_object("vram_voltage_adjustment").unwrap();
    
    let clocks_notebook: Notebook = builder.get_object("clocks_notebook").unwrap();

    let clocks_unsupported_label: Label = builder.get_object("clocks_unsupported_label").unwrap();

    //let power_levels_box: gtk::Box = builder.get_object("power_levels_box").unwrap();

    let power_profile_select_comboboxtext: ComboBoxText = builder
        .get_object("power_profile_select_comboboxtext")
        .unwrap();

    match fs::read_to_string("/proc/cmdline") {
        Ok(cmdline) => {
            if cmdline.contains("amdgpu.ppfeaturemask=") {
                overclocking_info_frame.set_visible(false);
            }
        }
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

    let vulkan_features = gpu_info
        .vulkan_info
        .features
        .replace(',', "\n")
        .replace("Features", "")
        .replace("{", "")
        .replace("}", "")
        .replace(" ", "")
        .replace(":", ": ");

    vulkan_device_name_text_buffer.set_text(&gpu_info.vulkan_info.device_name);
    vulkan_version_text_buffer.set_text(&gpu_info.vulkan_info.api_version);
    vulkan_features_text_buffer.set_text(&vulkan_features);

    let (power_cap, power_cap_max) = d.get_power_cap(gpu_id).unwrap();

    gpu_power_adjustment.set_upper(power_cap_max as f64);
    gpu_power_adjustment.set_value(power_cap as f64);

    match &gpu_info.power_profile {
        Some(power_profile) => {
            power_profile_select_comboboxtext.set_active(match power_profile {
                PowerProfile::Auto => Some(0),
                PowerProfile::High => Some(1),
                PowerProfile::Low => Some(2),
            });
        },
        None => {
            power_profile_select_comboboxtext.set_sensitive(false);
        }
    }

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
        }
        Err(_) => {
            automatic_fan_control_switch.set_sensitive(false);
            automatic_fan_control_switch.set_tooltip_text(Some("Unavailable"));

            fan_curve_frame.set_visible(false);
        }
    }

    match fan_control {
        Ok(fan_control) => {
            //let curve: Arc<RwLock<BTreeMap<i32, f64>>> = Arc::new(RwLock::new(fan_control.curve));

            for i in 1..6 {
                let curve_temperature_adjustment: Adjustment = builder
                    .get_object(&format!("curve_temperature_adjustment_{}", i))
                    .unwrap();

                let value = *fan_control.curve
                    .get(&(i * 20))
                    .expect("Could not get by index");
                println!("Setting value {} on adjustment {}", value, i);
                curve_temperature_adjustment.set_value(value);

                let b = apply_button.clone();

                curve_temperature_adjustment.connect_value_changed(move |_| {
                    b.set_sensitive(true);
                });
            }

        }
        Err(_) => (),
    }

    /*match gpu_info.clocks_table {
        Some(clocks_table) => {
            for (id, (clockspeed, voltage)) in clocks_table.gpu_power_levels {
                let adjustment = Adjustment::new(clockspeed as f64,
                    clocks_table.gpu_clocks_range.0 as f64,
                    clocks_table.gpu_clocks_range.1 as f64,
                    1f64, 1f64, 1f64);

                let scale = Scale::new(Orientation::Vertical, Some(&adjustment));
                power_levels_box.pack_end(&scale, true, true, 5);
            }
            power_levels_box.show_all();
        },
        None => (),
    }*/

    match gpu_info.clocks_table {
        Some(clocks_table) => {
            gpu_clockspeed_adjustment.set_lower(clocks_table.gpu_clocks_range.0 as f64);
            gpu_clockspeed_adjustment.set_upper(clocks_table.gpu_clocks_range.1 as f64);

            vram_clockspeed_adjustment.set_lower(clocks_table.mem_clocks_range.0 as f64);
            vram_clockspeed_adjustment.set_upper(clocks_table.mem_clocks_range.1 as f64);

            gpu_voltage_adjustment.set_lower(clocks_table.voltage_range.0 as f64 / 1000.0);
            gpu_voltage_adjustment.set_upper(clocks_table.voltage_range.1 as f64 / 1000.0);

            let (gpu_power_level_id, (gpu_clockspeed, gpu_voltage)) = clocks_table.gpu_power_levels.iter().next_back().unwrap();
            let (vram_power_level_id, (vram_clockspeed, vram_voltage)) = clocks_table.mem_power_levels.iter().next_back().unwrap();

            gpu_clockspeed_adjustment.set_value(*gpu_clockspeed as f64);
            vram_clockspeed_adjustment.set_value(*vram_clockspeed as f64);
            gpu_voltage_adjustment.set_value(*gpu_voltage as f64 / 1000.0);
            vram_voltage_adjustment.set_upper(*vram_voltage as f64 / 1000.0);
            vram_voltage_adjustment.set_value(*vram_voltage as f64 / 1000.0);

            gpu_power_level.lock().unwrap().replace(*gpu_power_level_id);
            vram_power_level.lock().unwrap().replace(*vram_power_level_id);
        },
        None => {
            clocks_notebook.set_visible(false);
            clocks_unsupported_label.set_visible(true);
        },
    }

    apply_button.set_sensitive(false);
}

fn main() {
    println!("Initializing gtk");
    let application = gtk::Application::new(Some("com.ilyaz.lact"), Default::default())
        .expect("failed to initialize");

    application.connect_activate(|app| {
        println!("Activating");
        build_ui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}
