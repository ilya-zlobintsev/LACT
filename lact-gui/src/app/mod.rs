mod apply_revealer;
mod graphs_window;
mod header;
mod info_row;
mod page_section;
mod root_stack;

#[cfg(feature = "bench")]
pub use graphs_window::plot::{Plot, PlotData};

use self::graphs_window::GraphsWindow;
use crate::{create_connection, APP_ID, GUI_VERSION};
use anyhow::{anyhow, Context};
use apply_revealer::ApplyRevealer;
use glib::clone;
use gtk::gio::ActionEntry;
use gtk::glib::{timeout_future, ControlFlow};
use gtk::{gio::ApplicationFlags, prelude::*, *};
use header::Header;
use lact_client::schema::request::{ConfirmCommand, SetClocksCommand};
use lact_client::schema::{FanOptions, GIT_COMMIT};
use lact_client::DaemonClient;
use lact_daemon::MODULE_CONF_PATH;
use root_stack::RootStack;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use tracing::{debug, error, trace, warn};

// In ms
const STATS_POLL_INTERVAL: u64 = 250;

#[derive(Clone)]
pub(crate) struct App {
    application: Application,
    pub window: ApplicationWindow,
    pub header: Header,
    root_stack: RootStack,
    apply_revealer: ApplyRevealer,
    daemon_client: DaemonClient,
    graphs_window: GraphsWindow,
}

impl App {
    pub fn new(daemon_client: DaemonClient) -> Self {
        #[cfg(feature = "adw")]
        let application: Application =
            adw::Application::new(Some(APP_ID), ApplicationFlags::default()).upcast();
        #[cfg(not(feature = "adw"))]
        let application = Application::new(Some(APP_ID), ApplicationFlags::default());

        let system_info_buf = daemon_client
            .get_system_info()
            .expect("Could not fetch system info");
        let system_info = system_info_buf.inner().expect("Invalid system info buffer");

        let header = Header::new(&system_info);
        let window = ApplicationWindow::builder()
            .title("LACT")
            .default_width(600)
            .default_height(860)
            .icon_name(APP_ID)
            .build();

        if system_info.version != GUI_VERSION || system_info.commit != Some(GIT_COMMIT) {
            let err = anyhow!("Version mismatch between GUI and daemon ({GUI_VERSION}-{GIT_COMMIT} vs {}-{})! Make sure you have restarted the service if you have updated LACT.", system_info.version, system_info.commit.unwrap_or_default());
            show_error(&window, err);
        }

        window.set_titlebar(Some(&header.container));

        let root_stack = RootStack::new(system_info, daemon_client.embedded);

        header.set_switcher_stack(&root_stack.container);

        let root_box = Box::new(Orientation::Vertical, 5);

        root_box.append(&root_stack.container);

        let apply_revealer = ApplyRevealer::new();

        root_box.append(&apply_revealer.container);

        window.set_child(Some(&root_box));

        let graphs_window = GraphsWindow::new();

        App {
            application,
            window,
            header,
            root_stack,
            apply_revealer,
            daemon_client,
            graphs_window,
        }
    }

