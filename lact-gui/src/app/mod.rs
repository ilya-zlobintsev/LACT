mod apply_revealer;
mod confirmation_dialog;
mod graphs_window;
mod header;
mod info_row;
mod msg;
mod page_section;
mod root_stack;

#[cfg(feature = "bench")]
pub use graphs_window::plot::{Plot, PlotData};

use crate::{APP_ID, GUI_VERSION};
use anyhow::{anyhow, Context};
use apply_revealer::{ApplyRevealer, ApplyRevealerMsg};
use confirmation_dialog::ConfirmationDialog;
use graphs_window::GraphsWindow;
use gtk::{
    glib::{self, clone, ControlFlow},
    prelude::{
        BoxExt, ButtonExt, Cast, DialogExtManual, FileChooserExt, FileExt, GtkWindowExt,
        OrientableExt, WidgetExt,
    },
    ApplicationWindow, ButtonsType, FileChooserAction, FileChooserDialog, MessageDialog,
    MessageType, ResponseType,
};
use header::Header;
use lact_client::DaemonClient;
use lact_schema::{
    request::{ConfirmCommand, SetClocksCommand},
    FanOptions, GIT_COMMIT,
};
use msg::AppMsg;
use relm4::{
    actions::{RelmAction, RelmActionGroup},
    tokio, Component, ComponentController, ComponentParts, ComponentSender,
};
use root_stack::RootStack;
use std::{cell::RefCell, rc::Rc, sync::atomic::AtomicBool, time::Duration};
use tracing::{debug, error, trace, warn};

const STATS_POLL_INTERVAL_MS: u64 = 250;

pub struct AppModel {
    daemon_client: DaemonClient,
    graphs_window: GraphsWindow,
    root_stack: RootStack,
    header: relm4::Controller<Header>,
    apply_revealer: relm4::Controller<ApplyRevealer>,
    stats_task_handle: Option<glib::JoinHandle<()>>,
}

#[relm4::component(pub)]
impl Component for AppModel {
    type Init = (DaemonClient, Option<anyhow::Error>);

    type Input = AppMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[name = "root_window"]
        gtk::ApplicationWindow {
            set_title: Some("LACT"),
            set_default_width: 600,
            set_default_height: 860,
            set_icon_name: Some(APP_ID),
            set_titlebar: Some(model.header.widget()),

            #[name = "root_box"]
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,

                model.root_stack.container.clone(),
                model.apply_revealer.widget(),
            }
        }
    }

    fn init(
        (daemon_client, conn_err): Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        register_actions(&sender);

        let system_info_buf = daemon_client
            .get_system_info()
            .expect("Could not fetch system info");
        let system_info = system_info_buf.inner().expect("Invalid system info buffer");

        let devices_buf = daemon_client
            .list_devices()
            .expect("Could not list devices");
        let devices = devices_buf.inner().expect("Could not access devices");

        if system_info.version != GUI_VERSION || system_info.commit != Some(GIT_COMMIT) {
            let err = anyhow!("Version mismatch between GUI and daemon ({GUI_VERSION}-{GIT_COMMIT} vs {}-{})! Make sure you have restarted the service if you have updated LACT.", system_info.version, system_info.commit.unwrap_or_default());
            sender.input(AppMsg::Error(err.into()));
        }

        let root_stack = RootStack::new(system_info, daemon_client.embedded);

        let header = Header::builder()
            .launch((devices, root_stack.container.clone()))
            .forward(sender.input_sender(), |msg| msg);

        let apply_revealer = ApplyRevealer::builder()
            .launch(())
            .forward(sender.input_sender(), |msg| msg);

        let graphs_window = GraphsWindow::new();

        let model = AppModel {
            daemon_client,
            graphs_window,
            root_stack,
            apply_revealer,
            header,
            stats_task_handle: None,
        };

        let widgets = view_output!();

        let embedded = model.daemon_client.embedded;
        let conn_err = RefCell::new(conn_err);
        root.connect_visible_notify(move |root| {
            if embedded {
                if let Some(err) = conn_err.borrow_mut().take() {
                    show_embedded_info(root, err);
                }
            }
        });

        sender.input(AppMsg::ReloadData { full: true });

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        trace!("update {msg:#?}");
        if let Err(err) = self.handle_msg(msg, sender.clone(), root) {
            show_error(root, &err);
        }
        self.update_view(widgets, sender);
    }
}

