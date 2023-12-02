mod apply_box;
mod dialogs;
mod gpu_selector;
mod headerbar;
mod info_row;
mod page_section;
mod root_stack;

use self::apply_box::ApplyBox;
use self::headerbar::Headerbar;
use crate::app::dialogs::show_error;
use crate::{info_dialog, APP_ID, GUI_VERSION};
use anyhow::{anyhow, Context};
use glib::clone;
use gpu_selector::GpuSelector;
use gtk::gio::ActionEntry;
use gtk::glib::{timeout_future, ControlFlow};
use gtk::{gio::ApplicationFlags, prelude::*, *};
use lact_client::schema::request::{ConfirmCommand, SetClocksCommand};
use lact_client::schema::DeviceStats;
use lact_client::DaemonClient;
use lact_daemon::MODULE_CONF_PATH;
use root_stack::RootStack;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use tracing::{debug, error, trace, warn};

#[cfg(feature = "adw")]
use adw::prelude::{AdwApplicationWindowExt, MessageDialogExt};

// In ms
const STATS_POLL_INTERVAL: u64 = 250;

#[derive(Clone)]
pub struct App {
    #[cfg(feature = "adw")]
    application: adw::Application,
    #[cfg(feature = "adw")]
    pub window: adw::ApplicationWindow,

    #[cfg(not(feature = "adw"))]
    application: Application,
    #[cfg(not(feature = "adw"))]
    pub window: ApplicationWindow,

    pub gpu_selector: GpuSelector,
    root_stack: RootStack,
    apply_box: ApplyBox,
    daemon_client: DaemonClient,
}

impl App {
    pub fn new(daemon_client: DaemonClient) -> Self {
        #[cfg(feature = "adw")]
        let application = adw::Application::new(Some(APP_ID), ApplicationFlags::default());

        #[cfg(not(feature = "adw"))]
        let application = Application::new(Some(APP_ID), ApplicationFlags::default());

        #[cfg(feature = "adw")]
        let window = adw::ApplicationWindow::builder()
            .title("LACT")
            .default_width(820)
            .default_height(750)
            .width_request(420)
            .height_request(200)
            .icon_name(APP_ID)
            .build();

        #[cfg(not(feature = "adw"))]
        let window = ApplicationWindow::builder()
            .title("LACT")
            .default_width(820)
            .default_height(750)
            .width_request(420)
            .height_request(200)
            .icon_name(APP_ID)
            .build();

        window.add_action_entries([ActionEntry::builder("quit")
            .activate(clone!(@weak application => move |_, _, _| {
                application.quit();
            }))
            .build()]);
        application.set_accels_for_action("win.quit", &["<Primary>Q"]);

        let system_info_buf = daemon_client
            .get_system_info()
            .expect("Could not fetch system info");
        let system_info = system_info_buf.inner().expect("Invalid system info buffer");

        if system_info.version != GUI_VERSION {
            let err = anyhow!("Version mismatch between GUI and daemon ({GUI_VERSION} vs {})! Make sure you have restarted the service if you have updated LACT.", system_info.version);
            show_error(&window, err);
        }

        let root_stack = RootStack::new(&window, system_info, daemon_client.embedded);

        let headerbar = Headerbar::new(&application, &window.clone().upcast::<ApplicationWindow>());

        #[cfg(feature = "adw")]
        {
            let root_view = adw::ToolbarView::new();
            root_view.add_top_bar(&headerbar.container);
            root_view.add_bottom_bar(
                &adw::ViewSwitcherBar::builder()
                    .reveal(true)
                    .stack(&root_stack.container)
                    .build(),
            );
            root_view.set_content(Some(&root_stack.container));
            window.set_content(Some(&root_view));
        }

        #[cfg(not(feature = "adw"))]
        {
            let root_view = Box::builder().orientation(Orientation::Vertical).build();
            root_view.append(&root_stack.container);
            root_view.append(&Separator::new(Orientation::Horizontal));
            root_view.append(
                &StackSwitcher::builder()
                    .stack(&root_stack.container)
                    .halign(Align::Center)
                    .vexpand(false)
                    .hexpand(true)
                    .build(),
            );
            window.set_titlebar(Some(&headerbar.container));
            window.set_child(Some(&root_view));
        }

        App {
            application,
            window,
            gpu_selector: headerbar.gpu_selector,
            root_stack,
            apply_box: headerbar.apply_box,
            daemon_client,
        }
    }

