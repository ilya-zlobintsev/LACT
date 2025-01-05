mod apply_revealer;
mod confirmation_dialog;
mod graphs_window;
mod header;
mod info_row;
mod msg;
mod page_section;
mod pages;

use crate::{APP_ID, GUI_VERSION};
use anyhow::{anyhow, Context};
use apply_revealer::{ApplyRevealer, ApplyRevealerMsg};
use confirmation_dialog::ConfirmationDialog;
use graphs_window::{GraphsWindow, GraphsWindowMsg};
use gtk::{
    glib::{self, clone, ControlFlow},
    prelude::{
        BoxExt, ButtonExt, Cast, DialogExtManual, FileChooserExt, FileExt, GtkWindowExt,
        OrientableExt, WidgetExt,
    },
    ApplicationWindow, ButtonsType, FileChooserAction, FileChooserDialog, MessageDialog,
    MessageType, ResponseType,
};
use header::{
    profile_rule_window::ProfileRuleWindowMsg, Header, HeaderMsg, PROFILE_RULE_WINDOW_BROKER,
};
use lact_client::{ConnectionStatusMsg, DaemonClient};
use lact_daemon::MODULE_CONF_PATH;
use lact_schema::{
    args::GuiArgs,
    request::{ConfirmCommand, SetClocksCommand},
    FanOptions, GIT_COMMIT,
};
use msg::AppMsg;
use pages::{
    info_page::InformationPage, oc_page::OcPage, software_page::SoftwarePage,
    thermals_page::ThermalsPage, PageUpdate,
};
use relm4::{
    actions::{RelmAction, RelmActionGroup},
    prelude::{AsyncComponent, AsyncComponentParts},
    tokio, AsyncComponentSender, Component, ComponentController, MessageBroker,
};
use std::{
    os::unix::net::UnixStream,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc,
    },
    time::Duration,
};
use tracing::{debug, error, info, trace, warn};

pub(crate) static APP_BROKER: MessageBroker<AppMsg> = MessageBroker::new();
static ERROR_WINDOW_COUNT: AtomicU32 = AtomicU32::new(0);

const STATS_POLL_INTERVAL_MS: u64 = 250;

pub struct AppModel {
    daemon_client: DaemonClient,
    graphs_window: relm4::Controller<GraphsWindow>,

    info_page: relm4::Controller<InformationPage>,
    oc_page: OcPage,
    thermals_page: ThermalsPage,
    software_page: relm4::Controller<SoftwarePage>,

    header: relm4::Controller<Header>,
    apply_revealer: relm4::Controller<ApplyRevealer>,
    stats_task_handle: Option<glib::JoinHandle<()>>,
}

#[relm4::component(pub, async)]
impl AsyncComponent for AppModel {
    type Init = GuiArgs;