    pub fn run(self, connection_err: Option<anyhow::Error>) -> anyhow::Result<()> {
        self.application.connect_activate(clone!(
            #[strong(rename_to = app)]
            self,
            move |_| {
                app.window.set_application(Some(&app.application));

                let current_gpu_id = Rc::new(RefCell::new(String::new()));

                app.header.connect_gpu_selection_changed(clone!(
                    #[strong]
                    app,
                    #[strong]
                    current_gpu_id,
                    move |gpu_id| {
                        debug!("GPU Selection changed");
                        app.set_info(&gpu_id);
                        *current_gpu_id.borrow_mut() = gpu_id;
                        debug!("Updated current GPU id");
                        app.graphs_window.clear();
                    }
                ));

                let devices_buf = app
                    .daemon_client
                    .list_devices()
                    .expect("Could not list devices");
                let devices = devices_buf.inner().expect("Could not access devices");
                app.header.set_devices(&devices);

                app.root_stack
                    .oc_page
                    .clocks_frame
                    .connect_clocks_reset(clone!(
                        #[strong]
                        app,
                        #[strong]
                        current_gpu_id,
                        move || {
                            debug!("Resetting clocks");

                            let gpu_id = current_gpu_id.borrow().clone();

                            match app
                                .daemon_client
                                .set_clocks_value(&gpu_id, SetClocksCommand::Reset)
                                .and_then(|_| {
                                    app.daemon_client
                                        .confirm_pending_config(ConfirmCommand::Confirm)
                                }) {
                                Ok(()) => {
                                    app.set_initial(&gpu_id);
                                }
                                Err(err) => {
                                    show_error(&app.window, err);
                                }
                            }
                        }
                    ));

                app.root_stack.thermals_page.connect_reset_pmfw(clone!(
                    #[strong]
                    app,
                    #[strong]
                    current_gpu_id,
                    move || {
                        debug!("Resetting PMFW settings");
                        let gpu_id = current_gpu_id.borrow().clone();

                        match app
                            .daemon_client
                            .reset_pmfw(&gpu_id)
                            .and_then(|buffer| buffer.inner())
                            .and_then(|_| {
                                app.daemon_client
                                    .confirm_pending_config(ConfirmCommand::Confirm)
                            }) {
                            Ok(()) => {
                                app.set_initial(&gpu_id);
                            }
                            Err(err) => {
                                show_error(&app.window, err);
                            }
                        }
                    }
                ));

                app.apply_revealer.connect_apply_button_clicked(clone!(
                    #[strong]
                    app,
                    #[strong]
                    current_gpu_id,
                    move || {
                        glib::idle_add_local_once(clone!(
                            #[strong]
                            app,
                            #[strong]
                            current_gpu_id,
                            move || {
                                if let Err(err) = app
                                    .apply_settings(current_gpu_id.clone())
                                    .context("Could not apply settings (GUI)")
                                {
                                    show_error(&app.window, err);

                                    glib::idle_add_local_once(clone!(
                                        #[strong]
                                        app,
                                        #[strong]
                                        current_gpu_id,
                                        move || {
                                            let gpu_id = current_gpu_id.borrow().clone();
                                            app.set_initial(&gpu_id)
                                        }
                                    ));
                                }
                            }
                        ));
                    }
                ));
                app.apply_revealer.connect_reset_button_clicked(clone!(
                    #[strong]
                    app,
                    #[strong]
                    current_gpu_id,
                    move || {
                        let gpu_id = current_gpu_id.borrow().clone();
                        app.set_initial(&gpu_id)
                    }
                ));

                if let Some(ref button) = app.root_stack.oc_page.enable_overclocking_button {
                    button.connect_clicked(clone!(
                        #[strong]
                        app,
                        move |_| {
                            app.enable_overclocking();
                        }
                    ));
                }

                let snapshot_action = ActionEntry::builder("generate-debug-snapshot")
                    .activate(clone!(
                        #[strong]
                        app,
                        move |_, _, _| {
                            app.generate_debug_snapshot();
                        }
                    ))
                    .build();

                let disable_overdive_action = ActionEntry::builder("disable-overdrive")
                    .activate(clone!(
                        #[strong]
                        app,
                        move |_, _, _| {
                            app.disable_overclocking();
                        }
                    ))
                    .build();

                let show_graphs_window_action = ActionEntry::builder("show-graphs-window")
                    .activate(clone!(
                        #[strong]
                        app,
                        move |_, _, _| {
                            app.graphs_window.show();
                        }
                    ))
                    .build();

                let dump_vbios_action = ActionEntry::builder("dump-vbios")
                    .activate(clone!(
                        #[strong]
                        app,
                        #[strong]
                        current_gpu_id,
                        move |_, _, _| {
                            let gpu_id = current_gpu_id.borrow();
                            app.dump_vbios(&gpu_id);
                        }
                    ))
                    .build();

                let reset_config_action = ActionEntry::builder("reset-config")
                    .activate(clone!(
                        #[strong]
                        app,
                        #[strong]
                        current_gpu_id,
                        move |_, _, _| {
                            let gpu_id = current_gpu_id.borrow().clone();
                            app.reset_config(gpu_id);
                        }
                    ))
                    .build();

                app.application.add_action_entries([
                    snapshot_action,
                    disable_overdive_action,
                    show_graphs_window_action,
                    dump_vbios_action,
                    reset_config_action,
                ]);

                app.start_stats_update_loop(current_gpu_id);

                app.window.show();

                if app.daemon_client.embedded {
                    let error_text = connection_err
                        .as_ref()
                        .map(|err| format!("Error info: {err:#}\n\n"))
                        .unwrap_or_default();

                    let text = format!(
                        "Could not connect to daemon, running in embedded mode. \n\
                        Please make sure the lactd service is running. \n\
                        Using embedded mode, you will not be able to change any settings. \n\n\
                        {error_text}\
                        To enable the daemon, run the following command:"
                    );

                    let text_label = Label::new(Some(&text));
                    let enable_label = Entry::builder()
                        .text("sudo systemctl enable --now lactd")
                        .editable(false)
                        .build();

                    let vbox = Box::builder()
                        .orientation(Orientation::Vertical)
                        .margin_top(10)
                        .margin_bottom(10)
                        .margin_start(10)
                        .margin_end(10)
                        .build();

                    let close_button = Button::builder().label("Close").build();

                    vbox.append(&text_label);
                    vbox.append(&enable_label);
                    vbox.append(&close_button);

                    let diag = MessageDialog::builder()
                        .title("Daemon info")
                        .message_type(MessageType::Warning)
                        .child(&vbox)
                        .transient_for(&app.window)
                        .build();

                    close_button.connect_clicked(clone!(
                        #[strong]
                        diag,
                        move |_| diag.hide()
                    ));

                    diag.run_async(|diag, _| {
                        diag.hide();
                    })
                }
            }
        ));

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
        self.root_stack.oc_page.set_info(&info);

        let vram_clock_ratio = info
            .drm_info
            .as_ref()
            .map(|info| info.vram_clock_ratio)
            .unwrap_or(1.0);
        self.graphs_window.set_vram_clock_ratio(vram_clock_ratio);

        self.set_initial(gpu_id);
        self.root_stack.thermals_page.set_info(&info);
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
        let show_revealer = clone!(
            #[strong(rename_to = apply_revealer)]
            self.apply_revealer,
            move || {
                debug!("settings changed, showing apply button");
                apply_revealer.show();
            }
        );

        self.root_stack
            .thermals_page
            .connect_settings_changed(show_revealer.clone());

        self.root_stack
            .oc_page
            .connect_settings_changed(show_revealer);

        self.apply_revealer.hide();
    }

