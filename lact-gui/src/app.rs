mod apply_revealer;
mod header;
mod root_stack;

use std::fs;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use crate::client::DaemonClient;
use apply_revealer::ApplyRevealer;
use gtk::prelude::*;
use gtk::*;
use header::Header;
use lact_schema::DeviceStats;
use root_stack::RootStack;
use tracing::{debug, error, info, trace};

#[derive(Clone)]
pub struct App {
    pub window: Window,
    pub header: Header,
    root_stack: RootStack,
    apply_revealer: ApplyRevealer,
    daemon_client: DaemonClient,
}

impl App {
    pub fn new(daemon_client: DaemonClient) -> Self {
        let window = Window::new(WindowType::Toplevel);

        let header = Header::new();

        window.set_titlebar(Some(&header.container));
        window.set_title("LACT");

        window.set_default_size(500, 600);

        window.connect_delete_event(move |_, _| {
            main_quit();
            Inhibit(false)
        });

        let root_stack = RootStack::new();

        header.set_switcher_stack(&root_stack.container);

        let root_box = Box::new(Orientation::Vertical, 5);

        root_box.add(&root_stack.container);

        let apply_revealer = ApplyRevealer::new();

        root_box.add(&apply_revealer.container);

        window.add(&root_box);

        App {
            window,
            header,
            root_stack,
            apply_revealer,
            daemon_client,
        }
    }

    pub fn run(&self) -> anyhow::Result<()> {
        self.window.show_all();

        let current_gpu_id = Arc::new(RwLock::new(String::new()));

        {
            let current_gpu_id = current_gpu_id.clone();
            let app = self.clone();

            self.header.connect_gpu_selection_changed(move |gpu_id| {
                info!("GPU Selection changed");
                app.set_info(&gpu_id);
                *current_gpu_id.write().unwrap() = gpu_id;
            });
        }

        let devices = self.daemon_client.list_devices()?;
        self.header.set_devices(&devices);

        // Show apply button on setting changes
        {
            let apply_revealer = self.apply_revealer.clone();

            self.root_stack
                .thermals_page
                .connect_settings_changed(move || {
                    debug!("Settings changed, showing apply button");
                    apply_revealer.show();
                });

            let apply_revealer = self.apply_revealer.clone();

            self.root_stack.oc_page.connect_settings_changed(move || {
                debug!("Settings changed, showing apply button");
                apply_revealer.show();
            });
        }

        {
            let app = self.clone();
            let current_gpu_id = current_gpu_id.clone();

            // TODO
            /*self.root_stack.oc_page.connect_clocks_reset(move || {
                info!("Resetting clocks, but not applying");

                let gpu_id = current_gpu_id.load(Ordering::SeqCst);

                app.daemon_client
                    .reset_gpu_power_states(gpu_id)
                    .expect("Failed to reset clocks");

                app.set_info(gpu_id);

                app.apply_revealer.show();
            })*/
        }

        // Apply settings
        {
            let current_gpu_id = current_gpu_id.clone();
            let app = self.clone();

            self.apply_revealer.connect_apply_button_clicked(move || {
                info!("Applying settings");

                let gpu_id = current_gpu_id.read().unwrap();

                {
                    let thermals_settings = app.root_stack.thermals_page.get_thermals_settings();

                    app.daemon_client
                        .set_fan_control(&gpu_id, thermals_settings.automatic_fan_control_enabled)
                        .expect("Could not set fan control");

                    // TODO
                    /*app.daemon_client
                    .set_fan_curve(gpu_id, thermals_settings.curve)
                    .unwrap_or(println!("Failed to set fan curve"));*/
                }

                /*if let Some(clocks_settings) = app.root_stack.oc_page.get_clocks() {
                    app.daemon_client
                        .set_gpu_max_power_state(
                            gpu_id,
                            clocks_settings.gpu_clock,
                            Some(clocks_settings.gpu_voltage),
                        )
                        .expect("Failed to set GPU clockspeed/voltage");

                    app.daemon_client
                        .set_vram_max_clock(gpu_id, clocks_settings.vram_clock)
                        .expect("Failed to set VRAM Clock");

                    app.daemon_client
                        .commit_gpu_power_states(gpu_id)
                        .expect("Failed to commit power states");
                }

                if let Some(profile) = app.root_stack.oc_page.get_power_profile() {
                    app.daemon_client
                        .set_power_profile(gpu_id, profile)
                        .expect("Failed to set power profile");
                }

                if let Some(cap) = app.root_stack.oc_page.get_power_cap() {
                    app.daemon_client
                        .set_power_cap(gpu_id, cap)
                        .expect("Failed to set power cap");
                }*/

                app.set_info(&gpu_id);
            });
        }

        self.start_stats_update_loop(current_gpu_id.clone());

        Ok(gtk::main())
    }