    type Input = AppMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        gtk::ApplicationWindow {
            set_title: Some("LACT"),
            set_default_width: 600,
            set_default_height: 900,
            set_icon_name: Some(APP_ID),
            set_titlebar: Some(model.header.widget()),

            #[name = "root_box"]
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,

                #[name = "root_stack"]
                gtk::Stack {
                    set_vexpand: true,
                    set_margin_top: 15,
                    set_margin_start: 30,
                    set_margin_end: 30,

                    add_titled[Some("info_page"), "Information"] = model.info_page.widget(),
                    add_titled[Some("oc_page"), "OC"] = &model.oc_page.container.clone(),
                    add_titled[Some("thermals_page"), "Thermals"] = &model.thermals_page.container.clone(),
                    add_titled[Some("software_page"), "Software"] = model.software_page.widget(),
                },

                model.apply_revealer.widget(),
            }
        },

        #[name = "reconnecting_dialog"]
        gtk::MessageDialog::new(
            Some(&root),
            gtk::DialogFlags::MODAL,
            gtk::MessageType::Error,
            gtk::ButtonsType::None,
            "Daemon connection lost, reconnecting...",
        ) -> gtk::MessageDialog {
            set_title: Some("Connection Lost"),
        }
    }

    async fn init(
        args: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let (daemon_client, conn_err) = match args.tcp_address {
            Some(remote_addr) => {
                info!("establishing connection to {remote_addr}");
                match DaemonClient::connect_tcp(&remote_addr).await {
                    Ok(conn) => (conn, None),
                    Err(err) => {
                        error!("TCP connection error: {err:#}");
                        let (conn, _) = create_connection()
                            .await
                            .expect("Could not create fallback connection");
                        (conn, Some(err))
                    }
                }
            }
            None => create_connection()
                .await
                .expect("Could not establish any daemon connection"),
        };

        let mut conn_status_rx = daemon_client.status_receiver();
        relm4::spawn_local(clone!(
            #[strong]
            sender,
            async move {
                loop {
                    if let Ok(msg) = conn_status_rx.recv().await {
                        sender.input(AppMsg::ConnectionStatus(msg));
                    }
                }
            }
        ));

        register_actions(&sender);

        let system_info = daemon_client
            .get_system_info()
            .await
            .expect("Could not fetch system info");

        let devices = daemon_client
            .list_devices()
            .await
            .expect("Could not list devices");

        if system_info.version != GUI_VERSION || system_info.commit.as_deref() != Some(GIT_COMMIT) {
            let err = anyhow!("Version mismatch between GUI and daemon ({GUI_VERSION}-{GIT_COMMIT} vs {}-{})! If you have updated LACT, you need to restart the service with `sudo systemctl restart lactd`.", system_info.version, system_info.commit.as_deref().unwrap_or_default());
            sender.input(AppMsg::Error(err.into()));
        }

        let info_page = InformationPage::builder().launch(()).detach();

        let oc_page = OcPage::new(&system_info);
        let thermals_page = ThermalsPage::new(&system_info);

        let software_page = SoftwarePage::builder()
            .launch((system_info, daemon_client.embedded))
            .detach();

        let header = Header::builder()
            .launch(devices)
            .forward(sender.input_sender(), |msg| msg);

        let apply_revealer = ApplyRevealer::builder()
            .launch(())
            .forward(sender.input_sender(), |msg| msg);

        oc_page.clocks_frame.connect_clocks_reset(clone!(
            #[strong]
            sender,
            move || {
                sender.input(AppMsg::ResetClocks);
            }
        ));
        thermals_page.connect_reset_pmfw(clone!(
            #[strong]
            sender,
            move || {
                sender.input(AppMsg::ResetPmfw);
            }
        ));

        if let Some(ref button) = oc_page.enable_overclocking_button {
            button.connect_clicked(clone!(
                #[strong]
                sender,
                move |_| {
                    sender.input(AppMsg::ask_confirmation(
                        AppMsg::EnableOverdrive,
                        "Enable Overclocking",
                        format!("This will enable the overdrive feature of the amdgpu driver by creating a file at <b>{MODULE_CONF_PATH}</b> and updating the initramfs. Are you sure you want to do this?"),
                        gtk::ButtonsType::OkCancel,
                    ));
                }
            ));
        }

        let graphs_window = GraphsWindow::builder().launch(()).detach();

        let model = AppModel {
            daemon_client,
            graphs_window,
            info_page,
            oc_page,
            thermals_page,
            software_page,
            apply_revealer,
            header,
            stats_task_handle: None,
        };

        let widgets = view_output!();

        if let Some(err) = conn_err {
            show_embedded_info(&root, err);
        }

        model
            .header
            .widgets()
            .stack_switcher
            .set_stack(Some(&widgets.root_stack));

        sender.input(AppMsg::ReloadProfiles {
            include_state: false,
        });

        AsyncComponentParts { model, widgets }
    }

    async fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: AsyncComponentSender<Self>,
        root: &Self::Root,
    ) {
        trace!("processing state update");
        if let Err(err) = self.handle_msg(msg, sender.clone(), root, widgets).await {
            show_error(root, &err);
        }
        self.update_view(widgets, sender);
    }
}