    fn start_stats_update_loop(&self, current_gpu_id: Rc<RefCell<String>>) {
        // The loop that gets stats
        glib::spawn_future_local(clone!(
            #[strong(rename_to = daemon_client)]
            self.daemon_client,
            #[strong(rename_to = root_stack)]
            self.root_stack,
            #[strong(rename_to = graphs_window)]
            self.graphs_window,
            async move {
                loop {
                    {
                        let gpu_id = current_gpu_id.borrow();
                        trace!("fetching new stats using id {gpu_id}");
                        match daemon_client
                            .get_device_stats(&gpu_id)
                            .and_then(|stats| stats.inner())
                        {
                            Ok(stats) => {
                                trace!("new stats received, updating {stats:?}");
                                root_stack.info_page.set_stats(&stats);
                                root_stack.thermals_page.set_stats(&stats, false);
                                root_stack.oc_page.set_stats(&stats, false);
                                graphs_window.set_stats(&stats);
                            }
                            Err(err) => {
                                error!("Could not fetch stats: {err}");
                            }
                        }
                    }
                    timeout_future(Duration::from_millis(STATS_POLL_INTERVAL)).await;
                }
            }
        ));
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
            .set_power_profile_mode(&gpu_id, None, vec![])
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
            let custom_heuristics = self
                .root_stack
                .oc_page
                .performance_frame
                .get_power_profile_mode_custom_heuristics();

            self.daemon_client
                .set_power_profile_mode(&gpu_id, mode_index, custom_heuristics)
                .context("Could not set active power profile mode")?;
            self.daemon_client
                .confirm_pending_config(ConfirmCommand::Confirm)
                .context("Could not commit config")?;
        }

        if let Some(thermals_settings) = self.root_stack.thermals_page.get_thermals_settings() {
            debug!("applying thermal settings: {thermals_settings:?}");
            let opts = FanOptions {
                id: &gpu_id,
                enabled: thermals_settings.manual_fan_control,
                mode: thermals_settings.mode,
                static_speed: thermals_settings.static_speed,
                curve: thermals_settings.curve,
                pmfw: thermals_settings.pmfw,
                spindown_delay_ms: thermals_settings.spindown_delay_ms,
                change_threshold: thermals_settings.change_threshold,
            };

            self.daemon_client
                .set_fan_control(opts)
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
                .context("Could not commit clocks settings")?;
            self.ask_settings_confirmation(gpu_id.clone(), delay);
        }

