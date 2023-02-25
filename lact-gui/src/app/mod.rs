mod apply_revealer;
mod header;
mod root_stack;

use crate::APP_ID;
use anyhow::{anyhow, Context};
use apply_revealer::ApplyRevealer;
use glib::clone;
use gtk::{gio::ApplicationFlags, prelude::*, *};
use header::Header;
use lact_client::schema::request::SetClocksCommand;
use lact_client::schema::DeviceStats;
use lact_client::DaemonClient;
use root_stack::RootStack;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use tracing::{debug, error, trace, warn};

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
        let application = Application::new(Some(APP_ID), ApplicationFlags::default());

        let header = Header::new();
        let window = ApplicationWindow::builder()
            .title("LACT")
            .default_width(500)
            .default_height(600)
            .icon_name(APP_ID)
            .build();

        window.set_titlebar(Some(&header.container));

        let system_info_buf = daemon_client
            .get_system_info()
            .expect("Could not fetch system info");
        let system_info = system_info_buf.inner().expect("Invalid system info buffer");
        let root_stack = RootStack::new(system_info, daemon_client.embedded);

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


                    app.header.connect_gpu_selection_changed(clone!(@strong app, @strong current_gpu_id => move |gpu_id| {
                        debug!("GPU Selection changed");
                        app.set_info(&gpu_id);
                        *current_gpu_id.write().unwrap() = gpu_id;
                        debug!("Updated current GPU id");
                    }));

                let devices_buf = app
                    .daemon_client
                    .list_devices()
                    .expect("Could not list devices");
                let devices = devices_buf.inner().expect("Could not access devices");
                app.header.set_devices(&devices);


                app.root_stack.oc_page.clocks_frame.connect_clocks_reset(clone!(@strong app, @strong current_gpu_id => move || {
                    debug!("Resetting clocks");

                    let gpu_id = current_gpu_id.read().unwrap();

                    match app.daemon_client.set_clocks_value(&gpu_id, SetClocksCommand::Reset) {
                        Ok(()) => {
                            app.set_initial(&gpu_id);
                        }
                        Err(err) => {
                            show_error(&app.window, err);
                        }
                    }
                }));

                app.apply_revealer.connect_apply_button_clicked(
                    clone!(@strong app, @strong current_gpu_id => move || {
                        if let Err(err) = app.apply_settings(current_gpu_id.clone()) {
                            show_error(&app.window, err.context("Could not apply settings"));

                            glib::idle_add_local_once(clone!(@strong app, @strong current_gpu_id => move || {
                                let gpu_id = current_gpu_id.read().unwrap();
                                app.set_initial(&gpu_id)
                            }));
                        }
                    }),
                );
                app.apply_revealer.connect_reset_button_clicked(clone!(@strong app, @strong current_gpu_id => move || {
                    let gpu_id = current_gpu_id.read().unwrap();
                    app.set_initial(&gpu_id)
                }));

                app.start_stats_update_loop(current_gpu_id);

                app.window.show();

                if app.daemon_client.embedded {
                    show_error(&app.window, anyhow!(
                        "Could not connect to daemon, running in embedded mode. \n\
                        Please make sure the lactd service is running. \n\
                        Using embedded mode, you will not be able to change any settings."
                    ));
                }
            }));

        // Args are passed manually since they were already processed by clap before
        self.application.run_with_args::<String>(&[]);
        Ok(())
    }

    fn set_info(&self, gpu_id: &str) {
        let info_buf = self
            .daemon_client
            .get_device_info(gpu_id)
            .expect("Could not fetch info");
        let info = info_buf.inner().unwrap();

        trace!("setting info {info:?}");

        self.root_stack.info_page.set_info(&info);

        self.set_initial(gpu_id);
    }

    fn set_initial(&self, gpu_id: &str) {
        let stats_buf = self
            .daemon_client
            .get_device_stats(gpu_id)
            .expect("Could not fetch stats");
        let stats = stats_buf.inner().unwrap();

        self.root_stack.oc_page.set_stats(&stats, true);
        self.root_stack.thermals_page.set_stats(&stats, true);

        let maybe_clocks_table = match self.daemon_client.get_device_clocks_info(gpu_id) {
            Ok(clocks_buf) => match clocks_buf.inner() {
                Ok(info) => info.table,
                Err(err) => {
                    debug!("could not extract clocks info: {err:?}");
                    None
                }
            },
            Err(err) => {
                debug!("could not fetch clocks info: {err:?}");
                None
            }
        };
        self.root_stack.oc_page.set_clocks_table(maybe_clocks_table);

        // Show apply button on setting changes
        // This is done here because new widgets may appear after applying settings (like fan curve points) which should be connected
        let show_revealer = clone!(@strong self.apply_revealer as apply_revealer => move || {
                debug!("settings changed, showing apply button");
                apply_revealer.show();
        });

        self.root_stack
            .thermals_page
            .connect_settings_changed(show_revealer.clone());

        self.root_stack
            .oc_page
            .connect_settings_changed(show_revealer);

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
                drop(gpu_id);
                thread::sleep(Duration::from_millis(STATS_POLL_INTERVAL));
            }),
        );

        // Receiving stats into the gui event loop

        receiver.attach(
            None,
            clone!(@strong self.root_stack as root_stack => move |msg| {
                match msg {
                    GuiUpdateMsg::GpuStats(stats) => {
                        trace!("new stats received, updating {stats:?}");
                        root_stack.info_page.set_stats(&stats);
                        root_stack.thermals_page.set_stats(&stats, false);
                        root_stack.oc_page.set_stats(&stats, false);
                    }
                }

                glib::Continue(true)
            }),
        );
    }

    fn apply_settings(&self, current_gpu_id: Arc<RwLock<String>>) -> anyhow::Result<()> {
        debug!("applying settings");

        let gpu_id = current_gpu_id.read().unwrap();

        if let Some(cap) = self.root_stack.oc_page.get_power_cap() {
            self.daemon_client
                .set_power_cap(&gpu_id, Some(cap))
                .context("Failed to set power cap")?;
        }

        if let Some(level) = self.root_stack.oc_page.get_performance_level() {
            self.daemon_client
                .set_performance_level(&gpu_id, level)
                .context("Failed to set power profile")?;
        }

        if let Some(thermals_settings) = self.root_stack.thermals_page.get_thermals_settings() {
            debug!("applying thermal settings: {thermals_settings:?}");

            self.daemon_client
                .set_fan_control(
                    &gpu_id,
                    thermals_settings.manual_fan_control,
                    thermals_settings.curve,
                )
                .context("Could not set fan control")?;
        }

        let clocks_settings = self.root_stack.oc_page.clocks_frame.get_settings();

        if let Some(clock) = clocks_settings.max_core_clock {
            self.daemon_client
                .set_clocks_value(&gpu_id, SetClocksCommand::MaxCoreClock(clock))
                .context("Could not set the maximum core clock")?;
        }

        if let Some(clock) = clocks_settings.max_memory_clock {
            self.daemon_client
                .set_clocks_value(&gpu_id, SetClocksCommand::MaxMemoryClock(clock))
                .context("Could not set the maximum memory clock")?;
        }

        if let Some(voltage) = clocks_settings.max_voltage {
            self.daemon_client
                .set_clocks_value(&gpu_id, SetClocksCommand::MaxVoltage(voltage))
                .context("Could not set the maximum voltage")?;
        }

        self.set_initial(&gpu_id);

        Ok(())
    }
}

enum GuiUpdateMsg {
    GpuStats(DeviceStats),
}

fn show_error(parent: &ApplicationWindow, err: anyhow::Error) {
    let text = format!("{err:?}");
    warn!("{}", text.trim());
    let diag = MessageDialog::builder()
        .title("Error")
        .message_type(MessageType::Error)
        .text(&text)
        .buttons(ButtonsType::Close)
        .transient_for(parent)
        .build();
    diag.run_async(|diag, _| {
        diag.hide();
    })
}