    fn set_info(&self, gpu_id: &str) {
        let info = self.daemon_client.get_device_info(gpu_id).unwrap();
        trace!("Setting info {info:?}");

        self.root_stack.info_page.set_info(&info);

        trace!("Setting clocks");
        self.root_stack.oc_page.set_info(&info);

        // TODO: this should be stats
        /*trace!("Setting performance level {:?}", info.power_profile);
        self.root_stack
            .oc_page
            .set_power_profile(&gpu_info.power_profile);

        log::trace!("Setting fan control info");
        match self.daemon_client.get_fan_control(gpu_id) {
            Ok(fan_control_info) => self
                .root_stack
                .thermals_page
                .set_ventilation_info(fan_control_info),
            Err(_) => self.root_stack.thermals_page.hide_fan_controls(),
        }*/

        {
            // It's overkill to both show and hide the frame, but it needs to be done in set_info because show_all overrides the default hidden state of the frame.
            match fs::read_to_string("/sys/module/amdgpu/parameters/ppfeaturemask") {
                Ok(ppfeaturemask) => {
                    const PP_OVERDRIVE_MASK: i32 = 0x4000;

                    let ppfeaturemask = ppfeaturemask.trim().strip_prefix("0x").unwrap();

                    trace!("ppfeaturemask {}", ppfeaturemask);

                    let ppfeaturemask: u64 =
                        u64::from_str_radix(ppfeaturemask, 16).expect("Invalid ppfeaturemask");

                    if (ppfeaturemask & PP_OVERDRIVE_MASK as u64) > 0 {
                        self.root_stack.oc_page.warning_frame.hide();
                    } else {
                        self.root_stack.oc_page.warning_frame.show();
                    }
                }
                Err(_) => {
                    info!("Failed to read feature mask! This is expected if your system doesn't have an AMD GPU.");
                    self.root_stack.oc_page.warning_frame.hide();
                }
            }
        }

        self.apply_revealer.hide();
    }

    fn start_stats_update_loop(&self, current_gpu_id: Arc<RwLock<String>>) {
        let context = glib::MainContext::default();

        let _guard = context.acquire();

        // The loop that gets stats
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        {
            let daemon_connection = self.daemon_client.clone();

            thread::spawn(move || loop {
                let gpu_id = current_gpu_id.read().unwrap();
                match daemon_connection.get_device_stats(&gpu_id) {
                    Ok(stats) => {
                        sender.send(GuiUpdateMsg::GpuStats(stats)).unwrap();
                    }
                    Err(err) => {
                        error!("Could not fetch stats: {err}");
                    }
                }
                thread::sleep(Duration::from_millis(500));
            });
        }

        // Receiving stats into the gui event loop
        {
            let thermals_page = self.root_stack.thermals_page.clone();
            let oc_page = self.root_stack.oc_page.clone();

            receiver.attach(None, move |msg| {
                match msg {
                    GuiUpdateMsg::GpuStats(stats) => {
                        trace!("New stats received, updating {stats:?}");
                        thermals_page.set_stats(&stats);
                        oc_page.set_stats(&stats);
                    } /*GuiUpdateMsg::FanControlInfo(fan_control_info) => {
                          thermals_page.set_ventilation_info(fan_control_info)
                      }*/
                }

                glib::Continue(true)
            });
        }
    }
}

enum GuiUpdateMsg {
    // FanControlInfo(FanControlInfo),
    GpuStats(DeviceStats),
}
