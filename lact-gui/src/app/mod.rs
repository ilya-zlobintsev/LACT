mod apply_revealer;
mod header;
mod root_stack;

use std::fs;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use anyhow::Context;
use apply_revealer::ApplyRevealer;
use glib::clone;
use gtk::gio::ApplicationFlags;
use gtk::prelude::*;
use gtk::*;
use header::Header;
use lact_client::schema::DeviceStats;
use lact_client::DaemonClient;
use root_stack::RootStack;
use tracing::{debug, error, info, trace};

// In ms
const STATS_POLL_INTERVAL: u64 = 250;

#[derive(Clone)]
pub struct App {
    application: Application,
    pub window: ApplicationWindow,
    pub header: Header,
    root_stack: RootStack,
    apply_revealer: ApplyRevealer,
    daemon_client: DaemonClient,
}

impl App {
    pub fn new(daemon_client: DaemonClient) -> Self {
        let application = Application::new(None, ApplicationFlags::default());

        let header = Header::new();
        let window = ApplicationWindow::builder()
            .title("LACT")
            .default_width(500)
            .default_height(600)
            .build();

        window.set_titlebar(Some(&header.container));

        // window.connect_close_request(move |_, _| {
        //     // main_quit();
        //     Inhibit(false)
        // });

        let root_stack = RootStack::new();

        header.set_switcher_stack(&root_stack.container);

        let root_box = Box::new(Orientation::Vertical, 5);

        root_box.append(&root_stack.container);

        let apply_revealer = ApplyRevealer::new();

        root_box.append(&apply_revealer.container);

        window.set_child(Some(&root_box));

        App {
            application,
            window,
            header,
            root_stack,
            apply_revealer,
            daemon_client,
        }
    }

    pub fn run(self) -> anyhow::Result<()> {
        self.application
            .connect_activate(clone!(@strong self as app => move |_| {
                app.window.set_application(Some(&app.application));

                let current_gpu_id = Arc::new(RwLock::new(String::new()));

                {
                    let current_gpu_id = current_gpu_id.clone();

                    app.header.connect_gpu_selection_changed(clone!(@strong app => move |gpu_id| {
                        info!("GPU Selection changed");
                        app.set_info(&gpu_id);
                        *current_gpu_id.write().unwrap() = gpu_id;
                    }));
                }

                let devices_buf = app
                    .daemon_client
                    .list_devices()
                    .expect("Could not list devices");
                let devices = devices_buf.inner().expect("Could not access devices");
                app.header.set_devices(&devices);

                // Show apply button on setting changes
                {
                    let apply_revealer = app.apply_revealer.clone();

                    app.root_stack
                        .thermals_page
                        .connect_settings_changed(move || {
                            debug!("Settings changed, showing apply button");
                            apply_revealer.show();
                        });

                    let apply_revealer = app.apply_revealer.clone();

                    app.root_stack.oc_page.connect_settings_changed(move || {
                        debug!("Settings changed, showing apply button");
                        apply_revealer.show();
                    });
                }

                {
                    let app = app.clone();
                    let current_gpu_id = current_gpu_id.clone();

                    // TODO
                    /*app.root_stack.oc_page.connect_clocks_reset(move || {
                        info!("Resetting clocks, but not applying");

                        let gpu_id = current_gpu_id.load(Ordering::SeqCst);

                        app.daemon_client
                            .reset_gpu_power_states(gpu_id)
                            .expect("Failed to reset clocks");

                        app.set_info(gpu_id);

                        app.apply_revealer.show();
                    })*/
                }

                app.apply_revealer.connect_apply_button_clicked(
                    clone!(@strong app as app, @strong current_gpu_id => move || {
                        if let Err(err) =  app.apply_settings(current_gpu_id.clone()) {
                            show_error(err.context("Could not apply settings"));

                            let gpu_id = current_gpu_id.read().unwrap();
                            app.set_info(&gpu_id)
                        }
                    }),
                );

                app.start_stats_update_loop(current_gpu_id.clone());

                app.window.show();
            }));

        self.application.run();
        Ok(())
    }