impl AppModel {
    fn handle_msg(
        &mut self,
        msg: AppMsg,
        sender: ComponentSender<Self>,
        root: &gtk::ApplicationWindow,
    ) -> Result<(), Rc<anyhow::Error>> {
        match msg {
            AppMsg::Error(err) => Err(err),
            AppMsg::ReloadData { full } => {
                let gpu_id = self.current_gpu_id()?;
                if full {
                    self.update_gpu_data_full(gpu_id, sender)?;
                } else {
                    self.update_gpu_data(gpu_id, sender)?;
                }
                Ok(())
            }
            AppMsg::Stats(stats) => {
                self.root_stack.info_page.set_stats(&stats);
                self.root_stack.thermals_page.set_stats(&stats, false);
                self.root_stack.oc_page.set_stats(&stats, false);
                self.graphs_window.set_stats(&stats);
                Ok(())
            }
            AppMsg::ApplyChanges => self
                .apply_settings(self.current_gpu_id()?, root, &sender)
                .map_err(|err| {
                    sender.input(AppMsg::ReloadData { full: false });
                    err.into()
                }),
            AppMsg::RevertChanges => {
                sender.input(AppMsg::ReloadData { full: false });
                Ok(())
            }
            AppMsg::ShowGraphsWindow => {
                self.graphs_window.show();
                Ok(())
            }
            AppMsg::DumpVBios => {
                self.dump_vbios(&self.current_gpu_id()?, root);
                Ok(())
            }
            AppMsg::DebugSnapshot => {
                self.generate_debug_snapshot(root);
                Ok(())
            }
            AppMsg::DisableOverdrive => todo!(),
            AppMsg::ResetConfig => {
                self.daemon_client.reset_config()?;
                sender.input(AppMsg::ReloadData { full: true });
                Ok(())
            }
            AppMsg::AskConfirmation(options, confirmed_msg) => {
                let sender = sender.clone();

                let mut controller = ConfirmationDialog::builder()
                    .launch((options, root.clone()))
                    .connect_receiver(move |_, response| {
                        if let gtk::ResponseType::Ok | gtk::ResponseType::Yes = response {
                            sender.input(*confirmed_msg.clone());
                        }
                    });
                controller.detach_runtime();

                Ok(())
            }
        }
    }

    fn current_gpu_id(&self) -> anyhow::Result<String> {
        self.header
            .model()
            .selected_gpu_id()
            .context("No GPU selected")
    }

    fn update_gpu_data_full(
        &mut self,
        gpu_id: String,
        sender: ComponentSender<AppModel>,
    ) -> anyhow::Result<()> {
        let daemon_client = self.daemon_client.clone();
        let info_buf = daemon_client
            .get_device_info(&gpu_id)
            .context("Could not fetch info")?;
        let info = info_buf.inner()?;

        self.root_stack.info_page.set_info(&info);
        self.root_stack.oc_page.set_info(&info);

        let vram_clock_ratio = info
            .drm_info
            .as_ref()
            .map(|info| info.vram_clock_ratio)
            .unwrap_or(1.0);
        self.graphs_window.set_vram_clock_ratio(vram_clock_ratio);

        self.update_gpu_data(gpu_id, sender)?;

        self.root_stack.thermals_page.set_info(&info);

        Ok(())
    }

    fn update_gpu_data(
        &mut self,
        gpu_id: String,
        sender: ComponentSender<AppModel>,
    ) -> anyhow::Result<()> {
        if let Some(stats_task) = self.stats_task_handle.take() {
            stats_task.abort();
        }

        debug!("updating info for gpu {gpu_id}");

        let stats = self
            .daemon_client
            .get_device_stats(&gpu_id)
            .context("Could not fetch stats")?
            .inner()?;

        self.root_stack.oc_page.set_stats(&stats, true);
        self.root_stack.thermals_page.set_stats(&stats, true);
        self.root_stack.info_page.set_stats(&stats);

        let maybe_clocks_table = match self.daemon_client.get_device_clocks_info(&gpu_id) {
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

        let maybe_modes_table = match self.daemon_client.get_device_power_profile_modes(&gpu_id) {
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
            .get_power_states(&gpu_id)
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
            #[strong(rename_to = apply_sender)]
            self.apply_revealer.sender(),
            move || {
                apply_sender.send(ApplyRevealerMsg::Show).unwrap();
            }
        );

        self.root_stack
            .thermals_page
            .connect_settings_changed(show_revealer.clone());

        self.root_stack
            .oc_page
            .connect_settings_changed(show_revealer);

        self.apply_revealer
            .sender()
            .send(ApplyRevealerMsg::Hide)
            .unwrap();

        self.graphs_window.clear();

        self.stats_task_handle = Some(start_stats_update_loop(
            gpu_id.to_owned(),
            self.daemon_client.clone(),
            sender,
        ));

        Ok(())
    }

    fn apply_settings(
        &self,
        gpu_id: String,
        root: &gtk::ApplicationWindow,
        sender: &ComponentSender<Self>,
    ) -> anyhow::Result<()> {
        // TODO: Ask confirmation for everything, not just clocks

        debug!("applying settings on gpu {gpu_id}");

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
            self.ask_settings_confirmation(delay, root, sender);
        }

        sender.input(AppMsg::ReloadData { full: false });

        Ok(())
    }