        self.set_initial(&gpu_id);

        Ok(())
    }

    fn generate_debug_snapshot(&self) {
        match self
            .daemon_client
            .generate_debug_snapshot()
            .and_then(|response| response.inner())
        {
            Ok(path) => {
                let path_label = Label::builder()
                    .use_markup(true)
                    .label(format!("<b>{path}</b>"))
                    .selectable(true)
                    .build();

                let vbox = Box::builder()
                    .orientation(Orientation::Vertical)
                    .margin_top(10)
                    .margin_bottom(10)
                    .margin_start(10)
                    .margin_end(10)
                    .build();

                vbox.append(&Label::new(Some("Debug snapshot saved at:")));
                vbox.append(&path_label);

                let diag = MessageDialog::builder()
                    .title("Snapshot generated")
                    .message_type(MessageType::Info)
                    .use_markup(true)
                    .text(format!("Debug snapshot saved at <b>{path}</b>"))
                    .buttons(ButtonsType::Ok)
                    .transient_for(&self.window)
                    .build();

                let message_box = diag.message_area().downcast::<gtk::Box>().unwrap();
                for child in message_box.observe_children().into_iter().flatten() {
                    if let Ok(label) = child.downcast::<Label>() {
                        label.set_selectable(true);
                    }
                }

                diag.run_async(|diag, _| {
                    diag.hide();
                })
            }
            Err(err) => show_error(&self.window, err.context("Could not generate snapshot")),
        }
    }

    fn enable_overclocking(&self) {
        let text = format!("This will enable the overdrive feature of the amdgpu driver by creating a file at <b>{MODULE_CONF_PATH}</b> and updating the initramfs. Are you sure you want to do this?");
        let dialog = MessageDialog::builder()
            .title("Enable Overclocking")
            .use_markup(true)
            .text(text)
            .message_type(MessageType::Question)
            .buttons(ButtonsType::OkCancel)
            .transient_for(&self.window)
            .build();

        dialog.run_async(clone!(
            #[strong(rename_to = app)]
            self,
            move |diag, response| {
                if response == ResponseType::Ok {
                    let handle = gio::spawn_blocking(|| {
                        let (daemon_client, _) =
                            create_connection().expect("Could not create new daemon connection");
                        daemon_client
                            .enable_overdrive()
                            .and_then(|buffer| buffer.inner())
                    });

                    let dialog =
                        app.spinner_dialog("Regenerating initramfs (this may take a while)");
                    dialog.show();

                    glib::spawn_future_local(async move {
                        let result = handle.await.unwrap();
                        dialog.hide();

                        match result {
                            Ok(msg) => oc_toggled_dialog(true, &msg),
                            Err(err) => {
                                show_error(&app.window, err);
                            }
                        }
                    });
                }
                diag.hide();
            }
        ));
    }

    fn disable_overclocking(&self) {
        let dialog = MessageDialog::builder()
            .title("Disable Overclocking")
            .use_markup(true)
            .text("The overclocking functionality in the driver will now be turned off.")
            .message_type(MessageType::Info)
            .buttons(ButtonsType::Ok)
            .transient_for(&self.window)
            .build();

        dialog.run_async(clone!(
            #[strong(rename_to = app)]
            self,
            move |diag, _| {
                diag.hide();

                let handle = gio::spawn_blocking(|| {
                    let (daemon_client, _) =
                        create_connection().expect("Could not create new daemon connection");
                    daemon_client
                        .disable_overdrive()
                        .and_then(|buffer| buffer.inner())
                });

                let dialog = app.spinner_dialog("Regenerating initramfs (this may take a while)");
                dialog.show();

                glib::spawn_future_local(async move {
                    let result = handle.await.unwrap();
                    dialog.hide();

                    match result {
                        Ok(msg) => oc_toggled_dialog(false, &msg),
                        Err(err) => {
                            show_error(&app.window, err);
                        }
                    }
                });
            }
        ));
    }

    fn dump_vbios(&self, gpu_id: &str) {
        match self
            .daemon_client
            .dump_vbios(gpu_id)
            .and_then(|response| response.inner())
        {
            Ok(vbios_data) => {
                let file_chooser = FileChooserDialog::new(
                    Some("Save VBIOS file"),
                    Some(&self.window),
                    FileChooserAction::Save,
                    &[
                        ("Save", ResponseType::Accept),
                        ("Cancel", ResponseType::Cancel),
                    ],
                );

                let file_name_suffix = gpu_id
                    .split_once('-')
                    .map(|(id, _)| id.replace(':', "_"))
                    .unwrap_or_default();
                file_chooser.set_current_name(&format!("{file_name_suffix}_vbios_dump.rom"));
                file_chooser.run_async(clone!(
                    #[strong(rename_to = window)]
                    self.window,
                    move |diag, _| {
                        diag.close();

                        if let Some(file) = diag.file() {
                            match file.path() {
                                Some(path) => {
                                    if let Err(err) = std::fs::write(path, vbios_data)
                                        .context("Could not save vbios file")
                                    {
                                        show_error(&window, err);
                                    }
                                }
                                None => show_error(
                                    &window,
                                    anyhow!("Selected file has an invalid path"),
                                ),
                            }
                        }
                    }
                ));
            }
            Err(err) => show_error(&self.window, err),
        }
    }

    fn reset_config(&self, gpu_id: String) {
        let dialog = MessageDialog::builder()
            .title("Reset configuration")
            .text("Are you sure you want to reset all GPU configuration?")
            .message_type(MessageType::Question)
            .buttons(ButtonsType::YesNo)
            .transient_for(&self.window)
            .build();

        dialog.run_async(clone!(
            #[strong(rename_to = app)]
            self,
            move |diag, response| {
                diag.hide();

                if response == ResponseType::Yes {
                    if let Err(err) = app
                        .daemon_client
                        .reset_config()
                        .and_then(|response| response.inner())
                    {
                        show_error(&app.window, err);
                    }

                    app.set_initial(&gpu_id);
                }
            }
        ));
    }

    fn ask_settings_confirmation(&self, gpu_id: String, mut delay: u64) {
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
            clone!(
                #[strong]
                dialog,
                #[strong(rename_to = app)]
                self,
                #[strong]
                gpu_id,
                #[strong]
                confirmed,
                move || {
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
                    } else {
                        ControlFlow::Continue
                    }
                }
            ),
        );

        dialog.run_async(clone!(
            #[strong(rename_to = app)]
            self,
            move |diag, response| {
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
            }
        ));
    }

    fn spinner_dialog(&self, title: &str) -> MessageDialog {
        let spinner = gtk::Spinner::new();
        spinner.start();
        spinner.set_margin_top(10);
        spinner.set_margin_bottom(10);

        let dialog = MessageDialog::builder()
            .title(title)
            .child(&spinner)
            .message_type(MessageType::Info)
            .transient_for(&self.window)
            .build();

        if let Some(bar) = dialog.titlebar() {
            bar.set_margin_start(15);
            bar.set_margin_end(15);
        }

        dialog
    }
}