    pub fn run(self, connection_err: Option<anyhow::Error>) -> anyhow::Result<()> {
        self.application
            .connect_activate(clone!(@strong self as app => move |_| {
                app.window.set_application(Some(&app.application));

                let current_gpu_id = Rc::new(RefCell::new(String::new()));

                app.gpu_selector.connect_gpu_selection_changed(clone!(@strong app, @strong current_gpu_id => move |gpu_id| {
                    debug!("GPU Selection changed");
                    app.set_info(&gpu_id);
                    *current_gpu_id.borrow_mut() = gpu_id;
                    debug!("Updated current GPU id");
                }));

                let devices_buf = app
                    .daemon_client
                    .list_devices()
                    .expect("Could not list devices");
                let devices = devices_buf.inner().expect("Could not access devices");
                app.gpu_selector.set_devices(&devices);

                app.root_stack.oc_page.clocks_frame.connect_clocks_reset(clone!(@strong app, @strong current_gpu_id => move || {
                    debug!("Resetting clocks");

                    let gpu_id = current_gpu_id.borrow().clone();

                    match app.daemon_client.set_clocks_value(&gpu_id, SetClocksCommand::Reset)
                        .and_then(|_| app.daemon_client.confirm_pending_config(ConfirmCommand::Confirm))
                    {
                        Ok(()) => {
                            app.set_initial(&gpu_id);
                        }
                        Err(err) => {
                            show_error(&app.window, err);
                        }
                    }
                }));

                app.apply_box.connect_apply_button_clicked(
                    clone!(@strong app, @strong current_gpu_id => move || {
                        glib::idle_add_local_once(clone!(@strong app, @strong current_gpu_id => move || {
                            if let Err(err) = app.apply_settings(current_gpu_id.clone()) {
                                show_error(&app.window, err.context("Could not apply settings"));

                                glib::idle_add_local_once(clone!(@strong app, @strong current_gpu_id => move || {
                                    let gpu_id = current_gpu_id.borrow().clone();
                                    app.set_initial(&gpu_id)
                                }));
                            }
                        }));
                    }),
                );
                app.apply_box.connect_reset_button_clicked(clone!(@strong app, @strong current_gpu_id => move || {
                    let gpu_id = current_gpu_id.borrow().clone();
                    app.set_initial(&gpu_id)
                }));

                if let Some(ref button) = app.root_stack.oc_page.enable_overclocking_button {
                    button.connect_clicked(clone!(@strong app => move |_| {
                        app.enable_overclocking();
                    }));
                }

                app.start_stats_update_loop(current_gpu_id);

                app.window.show();

                if app.daemon_client.embedded {
                    let error_text = connection_err.as_ref().map(|err| {
                        format!("Error info: {err:#}\n\n")
                    }).unwrap_or_default();

                    let enable_text = "sudo systemctl enable --now lactd";

                    let enable_label = Entry::builder()
                        .text(enable_text)
                        .css_classes(["card"])
                        .editable(false)
                        .hexpand(true)
                        .build();

                    let hbox = Box::builder()
                        .orientation(Orientation::Horizontal)
                        .spacing(6)
                        .build();
                    let copy_btn = Button::builder()
                        .css_classes(["circular", "flat"])
                        .tooltip_text("Copy")
                        .icon_name("edit-copy-symbolic")
                        .build();

                    copy_btn.connect_clicked(move |_| {
                        match gdk::Display::default() {
                            None => eprintln!("Failed to get default gdk display"),
                            Some(d) => d.clipboard().set_text(&enable_text)
                        }
                    });

                    hbox.append(&enable_label);
                    hbox.append(&copy_btn);

                    let diag = info_dialog!(
                        &app.window,
                        "Could not connect to daemon",
                        format!("Running in embedded mode.\n\
                            Please make sure the lactd service is running.\n\
                            Using embedded mode, you will not be able to change any settings.\n\n\
                            {error_text}\
                            To enable the daemon, run the following command:"),
                        "close",
                        "_Close");

                    #[cfg(feature = "adw")]
                    diag.set_extra_child(Some(&hbox));

                    #[cfg(not(feature = "adw"))]
                    {
                        hbox.set_margin_start(12);
                        hbox.set_margin_end(12);
                        diag.first_child().unwrap().first_child().unwrap().downcast::<Box>().unwrap().append(&hbox);
                    }
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
        debug!("setting initial stats for gpu {gpu_id}");
        let stats_buf = self
            .daemon_client
            .get_device_stats(gpu_id)
            .expect("Could not fetch stats");
        let stats = stats_buf.inner().unwrap();

        self.root_stack.oc_page.set_stats(&stats, true);
        self.root_stack.thermals_page.set_stats(&stats, true);
        self.root_stack.info_page.set_stats(&stats);

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

        let maybe_modes_table = match self.daemon_client.get_device_power_profile_modes(gpu_id) {
            Ok(buf) => match buf.inner() {
                Ok(table) => Some(table),
                Err(err) => {
                    debug!("Could not extract profile modes table: {err:?}");
                    None
                }
            },
            Err(err) => {
                debug!("Could not get profile modes table: {err:?}");
                None
            }
        };
        self.root_stack
            .oc_page
            .performance_frame
            .set_power_profile_modes(maybe_modes_table);

        match self
            .daemon_client
            .get_power_states(gpu_id)
            .and_then(|states| states.inner())
        {
            Ok(power_states) => {
                self.root_stack
                    .oc_page
                    .power_states_frame
                    .set_power_states(power_states);
            }
            Err(err) => warn!("could not get power states: {err:?}"),
        }

        // Show apply button on setting changes
        // This is done here because new widgets may appear after applying settings (like fan curve points) which should be connected
        let show_revealer = clone!(@strong self.apply_box as apply_box => move || {
                debug!("settings changed, showing apply button");
                apply_box.show();
        });

        self.root_stack
            .thermals_page
            .connect_settings_changed(show_revealer.clone());

        self.root_stack
            .oc_page
            .connect_settings_changed(show_revealer);

        self.apply_box.hide();
    }

    fn start_stats_update_loop(&self, current_gpu_id: Rc<RefCell<String>>) {
        let context = glib::MainContext::default();

        let _guard = context.acquire();

        // The loop that gets stats
        let (sender, receiver) = glib::MainContext::channel(glib::Priority::default());

        context.spawn_local(
            clone!(@strong self.daemon_client as daemon_client => async move {
                loop {
                    {
                        let gpu_id = current_gpu_id.borrow();
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
                    }
                    timeout_future(Duration::from_millis(STATS_POLL_INTERVAL)).await;
                }
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

                ControlFlow::Continue
            }),
        );
    }

    fn apply_settings(&self, current_gpu_id: Rc<RefCell<String>>) -> anyhow::Result<()> {
        // TODO: Ask confirmation for everything, not just clocks

        debug!("applying settings");
        let gpu_id = current_gpu_id.borrow().clone();
        debug!("using gpu {gpu_id}");

        if let Some(cap) = self.root_stack.oc_page.get_power_cap() {
            self.daemon_client
                .set_power_cap(&gpu_id, Some(cap))
                .context("Failed to set power cap")?;

            self.daemon_client
                .confirm_pending_config(ConfirmCommand::Confirm)
                .context("Could not commit config")?;
        }

        // Reset the power profile mode for switching to/from manual performance level
        self.daemon_client
            .set_power_profile_mode(&gpu_id, None)
            .context("Could not set default power profile mode")?;
        self.daemon_client
            .confirm_pending_config(ConfirmCommand::Confirm)
            .context("Could not commit config")?;

        if let Some(level) = self.root_stack.oc_page.get_performance_level() {
            self.daemon_client
                .set_performance_level(&gpu_id, level)
                .context("Failed to set power profile")?;
            self.daemon_client
                .confirm_pending_config(ConfirmCommand::Confirm)
                .context("Could not commit config")?;

            let mode_index = self
                .root_stack
                .oc_page
                .performance_frame
                .get_selected_power_profile_mode();
            self.daemon_client
                .set_power_profile_mode(&gpu_id, mode_index)
                .context("Could not set active power profile mode")?;
            self.daemon_client
                .confirm_pending_config(ConfirmCommand::Confirm)
                .context("Could not commit config")?;
        }

        if let Some(thermals_settings) = self.root_stack.thermals_page.get_thermals_settings() {
            debug!("applying thermal settings: {thermals_settings:?}");

            self.daemon_client
                .set_fan_control(
                    &gpu_id,
                    thermals_settings.manual_fan_control,
                    thermals_settings.mode,
                    thermals_settings.static_speed,
                    thermals_settings.curve,
                )
                .context("Could not set fan control")?;
            self.daemon_client
                .confirm_pending_config(ConfirmCommand::Confirm)
                .context("Could not commit config")?;
        }

        let clocks_settings = self.root_stack.oc_page.clocks_frame.get_settings();
        let mut clocks_commands = Vec::new();

        debug!("applying clocks settings {clocks_settings:#?}");

        if let Some(clock) = clocks_settings.min_core_clock {
            clocks_commands.push(SetClocksCommand::MinCoreClock(clock));
        }

        if let Some(clock) = clocks_settings.min_memory_clock {
            clocks_commands.push(SetClocksCommand::MinMemoryClock(clock));
        }

        if let Some(voltage) = clocks_settings.min_voltage {
            clocks_commands.push(SetClocksCommand::MinVoltage(voltage));
        }

        if let Some(clock) = clocks_settings.max_core_clock {
            clocks_commands.push(SetClocksCommand::MaxCoreClock(clock));
        }

        if let Some(clock) = clocks_settings.max_memory_clock {
            clocks_commands.push(SetClocksCommand::MaxMemoryClock(clock));
        }

        if let Some(voltage) = clocks_settings.max_voltage {
            clocks_commands.push(SetClocksCommand::MaxVoltage(voltage));
        }

        if let Some(offset) = clocks_settings.voltage_offset {
            clocks_commands.push(SetClocksCommand::VoltageOffset(offset));
        }

        let enabled_power_states = self.root_stack.oc_page.get_enabled_power_states();

        for (kind, states) in enabled_power_states {
            if !states.is_empty() {
                self.daemon_client
                    .set_enabled_power_states(&gpu_id, kind, states)
                    .context("Could not set power states")?;

                self.daemon_client
                    .confirm_pending_config(ConfirmCommand::Confirm)
                    .context("Could not commit config")?;
            }
        }

        if !clocks_commands.is_empty() {
            let delay = self
                .daemon_client
                .batch_set_clocks_value(&gpu_id, clocks_commands)
                .context("Could not commit clocks settins")?;
            self.ask_confirmation(gpu_id.clone(), delay);
        }

        self.set_initial(&gpu_id);

        Ok(())
    }

    #[cfg(feature = "adw")]
    fn enable_overclocking(&self) {
        let text = format!("This will enable the overdrive feature of the amdgpu driver by creating a file at <b>{MODULE_CONF_PATH}</b>");
        let dialog = adw::MessageDialog::builder()
            .heading("Enable Overclocking")
            .body_use_markup(true)
            .body(text)
            .modal(true)
            .transient_for(&self.window)
            .build();

        let res_ok = "ok";
        let res_cancel = "cancel";

        dialog.add_response(res_cancel, "_Cancel");
        dialog.add_response(res_ok, "_Ok");
        dialog.set_response_appearance(res_cancel, adw::ResponseAppearance::Destructive);
        dialog.set_response_appearance(res_ok, adw::ResponseAppearance::Suggested);

        dialog.connect_response(
            None,
            clone!(@strong self as app => move |_, response| {
                if response == res_ok {
                    match app.daemon_client.enable_overdrive().and_then(|buffer| buffer.inner()) {
                        Ok(_) => {
                            info_dialog!(
                                &app.window,
                                "Success",
                                concat!(
                                    "Overclocking successfully enabled. ",
                                    "A system reboot is required to apply the changes"),
                                "ok",
                                "_Ok");
                        }
                        Err(err) => {
                            show_error(&app.window, err);
                        }
                    }
                }
            }),
        );

        dialog.present();
    }

    #[cfg(not(feature = "adw"))]
    fn enable_overclocking(&self) {
        let text = format!("This will enable the overdrive feature of the amdgpu driver by creating a file at <b>{MODULE_CONF_PATH}</b>. Are you sure you want to do this?");
        let dialog = MessageDialog::builder()
            .title("Enable Overclocking")
            .use_markup(true)
            .text(text)
            .message_type(MessageType::Question)
            .buttons(ButtonsType::OkCancel)
            .transient_for(&self.window)
            .build();

        dialog.run_async(clone!(@strong self as app => move |diag, response| {
            if response == ResponseType::Ok {
                match app.daemon_client.enable_overdrive().and_then(|buffer| buffer.inner()) {
                    Ok(_) => {
                        let success_dialog = MessageDialog::builder()
                            .title("Success")
                            .text("Overclocking successfully enabled. A system reboot is required to apply the changes")
                            .message_type(MessageType::Info)
                            .buttons(ButtonsType::Ok)
                            .build();
                        success_dialog.run_async(move |diag, _| {
                            diag.hide();
                        });
                    }
                    Err(err) => {
                        show_error(&app.window, err);
                    }
                }
            }
            diag.hide();
        }));
    }

    #[cfg(feature = "adw")]
    fn ask_confirmation(&self, gpu_id: String, mut delay: u64) {
        let text = confirmation_text(delay);
        let dialog = adw::MessageDialog::builder()
            .heading("Confirm settings")
            .body(text)
            .modal(true)
            .transient_for(&self.window)
            .build();

        let res_yes = "yes";
        let res_no = "no";

        dialog.add_response(res_no, "_No");
        dialog.add_response(res_yes, "_Yes");
        dialog.set_response_appearance(res_no, adw::ResponseAppearance::Destructive);
        dialog.set_response_appearance(res_yes, adw::ResponseAppearance::Suggested);
        let confirmed = Rc::new(AtomicBool::new(false));

        glib::source::timeout_add_local(
            Duration::from_secs(1),
            clone!(@strong dialog, @strong self as app, @strong gpu_id, @strong confirmed => move || {
                if confirmed.load(std::sync::atomic::Ordering::SeqCst) {
                    return ControlFlow::Break;

                }
                delay -= 1;

                let text = confirmation_text(delay);
                dialog.set_body(&text);

                if delay == 0 {
                    dialog.hide();
                    app.set_initial(&gpu_id);

                    ControlFlow::Break
                }  else {
                    ControlFlow::Continue
                }
            }),
        );

        dialog.connect_response(
            None,
            clone!(@strong self as app => move |diag, response| {
                confirmed.store(true, std::sync::atomic::Ordering::SeqCst);

                let command = match response {
                    res if res == res_yes => ConfirmCommand::Confirm,
                    _ => ConfirmCommand::Revert,
                };

                diag.hide();

                if let Err(err) = app.daemon_client.confirm_pending_config(command) {
                    show_error(&app.window, err);
                }
                app.set_initial(&gpu_id);
            }),
        );
        dialog.present();
    }

    #[cfg(not(feature = "adw"))]
    fn ask_confirmation(&self, gpu_id: String, mut delay: u64) {
        let text = confirmation_text(delay);
        let dialog = MessageDialog::builder()
            .title("Confirm settings")
            .text(text)
            .message_type(MessageType::Question)
            .buttons(ButtonsType::YesNo)
            .transient_for(&self.window)
            .build();
        let confirmed = Rc::new(AtomicBool::new(false));

        glib::source::timeout_add_local(
            Duration::from_secs(1),
            clone!(@strong dialog, @strong self as app, @strong gpu_id, @strong confirmed => move || {
                if confirmed.load(std::sync::atomic::Ordering::SeqCst) {
                    return ControlFlow::Break;

                }
                delay -= 1;

                let text = confirmation_text(delay);
                dialog.set_text(Some(&text));

                if delay == 0 {
                    dialog.hide();
                    app.set_initial(&gpu_id);

                    ControlFlow::Break
                }  else {
                    ControlFlow::Continue
                }
            }),
        );

        dialog.run_async(clone!(@strong self as app => move |diag, response| {
            confirmed.store(true, std::sync::atomic::Ordering::SeqCst);

            let command = match response {
                ResponseType::Yes => ConfirmCommand::Confirm,
                _ => ConfirmCommand::Revert,
            };

            diag.hide();

            if let Err(err) = app.daemon_client.confirm_pending_config(command) {
                show_error(&app.window, err);
            }
            app.set_initial(&gpu_id);
        }));
    }
}

enum GuiUpdateMsg {
    GpuStats(DeviceStats),
}

fn confirmation_text(seconds_left: u64) -> String {
    format!("Do you want to keep the new settings? (Reverting in {seconds_left} seconds)")
}