    fn ask_settings_confirmation(
        &self,
        mut delay: u64,
        window: &gtk::ApplicationWindow,
        sender: &ComponentSender<AppModel>,
    ) {
        let text = confirmation_text(delay);
        let dialog = MessageDialog::builder()
            .title("Confirm settings")
            .text(text)
            .message_type(MessageType::Question)
            .buttons(ButtonsType::YesNo)
            .transient_for(window)
            .build();
        let confirmed = Rc::new(AtomicBool::new(false));

        glib::source::timeout_add_local(
            Duration::from_secs(1),
            clone!(
                #[strong]
                dialog,
                #[strong]
                sender,
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
                        sender.input(AppMsg::ReloadData { full: false });

                        ControlFlow::Break
                    } else {
                        ControlFlow::Continue
                    }
                }
            ),
        );

        dialog.run_async(clone!(
            #[strong]
            sender,
            #[strong(rename_to = daemon_client)]
            self.daemon_client,
            #[strong]
            window,
            move |diag, response| {
                confirmed.store(true, std::sync::atomic::Ordering::SeqCst);

                let command = match response {
                    ResponseType::Yes => ConfirmCommand::Confirm,
                    _ => ConfirmCommand::Revert,
                };

                diag.close();

                if let Err(err) = daemon_client.confirm_pending_config(command) {
                    show_error(&window, &err);
                }
                sender.input(AppMsg::ReloadData { full: false });
            }
        ));
    }

    fn dump_vbios(&self, gpu_id: &str, root: &gtk::ApplicationWindow) {
        match self
            .daemon_client
            .dump_vbios(gpu_id)
            .and_then(|response| response.inner())
        {
            Ok(vbios_data) => {
                let file_chooser = FileChooserDialog::new(
                    Some("Save VBIOS file"),
                    Some(root),
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
                    #[strong]
                    root,
                    move |diag, response| {
                        diag.close();

                        if response == gtk::ResponseType::Accept {
                            if let Some(file) = diag.file() {
                                match file.path() {
                                    Some(path) => {
                                        if let Err(err) = std::fs::write(path, vbios_data)
                                            .context("Could not save vbios file")
                                        {
                                            show_error(&root, &err);
                                        }
                                    }
                                    None => show_error(
                                        &root,
                                        &anyhow!("Selected file has an invalid path"),
                                    ),
                                }
                            }
                        }
                    }
                ));
            }
            Err(err) => show_error(root, &err),
        }
    }

    fn generate_debug_snapshot(&self, root: &gtk::ApplicationWindow) {
        match self
            .daemon_client
            .generate_debug_snapshot()
            .and_then(|response| response.inner())
        {
            Ok(path) => {
                let path_label = gtk::Label::builder()
                    .use_markup(true)
                    .label(format!("<b>{path}</b>"))
                    .selectable(true)
                    .build();

                let vbox = gtk::Box::builder()
                    .orientation(gtk::Orientation::Vertical)
                    .margin_top(10)
                    .margin_bottom(10)
                    .margin_start(10)
                    .margin_end(10)
                    .build();

                vbox.append(&gtk::Label::new(Some("Debug snapshot saved at:")));
                vbox.append(&path_label);

                let diag = MessageDialog::builder()
                    .title("Snapshot generated")
                    .message_type(MessageType::Info)
                    .use_markup(true)
                    .text(format!("Debug snapshot saved at <b>{path}</b>"))
                    .buttons(ButtonsType::Ok)
                    .transient_for(root)
                    .build();

                let message_box = diag.message_area().downcast::<gtk::Box>().unwrap();
                for child in message_box.observe_children().into_iter().flatten() {
                    if let Ok(label) = child.downcast::<gtk::Label>() {
                        label.set_selectable(true);
                    }
                }

                diag.run_async(|diag, _| {
                    diag.hide();
                })
            }
            Err(err) => show_error(root, &err.context("Could not generate snapshot")),
        }
    }
}