fn show_error(parent: &ApplicationWindow, err: anyhow::Error) {
    let text = format!("{err:?}")
        .lines()
        .map(str::trim)
        .collect::<Vec<&str>>()
        .join("\n");
    warn!("{text}");
    let diag = MessageDialog::builder()
        .title("Error")
        .message_type(MessageType::Error)
        .text(text)
        .buttons(ButtonsType::Close)
        .transient_for(parent)
        .build();
    diag.run_async(|diag, _| {
        diag.hide();
    })
}

fn oc_toggled_dialog(enabled: bool, msg: &str) {
    let enabled_text = if enabled { "enabled" } else { "disabled" };

    let child = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(5)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(10)
        .margin_end(10)
        .build();
    child.append(&Label::new(Some(&format!("Overclocking successfully {enabled_text}. A system reboot is required to apply the changes.\nSystem message:"))));

    let msg_label = Label::builder()
        .label(msg)
        .valign(Align::Start)
        .halign(Align::Start)
        .build();
    let msg_scrollable = ScrolledWindow::builder().child(&msg_label).build();
    child.append(&msg_scrollable);

    let ok_button = Button::builder().label("OK").build();
    child.append(&ok_button);

    let success_dialog = MessageDialog::builder()
        .title("Success")
        .child(&child)
        .message_type(MessageType::Info)
        .build();

    ok_button.connect_clicked(clone!(
        #[strong]
        success_dialog,
        move |_| success_dialog.hide(),
    ));

    success_dialog.run_async(move |diag, _| {
        diag.hide();
    });
}

fn confirmation_text(seconds_left: u64) -> String {
    format!("Do you want to keep the new settings? (Reverting in {seconds_left} seconds)")
}