impl AppModel {
    async fn handle_msg(
        &mut self,
        msg: AppMsg,
        sender: AsyncComponentSender<Self>,
        root: &gtk::ApplicationWindow,
        widgets: &AppModelWidgets,
    ) -> Result<(), Arc<anyhow::Error>> {
        match msg {
            AppMsg::Error(err) => return Err(err),
            AppMsg::ReloadProfiles { include_state } => {
                self.reload_profiles(include_state).await?;
                sender.input(AppMsg::ReloadData { full: false });
            }
            AppMsg::ReloadData { full } => {
                let gpu_id = self.current_gpu_id()?;
                if full {
                    self.update_gpu_data_full(gpu_id, sender).await?;
                } else {
                    self.update_gpu_data(gpu_id, sender).await?;
                }
            }
            AppMsg::SelectProfile {
                profile,
                auto_switch,
            } => {
                self.daemon_client.set_profile(profile, auto_switch).await?;
                sender.input(AppMsg::ReloadProfiles {
                    include_state: false,
                });
            }
            AppMsg::CreateProfile(name, base) => {
                self.daemon_client
                    .create_profile(name.clone(), base)
                    .await?;

                let auto_switch = self.header.model().auto_switch_profiles();
                self.daemon_client
                    .set_profile(Some(name), auto_switch)
                    .await?;

                sender.input(AppMsg::ReloadProfiles {
                    include_state: false,
                });
            }
            AppMsg::DeleteProfile(profile) => {
                self.daemon_client.delete_profile(profile).await?;
                sender.input(AppMsg::ReloadProfiles {
                    include_state: false,
                });
            }
            AppMsg::MoveProfile(name, new_position) => {
                self.daemon_client.move_profile(name, new_position).await?;
                sender.input(AppMsg::ReloadProfiles {
                    include_state: false,
                });
            }
            AppMsg::Stats(stats) => {
                self.info_page.emit(PageUpdate::Stats(stats.clone()));

                self.thermals_page.set_stats(&stats, false);
                self.oc_page.set_stats(&stats, false);

                self.graphs_window.emit(GraphsWindowMsg::Stats(stats));
            }
            AppMsg::ApplyChanges => {
                self.apply_settings(self.current_gpu_id()?, root, &sender)
                    .await
                    .inspect_err(|_| {
                        sender.input(AppMsg::ReloadData { full: false });
                    })?;
            }
            AppMsg::RevertChanges => {
                sender.input(AppMsg::ReloadData { full: false });
            }
            AppMsg::ResetClocks => {
                let gpu_id = self.current_gpu_id()?;
                self.daemon_client
                    .set_clocks_value(&gpu_id, SetClocksCommand::Reset)
                    .await?;
                self.daemon_client
                    .confirm_pending_config(ConfirmCommand::Confirm)
                    .await?;
                sender.input(AppMsg::ReloadData { full: false });
            }
            AppMsg::ResetPmfw => {
                let gpu_id = self.current_gpu_id()?;
                self.daemon_client.reset_pmfw(&gpu_id).await?;
                self.daemon_client
                    .confirm_pending_config(ConfirmCommand::Confirm)
                    .await?;
                sender.input(AppMsg::ReloadData { full: false });
            }
            AppMsg::ShowGraphsWindow => {
                self.graphs_window.emit(GraphsWindowMsg::Show);
            }
            AppMsg::DumpVBios => {
                self.dump_vbios(&self.current_gpu_id()?, root).await;
            }
            AppMsg::DebugSnapshot => {
                self.generate_debug_snapshot(root).await;
            }
            AppMsg::EnableOverdrive => {
                toggle_overdrive(&self.daemon_client, true, root.clone()).await;
            }
            AppMsg::DisableOverdrive => {
                toggle_overdrive(&self.daemon_client, false, root.clone()).await;
            }
            AppMsg::ResetConfig => {
                self.daemon_client.reset_config().await?;
                sender.input(AppMsg::ReloadData { full: true });
            }
            AppMsg::ConnectionStatus(status) => match status {
                ConnectionStatusMsg::Disconnected => widgets.reconnecting_dialog.present(),
                ConnectionStatusMsg::Reconnected => widgets.reconnecting_dialog.hide(),
            },
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
            }
            AppMsg::EvaluateProfile(rule) => {
                match self.daemon_client.evaluate_profile_rule(rule).await {
                    Ok(matches) => {
                        PROFILE_RULE_WINDOW_BROKER
                            .send(ProfileRuleWindowMsg::EvaluationResult(matches));
                    }
                    Err(err) => {
                        warn!("{err:#}");
                    }
                }
            }
            AppMsg::SetProfileRule { name, rule } => {
                self.daemon_client.set_profile_rule(name, rule).await?;
                self.reload_profiles(false).await?;
            }
        }
        Ok(())
    }

    fn current_gpu_id(&self) -> anyhow::Result<String> {
        self.header
            .model()
            .selected_gpu_id()
            .context("No GPU selected")
    }

    async fn reload_profiles(&mut self, include_state: bool) -> anyhow::Result<()> {
        let mut profiles = self.daemon_client.list_profiles(include_state).await?;

        if let Some(state) = profiles.watcher_state.take() {
            PROFILE_RULE_WINDOW_BROKER.send(ProfileRuleWindowMsg::WatcherState(state));
        }

        self.header.emit(HeaderMsg::Profiles(Box::new(profiles)));

        Ok(())
    }

    async fn update_gpu_data_full(
        &mut self,
        gpu_id: String,
        sender: AsyncComponentSender<AppModel>,
    ) -> anyhow::Result<()> {
        let daemon_client = self.daemon_client.clone();
        let info_buf = daemon_client
            .get_device_info(&gpu_id)
            .await
            .context("Could not fetch info")?;
        let info = Arc::new(info_buf);

        // Plain `nvidia` means that the nvidia driver is loaded, but it does not contain a version fetched from NVML
        if info.driver == "nvidia" {
            sender.input(AppMsg::Error(Arc::new(anyhow!("Nvidia driver detected, but the management library could not be loaded. Check lact service status for more information."))));
        }

        self.info_page.emit(PageUpdate::Info(info.clone()));

        self.oc_page.set_info(&info);

        let vram_clock_ratio = info
            .drm_info
            .as_ref()
            .map(|info| info.vram_clock_ratio)
            .unwrap_or(1.0);
        self.graphs_window
            .emit(GraphsWindowMsg::VramClockRatio(vram_clock_ratio));

        self.update_gpu_data(gpu_id, sender).await?;

        self.thermals_page.set_info(&info);

        self.graphs_window.emit(GraphsWindowMsg::Clear);

        Ok(())
    }

    async fn update_gpu_data(
        &mut self,
        gpu_id: String,
        sender: AsyncComponentSender<AppModel>,
    ) -> anyhow::Result<()> {
        if let Some(stats_task) = self.stats_task_handle.take() {
            stats_task.abort();
        }

        debug!("updating info for gpu {gpu_id}");

        let stats = self
            .daemon_client
            .get_device_stats(&gpu_id)
            .await
            .context("Could not fetch stats")?;
        let stats = Arc::new(stats);

        self.oc_page.set_stats(&stats, true);
        self.thermals_page.set_stats(&stats, true);

        self.info_page.emit(PageUpdate::Stats(stats));

        let maybe_clocks_table = match self.daemon_client.get_device_clocks_info(&gpu_id).await {
            Ok(info) => info.table,
            Err(err) => {
                debug!("could not fetch clocks info: {err:?}");
                None
            }
        };
        self.oc_page.set_clocks_table(maybe_clocks_table);

        let maybe_modes_table = match self
            .daemon_client
            .get_device_power_profile_modes(&gpu_id)
            .await
        {
            Ok(buf) => Some(buf),
            Err(err) => {
                debug!("Could not get profile modes table: {err:?}");
                None
            }
        };
        self.oc_page
            .performance_frame
            .set_power_profile_modes(maybe_modes_table);

        match self.daemon_client.get_power_states(&gpu_id).await {
            Ok(power_states) => {
                self.oc_page
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

        self.thermals_page
            .connect_settings_changed(show_revealer.clone());

        self.oc_page.connect_settings_changed(show_revealer);

        self.apply_revealer
            .sender()
            .send(ApplyRevealerMsg::Hide)
            .unwrap();

        self.stats_task_handle = Some(start_stats_update_loop(
            gpu_id.to_owned(),
            self.daemon_client.clone(),
            sender,
            self.header.sender().clone(),
        ));

        Ok(())
    }

    async fn apply_settings(
        &self,
        gpu_id: String,
        root: &gtk::ApplicationWindow,
        sender: &AsyncComponentSender<Self>,
    ) -> anyhow::Result<()> {
        // TODO: Ask confirmation for everything, not just clocks

        debug!("applying settings on gpu {gpu_id}");

        if let Some(cap) = self.oc_page.get_power_cap() {
            self.daemon_client
                .set_power_cap(&gpu_id, Some(cap))
                .await
                .context("Failed to set power cap")?;

            self.daemon_client
                .confirm_pending_config(ConfirmCommand::Confirm)
                .await
                .context("Could not commit config")?;
        }

        // Reset the power profile mode for switching to/from manual performance level
        self.daemon_client
            .set_power_profile_mode(&gpu_id, None, vec![])
            .await
            .context("Could not set default power profile mode")?;
        self.daemon_client
            .confirm_pending_config(ConfirmCommand::Confirm)
            .await
            .context("Could not commit config")?;

        if let Some(level) = self.oc_page.get_performance_level() {
            self.daemon_client
                .set_performance_level(&gpu_id, level)
                .await
                .context("Failed to set power profile")?;
            self.daemon_client
                .confirm_pending_config(ConfirmCommand::Confirm)
                .await
                .context("Could not commit config")?;

            let mode_index = self
                .oc_page
                .performance_frame
                .get_selected_power_profile_mode();
            let custom_heuristics = self
                .oc_page
                .performance_frame
                .get_power_profile_mode_custom_heuristics();

            self.daemon_client
                .set_power_profile_mode(&gpu_id, mode_index, custom_heuristics)
                .await
                .context("Could not set active power profile mode")?;
            self.daemon_client
                .confirm_pending_config(ConfirmCommand::Confirm)
                .await
                .context("Could not commit config")?;
        }

        if let Some(thermals_settings) = self.thermals_page.get_thermals_settings() {
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
                .await
                .context("Could not set fan control")?;
            self.daemon_client
                .confirm_pending_config(ConfirmCommand::Confirm)
                .await
                .context("Could not commit config")?;
        }

        let clocks_commands = self.oc_page.clocks_frame.get_commands();

        debug!("applying clocks commands {clocks_commands:#?}");

        let enabled_power_states = self.oc_page.get_enabled_power_states();

        for (kind, states) in enabled_power_states {
            if !states.is_empty() {
                self.daemon_client
                    .set_enabled_power_states(&gpu_id, kind, states)
                    .await
                    .context("Could not set power states")?;

                self.daemon_client
                    .confirm_pending_config(ConfirmCommand::Confirm)
                    .await
                    .context("Could not commit config")?;
            }
        }

        if !clocks_commands.is_empty() {
            let delay = self
                .daemon_client
                .batch_set_clocks_value(&gpu_id, clocks_commands)
                .await
                .context("Could not commit clocks settings")?;
            self.ask_settings_confirmation(delay, root, sender).await;
        }

        sender.input(AppMsg::ReloadData { full: false });

        Ok(())
    }

    async fn ask_settings_confirmation(
        &self,
        mut delay: u64,
        window: &gtk::ApplicationWindow,
        sender: &AsyncComponentSender<AppModel>,
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

                relm4::spawn_local(async move {
                    if let Err(err) = daemon_client.confirm_pending_config(command).await {
                        show_error(&window, &err);
                    }
                    sender.input(AppMsg::ReloadData { full: false });
                });
            }
        ));
    }

    async fn dump_vbios(&self, gpu_id: &str, root: &gtk::ApplicationWindow) {
        match self.daemon_client.dump_vbios(gpu_id).await {
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

    async fn generate_debug_snapshot(&self, root: &gtk::ApplicationWindow) {
        match self.daemon_client.generate_debug_snapshot().await {
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

    let errors_count = ERROR_WINDOW_COUNT.load(Ordering::SeqCst);
    if errors_count > 2 {
        warn!("Not showing error window, too many already open");
        return;
    }

    ERROR_WINDOW_COUNT.fetch_add(1, Ordering::SeqCst);

    let diag = MessageDialog::builder()
        .title("Error")
        .message_type(MessageType::Error)
        .text(text)
        .buttons(ButtonsType::Close)
        .transient_for(parent)
        .build();
    diag.run_async(|diag, _| {
        diag.close();
        ERROR_WINDOW_COUNT.fetch_sub(1, Ordering::SeqCst);
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
    sender: AsyncComponentSender<AppModel>,
    header_sender: relm4::Sender<HeaderMsg>,
) -> glib::JoinHandle<()> {
    debug!("spawning new stats update task with {STATS_POLL_INTERVAL_MS}ms interval");
    let duration = Duration::from_millis(STATS_POLL_INTERVAL_MS);
    relm4::spawn_local(async move {
        loop {
            tokio::time::sleep(duration).await;

            match daemon_client.get_device_stats(&gpu_id).await {
                Ok(stats) => {
                    sender.input(AppMsg::Stats(Arc::new(stats)));
                }
                Err(err) => {
                    error!("could not fetch stats: {err:#}");
                }
            }

            match daemon_client.list_profiles(false).await {
                Ok(profiles) => {
                    let _ = header_sender.send(HeaderMsg::Profiles(Box::new(profiles)));
                }
                Err(err) => {
                    error!("could not fetch profile info: {err:#}");
                }
            }
        }
    })
}

fn oc_toggled_dialog(enabled: bool, msg: &str) {
    let enabled_text = if enabled { "enabled" } else { "disabled" };

    let child = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(5)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(10)
        .margin_end(10)
        .build();
    child.append(&gtk::Label::new(Some(&format!("Overclocking {enabled_text}. A system reboot is required to apply the changes.\nSystem message:"))));

    let msg_label = gtk::Label::builder()
        .label(msg)
        .valign(gtk::Align::Start)
        .halign(gtk::Align::Start)
        .build();
    let msg_scrollable = gtk::ScrolledWindow::builder().child(&msg_label).build();
    child.append(&msg_scrollable);

    let ok_button = gtk::Button::builder().label("OK").build();
    child.append(&ok_button);

    let success_dialog = MessageDialog::builder()
        .title("Overclock info")
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

async fn toggle_overdrive(daemon_client: &DaemonClient, enable: bool, root: ApplicationWindow) {
    let dialog = spinner_dialog(&root, "Regenerating initramfs (this may take a while)");
    dialog.show();

    let result = if enable {
        daemon_client.enable_overdrive().await
    } else {
        daemon_client.disable_overdrive().await
    };

    dialog.hide();

    match result {
        Ok(msg) => oc_toggled_dialog(false, &msg),
        Err(err) => {
            show_error(&root, &err);
        }
    }
}

fn spinner_dialog(parent: &ApplicationWindow, title: &str) -> MessageDialog {
    let spinner = gtk::Spinner::new();
    spinner.start();
    spinner.set_margin_top(10);
    spinner.set_margin_bottom(10);

    let dialog = MessageDialog::builder()
        .title(title)
        .child(&spinner)
        .message_type(MessageType::Info)
        .transient_for(parent)
        .build();

    if let Some(bar) = dialog.titlebar() {
        bar.set_margin_start(15);
        bar.set_margin_end(15);
    }

    dialog
}

fn register_actions(sender: &AsyncComponentSender<AppModel>) {
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
                "Disable Overclocking",
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

async fn create_connection() -> anyhow::Result<(DaemonClient, Option<anyhow::Error>)> {
    match DaemonClient::connect().await {
        Ok(connection) => {
            debug!("Established daemon connection");
            Ok((connection, None))
        }
        Err(err) => {
            info!("could not connect to socket: {err:#}");
            info!("using a local daemon");

            let (server_stream, client_stream) = UnixStream::pair()?;

            std::thread::spawn(move || {
                if let Err(err) = lact_daemon::run_embedded(server_stream) {
                    error!("Builtin daemon error: {err}");
                }
            });

            let client = DaemonClient::from_stream(client_stream, true)?;
            Ok((client, Some(err)))
        }
    }
}