fn show_error(parent: &ApplicationWindow, err: &anyhow::Error) {
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
        diag.close();
    })
}

fn show_embedded_info(parent: &ApplicationWindow, err: anyhow::Error) {
    let error_text = format!("Error info: {err:#}\n\n");

    let text = format!(
        "Could not connect to daemon, running in embedded mode. \n\
                        Please make sure the lactd service is running. \n\
                        Using embedded mode, you will not be able to change any settings. \n\n\
                        {error_text}\
                        To enable the daemon, run the following command:"
    );

    let text_label = gtk::Label::new(Some(&text));
    let enable_label = gtk::Entry::builder()
        .text("sudo systemctl enable --now lactd")
        .editable(false)
        .build();

    let vbox = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(10)
        .margin_end(10)
        .build();

    let close_button = gtk::Button::builder().label("Close").build();

    vbox.append(&text_label);
    vbox.append(&enable_label);
    vbox.append(&close_button);

    let diag = gtk::MessageDialog::new(
        Some(parent),
        gtk::DialogFlags::MODAL,
        gtk::MessageType::Question,
        gtk::ButtonsType::Ok,
        "",
    );
    diag.set_title(Some("Daemon info"));
    diag.set_child(Some(&vbox));

    close_button.connect_clicked(clone!(
        #[strong]
        diag,
        move |_| diag.hide()
    ));

    diag.run_async(|diag, _| {
        diag.hide();
    })
}

fn start_stats_update_loop(
    gpu_id: String,
    daemon_client: DaemonClient,
    sender: ComponentSender<AppModel>,
) -> glib::JoinHandle<()> {
    debug!("spawning new stats update task with {STATS_POLL_INTERVAL_MS}ms interval");
    let duration = Duration::from_millis(STATS_POLL_INTERVAL_MS);
    relm4::spawn_local(async move {
        loop {
            tokio::time::sleep(duration).await;

            match daemon_client
                .get_device_stats(&gpu_id)
                .and_then(|buffer| buffer.inner())
            {
                Ok(stats) => {
                    sender.input(AppMsg::Stats(stats));
                }
                Err(err) => {
                    error!("could not fetch stats: {err:#}");
                }
            }
        }
    })
}

fn confirmation_text(seconds_left: u64) -> String {
    format!("Do you want to keep the new settings? (Reverting in {seconds_left} seconds)")
}

fn register_actions(sender: &ComponentSender<AppModel>) {
    let mut group = RelmActionGroup::<AppActionGroup>::new();

    macro_rules! actions {
        ($(($action:ty, $msg:expr),)*) => {
            $(
                group.add_action(RelmAction::<$action>::new_stateless(clone!(
                    #[strong]
                    sender,
                    move |_| sender.input($msg)
                )));
            )*
        }
    }

    actions! {
        (ShowGraphsWindow, AppMsg::ShowGraphsWindow),
        (DumpVBios, AppMsg::DumpVBios),
        (DebugSnapshot, AppMsg::DebugSnapshot),
        (
            DisableOverdrive,
            AppMsg::ask_confirmation(
                AppMsg::DisableOverdrive,
                "Disable overclocking",
                "This will disable overclocking support on next reboot.",
                gtk::ButtonsType::OkCancel,
            )
        ),
        (
            ResetConfig,
            AppMsg::ask_confirmation(
                AppMsg::ResetConfig,
                "Reset configuration",
                "Are you sure you want to reset all GPU configuration?",
                gtk::ButtonsType::YesNo,
            )
        ),
    };

    group.register_for_main_application();
}

relm4::new_action_group!(AppActionGroup, "app");
relm4::new_stateless_action!(ShowGraphsWindow, AppActionGroup, "show-graphs-window");
relm4::new_stateless_action!(DumpVBios, AppActionGroup, "dump-vbios");
relm4::new_stateless_action!(DebugSnapshot, AppActionGroup, "generate-debug-snapshot");
relm4::new_stateless_action!(DisableOverdrive, AppActionGroup, "disable-overdrive");
relm4::new_stateless_action!(ResetConfig, AppActionGroup, "reset-config");