    fn set_info(&self, gpu_id: &str) {
        let info_buf = self
            .daemon_client
            .get_device_info(gpu_id)
            .expect("Could not fetch info");
        let info = info_buf.inner().unwrap();
        let stats_buf = self
            .daemon_client
            .get_device_stats(gpu_id)
            .expect("Could not fetch stats");
        let stats = stats_buf.inner().unwrap();

        trace!("Setting info {info:?} and stats {stats:?}");

        self.root_stack.info_page.set_info(&info);
        self.root_stack.oc_page.set_stats(&stats, true);
        self.root_stack.thermals_page.set_stats(&stats, true);

        // trace!("Setting clocks");
        // self.root_stack.oc_page.set_info(&info);

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

                    /*if (ppfeaturemask & PP_OVERDRIVE_MASK as u64) > 0 {
                        self.root_stack.oc_page.warning_frame.hide();
                    } else {
                        self.root_stack.oc_page.warning_frame.show();
                    }*/
                }
                Err(_) => {
                    info!("Failed to read feature mask! This is expected if your system doesn't have an AMD GPU.");
                    // self.root_stack.oc_page.warning_frame.hide();
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

        thread::spawn(
            clone!(@strong self.daemon_client as daemon_client => move || loop {
                let gpu_id = current_gpu_id.read().unwrap();
                match daemon_client
                    .get_device_stats(&gpu_id)
                    .and_then(|stats| stats.inner())
                {
                    Ok(stats) => {
                        sender.send(GuiUpdateMsg::GpuStats(stats)).unwrap();
                    }
                    Err(err) => {
                        error!("Could not fetch stats: {err}");
                    }
                }
                thread::sleep(Duration::from_millis(STATS_POLL_INTERVAL));
            }),
        );

        // Receiving stats into the gui event loop

        receiver.attach(
            None,
            clone!(@strong self.root_stack as root_stack => move |msg| {
                match msg {
                    GuiUpdateMsg::GpuStats(stats) => {
                        trace!("New stats received, updating {stats:?}");
                        root_stack.info_page.set_stats(&stats);
                        root_stack.thermals_page.set_stats(&stats, false);
                        root_stack.oc_page.set_stats(&stats, false);
                    } /*GuiUpdateMsg::FanControlInfo(fan_control_info) => {
                          thermals_page.set_ventilation_info(fan_control_info)
                      }*/
                }

                glib::Continue(true)
            }),
        );
    }

    fn apply_settings(&self, current_gpu_id: Arc<RwLock<String>>) -> anyhow::Result<()> {
        info!("Applying settings");

        let gpu_id = current_gpu_id.read().unwrap();

        /*let thermals_settings = self.root_stack.thermals_page.get_thermals_settings();

        self.daemon_client
            .set_fan_control(
                &gpu_id,
                thermals_settings.automatic_fan_control_enabled,
                None,
            )
            .context("Could not set fan control")?;*/

        // TODO
        /*self.daemon_client
        .set_fan_curve(gpu_id, thermals_settings.curve)
        .unwrap_or(println!("Failed to set fan curve"));*/

        /*if let Some(clocks_settings) = self.root_stack.oc_page.get_clocks() {
            self.daemon_client
                .set_gpu_max_power_state(
                    gpu_id,
                    clocks_settings.gpu_clock,
                    Some(clocks_settings.gpu_voltage),
                )
                .expect("Failed to set GPU clockspeed/voltage");

            self.daemon_client
                .set_vram_max_clock(gpu_id, clocks_settings.vram_clock)
                .expect("Failed to set VRAM Clock");

            self.daemon_client
                .commit_gpu_power_states(gpu_id)
                .expect("Failed to commit power states");
        }

        if let Some(profile) = self.root_stack.oc_page.get_power_profile() {
            self.daemon_client
                .set_power_profile(gpu_id, profile)
                .expect("Failed to set power profile");
        }*/

        if let Some(cap) = self.root_stack.oc_page.get_power_cap() {
            self.daemon_client
                .set_power_cap(&gpu_id, Some(cap))
                .context("Failed to set power cap")?;
        }

        self.set_info(&gpu_id);

        Ok(())
    }
}

enum GuiUpdateMsg {
    // FanControlInfo(FanControlInfo),
    GpuStats(DeviceStats),
}

fn show_error(err: anyhow::Error) {
    glib::idle_add(move || {
        let text = format!("{err:?}");
        let diag = MessageDialog::builder()
            .title("Error")
            .message_type(MessageType::Error)
            .text(&text)
            .buttons(ButtonsType::Close)
            .build();
        diag.set_modal(true);
        diag.hide();
        glib::Continue(false)
    });
}
