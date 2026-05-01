mod about_dialog;
mod ext;
pub(crate) mod formatting;
mod gpu_selector;
pub mod graphs_window;
mod info_dialog;
mod info_row;
mod info_row_level;
pub(crate) mod msg;
mod overdrive_dialog;
mod page_section;
mod page_section_expander;
pub(crate) mod pages;
mod preferences_dialog;
mod process_monitor;
mod profiles;
pub(crate) mod styles;

use crate::{
    APP_ID, CONFIG, GUI_VERSION, I18N,
    app::{
        about_dialog::{AboutDialog, AboutDialogMsg},
        gpu_selector::GpuSelector,
        info_dialog::{
            InfoDialog, InfoDialogConfirmation, InfoDialogData, InfoDialogId, InfoDialogMsg,
        },
        overdrive_dialog::{OverdriveDialog, OverdriveDialogMsg},
        preferences_dialog::{PreferencesDialog, PreferencesDialogMsg},
        process_monitor::{ProcessMonitorWindow, ProcessMonitorWindowMsg},
        profiles::{
            ProfileSelector, ProfileSelectorMsg,
            profile_rule_window::{ProfileRuleWindowMsg, profile_rule_row::ProfileRuleRowMsg},
        },
    },
};
use adw::prelude::*;
use anyhow::{Context, anyhow};
use ext::RelmDefaultLauchable;
use graphs_window::{GraphsWindow, GraphsWindowMsg};
use gtk::{
    FileChooserAction, FileChooserDialog, ResponseType, STYLE_PROVIDER_PRIORITY_APPLICATION,
    glib::{self, clone},
};
use i18n_embed_fl::fl;
use lact_client::{ConnectionStatusMsg, DaemonClient};
use lact_schema::{
    DeviceFlag, DeviceStats, GIT_COMMIT, SystemInfo,
    args::GuiArgs,
    config::{GpuConfig, Profile},
    request::{ConfirmCommand, ProfileBase, SetClocksCommand},
};
use msg::AppMsg;
use pages::{
    PageUpdate,
    crash_page::CrashPage,
    info_page::InformationPage,
    oc_page::{OcPage, OcPageMsg},
    software_page::{SoftwarePage, SoftwarePageMsg},
    thermals_page::{ThermalsPage, ThermalsPageMsg},
};
use relm4::{
    AsyncComponentSender, Component, ComponentController, MessageBroker, RelmObjectExt,
    RelmWidgetExt,
    binding::BoolBinding,
    css,
    prelude::{AsyncComponent, AsyncComponentParts},
    tokio::{self, time::sleep},
};
use relm4_components::{
    open_dialog::{OpenDialog, OpenDialogMsg, OpenDialogResponse, OpenDialogSettings},
    save_dialog::{SaveDialog, SaveDialogMsg, SaveDialogResponse, SaveDialogSettings},
};
use std::{fs, os::unix::net::UnixStream, path::PathBuf, sync::Arc, time::Duration};
use tracing::{debug, error, info, trace, warn};

pub(crate) static APP_BROKER: MessageBroker<AppMsg> = MessageBroker::new();

const PROCESS_POLL_INTERVAL_MS: u64 = 1500;
const NVIDIA_RECOMMENDED_MIN_VERSION: u32 = 560;
const CONTENT_MAXIMUM_WIDTH: i32 = 1200;

pub struct AppModel {
    daemon_client: DaemonClient,
    graphs_window: relm4::Controller<GraphsWindow>,
    process_monitor_window: relm4::Controller<ProcessMonitorWindow>,
    overdrive_dialog: relm4::Controller<OverdriveDialog>,
    preferences_dialog: relm4::Controller<PreferencesDialog>,
    about_dialog: relm4::Controller<AboutDialog>,
    info_dialog: relm4::Controller<InfoDialog>,

    ui_sensitive: BoolBinding,
    selected_gpu_index: u32,

    info_page: relm4::Controller<InformationPage>,
    oc_page: relm4::Controller<OcPage>,
    thermals_page: relm4::Controller<ThermalsPage>,
    software_page: relm4::Controller<SoftwarePage>,
    crash_page: relm4::Controller<CrashPage>,

    gpu_selector: relm4::Controller<GpuSelector>,
    profile_selector: relm4::Controller<ProfileSelector>,
    stats_task_handle: Option<glib::JoinHandle<()>>,

    settings_changed: BoolBinding,

    system_info: SystemInfo,
    device_flags: Vec<DeviceFlag>,
    device_driver: String,
}

#[derive(Debug)]
pub enum CommandOutput {
    ProfileImport(PathBuf),
    Error(anyhow::Error),
}

#[relm4::component(pub, async)]
impl AsyncComponent for AppModel {
    type Init = GuiArgs;

    type Input = AppMsg;
    type Output = ();
    type CommandOutput = Option<CommandOutput>;

    view! {
        #[root]
        adw::ApplicationWindow::builder()
            .default_height(750)
            .default_width(1100)
            .icon_name(APP_ID)
            .title("LACT")
            .build() {
                #[name = "toast_overlay"]
                adw::ToastOverlay {
                    #[wrap(Some)]
                    #[name = "root_box"]
                    set_child = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    #[name = "navbar"]
                    adw::NavigationSplitView {
                        set_expand: true,
                        set_max_sidebar_width: 230.0,

                        #[wrap(Some)]
                        set_sidebar = &adw::NavigationPage {
                            set_title: "LACT",

                            #[wrap(Some)]
                            #[name = "sidebar_view"]
                            set_child = &adw::ToolbarView {
                                add_top_bar = &adw::HeaderBar {
                                    set_show_end_title_buttons: false,
                                },

                                #[wrap(Some)]
                                set_content = &gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_vexpand: true,
                                    add_css_class: "main-sidebar-container",

                                    model.gpu_selector.widget().clone() {},

                                    model.profile_selector.widget().clone() {},

                                    gtk::Separator {},

                                    gtk::StackSidebar {
                                        set_margin_vertical: 1,
                                        set_stack: &root_stack,
                                        set_vexpand: true,
                                    },

                                    gtk::Separator {},

                                    gtk::Box {
                                        set_orientation: gtk::Orientation::Vertical,
                                        set_spacing: 6,
                                        set_margin_all: 8,
                                        add_binding: (&model.settings_changed, "sensitive"),

                                        gtk::Button {
                                            set_label: &fl!(I18N, "apply-button"),
                                            add_css_class: css::SUGGESTED_ACTION,
                                            set_width_request: 150,
                                            connect_clicked[sender] => move |_| {
                                                sender.input(AppMsg::ApplyChanges);
                                            },
                                        },

                                        gtk::Button {
                                            set_label: &fl!(I18N, "revert-button"),
                                            connect_clicked[sender] => move |_| {
                                                sender.input(AppMsg::RevertChanges);
                                            },
                                        }
                                    }
                                },
                            },
                        },

                        #[wrap(Some)]
                        #[name = "content_page"]
                        set_content = &adw::NavigationPage {

                            #[wrap(Some)]
                            set_child = &adw::ToolbarView {
                                #[name = "content_header"]
                                add_top_bar = &adw::HeaderBar {

                                    pack_end = &gtk::MenuButton {
                                        set_icon_name: "open-menu-symbolic",

                                        #[wrap(Some)]
                                        #[name = "header_menu_popover"]
                                        set_popover = &gtk::Popover {

                                            gtk::Box {
                                                set_orientation: gtk::Orientation::Vertical,
                                                add_css_class: "header-settings-popover-container",

                                                gtk::Button {
                                                    set_label: &fl!(I18N, "show-process-monitor"),
                                                    connect_clicked[header_menu_popover] => move |_| {
                                                        header_menu_popover.popdown();
                                                        APP_BROKER.send(AppMsg::ShowProcessMonitor);
                                                    },
                                                    add_css_class: "flat",
                                                },

                                                gtk::Separator {},

                                                gtk::Button {
                                                    set_label: &fl!(I18N, "generate-debug-snapshot"),
                                                    connect_clicked[header_menu_popover] => move |_| {
                                                        header_menu_popover.popdown();
                                                        APP_BROKER.send(AppMsg::DebugSnapshot);
                                                    },
                                                    add_css_class: "flat",
                                                },

                                                gtk::Button {
                                                    set_label: &fl!(I18N, "dump-vbios"),
                                                    connect_clicked[header_menu_popover] => move |_| {
                                                        header_menu_popover.popdown();
                                                        APP_BROKER.send(AppMsg::DumpVBios);
                                                    },
                                                    add_css_class: "flat",
                                                    #[watch]
                                                    set_sensitive: model.device_flags.contains(&DeviceFlag::DumpableVBios),
                                                },

                                                gtk::Separator {},

                                                gtk::Button {
                                                    set_label: &fl!(I18N, "preferences"),
                                                    connect_clicked[header_menu_popover] => move |_| {
                                                        header_menu_popover.popdown();
                                                        APP_BROKER.send(AppMsg::ShowPreferencesDialog);
                                                    },
                                                    add_css_class: "flat",
                                                },

                                                gtk::Button {
                                                    set_label: &fl!(I18N, "about"),
                                                    connect_clicked[header_menu_popover] => move |_| {
                                                        header_menu_popover.popdown();
                                                        APP_BROKER.send(AppMsg::ShowAboutDialog);
                                                    },
                                                    add_css_class: "flat",
                                                },
                                            }
                                        },
                                    },

                                    pack_end = &gtk::Button {
                                        set_label: &fl!(I18N, "show-historical-charts"),
                                        connect_clicked => move |_| APP_BROKER.send(AppMsg::ShowGraphsWindow),
                                    },
                                },

                                add_top_bar = &adw::Banner {
                                    #[watch]
                                    set_revealed: model.system_info.amdgpu_overdrive_enabled == Some(false) && model.device_driver == "amdgpu",
                                    set_title: &fl!(I18N, "amd-oc-disabled"),
                                    set_use_markup: true,
                                    set_button_label: Some(&fl!(I18N, "enable-amd-oc")),

                                    connect_button_clicked => AppMsg::ShowOverdriveDialog,
                                },

                                #[wrap(Some)]
                                set_content = &gtk::ScrolledWindow {
                                    set_hscrollbar_policy: gtk::PolicyType::Never,

                                    adw::Clamp {
                                        set_maximum_size: CONTENT_MAXIMUM_WIDTH,
                                        set_tightening_threshold: CONTENT_MAXIMUM_WIDTH,

                                        #[name = "root_stack"]
                                        gtk::Stack {
                                            set_vexpand: false,
                                            set_vhomogeneous: false,

                                            add_binding: (&model.ui_sensitive, "sensitive"),

                                            add_titled[Some("info_page"), &fl!(I18N, "info-page")] = model.info_page.widget(),
                                            add_titled[Some("oc_page"), &fl!(I18N, "oc-page")] = model.oc_page.widget(),
                                            add_titled[Some("thermals_page"), &fl!(I18N, "thermals-page")] = model.thermals_page.widget(),
                                            add_titled[Some("software_page"), &fl!(I18N, "software-page")] = model.software_page.widget(),
                                            add_named[Some("crash_page")] = model.crash_page.widget(),

                                            set_visible_child_name: &CONFIG.read().selected_tab,
                                            connect_visible_child_name_notify[content_page] => move |stack| {
                                                if let Some(child) = stack.visible_child() {
                                                    let page = stack.page(&child);
                                                    content_page.set_title(&page.title().unwrap_or_default());

                                                    let name = stack.visible_child_name().unwrap().to_string();
                                                    if name != "crash_page" {
                                                        CONFIG.write().edit(|config| {
                                                            config.selected_tab = name;
                                                        });
                                                    }
                                                }
                                            },
                                        }
                                    }
                                },
                            }
                        }
                    },
                }
                }
            },

        #[name = "reconnecting_dialog"]
        adw::Dialog {
            set_title: &fl!(I18N, "daemon-connection-lost"),
            set_content_width: 300,
            set_content_height: 80,
            set_can_close: false,

            #[wrap(Some)]
            set_child = &gtk::Label {
                set_margin_all: 10,
                set_label: &fl!(I18N, "reconnecting-to-daemon"),
            }
        },
    }

    async fn init(
        args: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        relm4::set_global_css_with_priority(
            styles::COMBINED_CSS,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        if let Err(err) = styles::apply_theme(CONFIG.read().theme) {
            error!("could not apply theme: {err:#}");
        }

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

        let settings_changed = BoolBinding::new(false);

        let system_info = daemon_client
            .get_system_info()
            .await
            .expect("Could not fetch system info");

        let devices = daemon_client
            .list_devices()
            .await
            .expect("Could not list devices");

        let version_mismatch_info = (system_info.version != GUI_VERSION
            || system_info.commit.as_deref() != Some(GIT_COMMIT))
        .then(|| InfoDialogData {
            id: InfoDialogId::VersionMismatch,
            heading: fl!(I18N, "version-mismatch"),
            body: fl!(
                I18N,
                "version-mismatch-description",
                gui_version = GUI_VERSION,
                gui_commit = GIT_COMMIT,
                daemon_version = system_info.version.as_str(),
                daemon_commit = system_info.commit.as_deref().unwrap_or_default()
            ),
            stacktrace: None,
            selectable_text: Some("sudo systemctl restart lactd".to_string()),
            confirmation: None,
        });

        let info_page = InformationPage::detach_default();

        let oc_page = OcPage::builder()
            .launch(settings_changed.clone())
            .forward(sender.input_sender(), |msg| msg);
        let thermals_page = ThermalsPage::builder().launch(()).detach();

        let software_page = SoftwarePage::builder()
            .launch((system_info.clone(), daemon_client.embedded))
            .detach();

        let crash_page = CrashPage::builder()
            .launch(String::new())
            .forward(sender.input_sender(), |msg| msg);

        let overdrive_dialog = OverdriveDialog::builder()
            .launch((system_info.clone(), root.clone().upcast()))
            .detach();

        let preferences_dialog = PreferencesDialog::builder()
            .launch((system_info.clone(), root.clone()))
            .detach();

        let about_dialog = AboutDialog::builder().launch(root.clone()).detach();
        let info_dialog = InfoDialog::builder().launch(root.clone()).detach();

        let graphs_window = GraphsWindow::detach_default();
        let process_monitor_window = ProcessMonitorWindow::detach_default();

        let gpu_selector = GpuSelector::builder()
            .launch(devices)
            .forward(sender.input_sender(), |gpu_idx| {
                AppMsg::GpuSelected(gpu_idx)
            });

        let profile_selector = ProfileSelector::builder()
            .launch(())
            .forward(sender.input_sender(), |msg| msg);

        let model = AppModel {
            daemon_client,
            graphs_window,
            process_monitor_window,
            overdrive_dialog,
            preferences_dialog,
            about_dialog,
            info_dialog,
            info_page,
            oc_page,
            thermals_page,
            software_page,
            crash_page,
            gpu_selector,
            profile_selector,
            ui_sensitive: BoolBinding::new(false),
            selected_gpu_index: 0,
            stats_task_handle: None,
            settings_changed,
            system_info,
            device_flags: vec![],
            device_driver: String::new(),
        };

        let widgets = view_output!();

        if let Some(child) = widgets.root_stack.visible_child() {
            let page = widgets.root_stack.page(&child);
            widgets
                .content_page
                .set_title(&page.title().unwrap_or_default());
        }

        if let Some(err) = conn_err {
            let error_text = format!("Error info: {err:#}\n\n");

            model.info_dialog.emit(InfoDialogMsg::Show(InfoDialogData {
                id: InfoDialogId::EmbeddedDaemonInfo,
                heading: fl!(I18N, "daemon-info-heading"),
                body: fl!(I18N, "embedded-daemon-info", error_info = error_text),
                stacktrace: None,
                selectable_text: Some("sudo systemctl enable --now lactd".to_string()),
                confirmation: None,
            }));
        }

        if let Some(info) = version_mismatch_info {
            model.info_dialog.emit(InfoDialogMsg::Show(info));
        }

        sender.input(AppMsg::ReloadProfiles { state_sender: None });

        let task_sender = sender.clone();
        sender.command(move |_, shutdown| {
            shutdown
                .register(async move {
                    loop {
                        sleep(Duration::from_millis(PROCESS_POLL_INTERVAL_MS)).await;
                        task_sender.input(AppMsg::FetchProcessList);
                    }
                })
                .drop_on_shutdown()
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
            self.info_dialog.emit(InfoDialogMsg::Show(InfoDialogData {
                id: InfoDialogId::Error,
                heading: fl!(I18N, "error-heading"),
                body: format!("{err:#}"),
                stacktrace: Some(format!("{err:?}")),
                selectable_text: None,
                confirmation: None,
            }));
        }
        self.update_view(widgets, sender);
    }

    async fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        if let Some(msg) = msg
            && let Err(err) = self.handle_cmd_output(msg, &sender).await
        {
            sender.input(AppMsg::Error(Arc::new(err)));
        }
    }
}

impl AppModel {
    async fn handle_msg(
        &mut self,
        msg: AppMsg,
        sender: AsyncComponentSender<Self>,
        root: &adw::ApplicationWindow,
        widgets: &AppModelWidgets,
    ) -> Result<(), Arc<anyhow::Error>> {
        match msg {
            AppMsg::Error(err) => return Err(err),
            AppMsg::SettingsChanged => {
                self.settings_changed.set_value(true);
            }
            AppMsg::ReloadProfiles { state_sender } => {
                self.reload_profiles(state_sender).await?;
                sender.input(AppMsg::ReloadData { full: false });
            }
            AppMsg::GpuSelected(idx) => {
                self.selected_gpu_index = idx;
                sender.input(AppMsg::ReloadData { full: true });
            }
            AppMsg::ReloadData { full } => {
                self.settings_changed.set_value(false);

                let gpu_id = self.current_gpu_id()?;
                if full {
                    self.update_gpu_data_full(gpu_id, sender).await?;
                } else {
                    self.update_gpu_data(gpu_id, sender).await?;
                }
            }
            AppMsg::ShowPreferencesDialog => {
                self.preferences_dialog.emit(PreferencesDialogMsg::Show);
            }
            AppMsg::ShowAboutDialog => {
                self.about_dialog.emit(AboutDialogMsg::Show);
            }
            AppMsg::ShowOverdriveDialog => {
                self.overdrive_dialog.emit(OverdriveDialogMsg::Show);
            }
            AppMsg::SelectProfile {
                profile,
                auto_switch,
            } => {
                self.daemon_client.set_profile(profile, auto_switch).await?;
                sender.input(AppMsg::ReloadProfiles { state_sender: None });
            }
            AppMsg::CreateProfile(name, base) => {
                self.daemon_client
                    .create_profile(name.clone(), base)
                    .await?;

                let auto_switch = self.profile_selector.model().auto_switch_profiles();
                self.daemon_client
                    .set_profile(Some(name), auto_switch)
                    .await?;

                sender.input(AppMsg::ReloadProfiles { state_sender: None });
            }
            AppMsg::RenameProfile(old_name, new_name) => {
                if old_name != new_name {
                    let original_profile = self
                        .daemon_client
                        .get_profile(Some(old_name.clone()))
                        .await
                        .context("Could not get profile by old name")?
                        .context("Original profile not found")?;
                    self.daemon_client
                        .create_profile(new_name, ProfileBase::Provided(original_profile))
                        .await
                        .context("Could not create new profile")?;
                    self.daemon_client
                        .delete_profile(old_name)
                        .await
                        .context("Could not delete old name")?;

                    sender.input(AppMsg::ReloadProfiles { state_sender: None });
                }
            }
            AppMsg::DeleteProfile(profile) => {
                self.daemon_client.delete_profile(profile).await?;
                sender.input(AppMsg::ReloadProfiles { state_sender: None });
            }
            AppMsg::MoveProfile(name, new_position) => {
                self.daemon_client.move_profile(name, new_position).await?;
                sender.input(AppMsg::ReloadProfiles { state_sender: None });
            }
            AppMsg::ImportProfile => {
                let json_filter = gtk::FileFilter::new();
                json_filter.add_mime_type("application/json");

                let settings = OpenDialogSettings {
                    filters: vec![json_filter],
                    ..Default::default()
                };
                let file_picker = OpenDialog::builder().launch(settings);
                file_picker.emit(OpenDialogMsg::Open);
                let stream = file_picker.into_stream();

                sender.oneshot_command(async move {
                    if let Some(OpenDialogResponse::Accept(path)) = stream.recv_one().await {
                        Some(CommandOutput::ProfileImport(path))
                    } else {
                        None
                    }
                });
            }
            AppMsg::ExportProfile(name) => {
                if let Some(profile) = self.daemon_client.get_profile(name.clone()).await? {
                    let settings = SaveDialogSettings {
                        create_folders: true,
                        is_modal: true,
                        ..Default::default()
                    };
                    let diag = SaveDialog::builder().launch(settings);
                    diag.emit(SaveDialogMsg::SaveAs(format!(
                        "LACT-profile-{}.json",
                        name.as_deref().unwrap_or("default")
                    )));

                    let stream = diag.into_stream();

                    sender.oneshot_command(async move {
                        if let Some(SaveDialogResponse::Accept(path)) = stream.recv_one().await {
                            let contents = serde_json::to_string(&profile)
                                .expect("Could not serialize profile");

                            if let Err(err) =
                                fs::write(path, contents).context("Could not export profile")
                            {
                                return Some(CommandOutput::Error(err));
                            }
                        }
                        None
                    });
                }
            }
            AppMsg::Stats(stats) => {
                let update = PageUpdate::Stats(stats.clone());
                self.oc_page.emit(OcPageMsg::Update {
                    update: update.clone(),
                    initial: false,
                });
                self.thermals_page.emit(ThermalsPageMsg::Update {
                    update: update.clone(),
                    initial: false,
                });
                self.graphs_window.emit(GraphsWindowMsg::Stats {
                    stats,
                    selected_gpu_id: None,
                });
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
                    .set_clocks_value(&gpu_id, SetClocksCommand::reset())
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
            AppMsg::ShowProcessMonitor => {
                self.process_monitor_window
                    .emit(ProcessMonitorWindowMsg::Show);
            }
            AppMsg::DumpVBios => {
                self.dump_vbios(&self.current_gpu_id()?, root, sender.clone())
                    .await?;
            }
            AppMsg::DebugSnapshot => {
                self.generate_debug_snapshot(root, widgets)
                    .await
                    .context("Could not generate snapshot")?;
            }
            AppMsg::EnableOverdrive => {
                self.overdrive_dialog.emit(OverdriveDialogMsg::Loading);
                let result = self.daemon_client.enable_overdrive().await;
                self.overdrive_dialog.emit(OverdriveDialogMsg::Loaded);
                result?;
            }
            AppMsg::DisableOverdrive => {
                self.overdrive_dialog.emit(OverdriveDialogMsg::Loading);
                let result = self.daemon_client.disable_overdrive().await;
                self.overdrive_dialog.emit(OverdriveDialogMsg::Loaded);
                result?;
            }
            AppMsg::ResetConfig => {
                let sender = sender.clone();
                self.info_dialog.emit(InfoDialogMsg::ShowConfirmation(
                    InfoDialogData {
                        id: InfoDialogId::ResetConfigConfirmation,
                        heading: fl!(I18N, "reset-config"),
                        body: fl!(I18N, "reset-config-description"),
                        stacktrace: None,
                        selectable_text: None,
                        confirmation: Some(InfoDialogConfirmation {
                            confirm_label: fl!(I18N, "reset-button"),
                            cancel_label: fl!(I18N, "cancel"),
                            appearance: adw::ResponseAppearance::Destructive,
                            timeout_seconds: None,
                        }),
                    },
                    Box::new(move || sender.input(AppMsg::ResetConfigConfirmed)),
                ));
            }
            AppMsg::ResetConfigConfirmed => {
                self.daemon_client.reset_config().await?;
                sender.input(AppMsg::ReloadData { full: true });
            }
            AppMsg::FetchProcessList => {
                if self.process_monitor_window.widget().is_visible()
                    && let Ok(gpu_id) = self.current_gpu_id()
                {
                    match self.daemon_client.get_process_list(&gpu_id).await {
                        Ok(process_list) => {
                            self.process_monitor_window
                                .emit(ProcessMonitorWindowMsg::Data(process_list));
                        }
                        Err(err) => {
                            warn!("could not fetch process list: {err:#}");
                        }
                    }
                }
            }
            AppMsg::ConnectionStatus(status) => match status {
                ConnectionStatusMsg::Disconnected => {
                    widgets.reconnecting_dialog.present(Some(root))
                }
                ConnectionStatusMsg::Reconnected => widgets.reconnecting_dialog.force_close(),
            },
            AppMsg::EvaluateProfile(rule, sender) => {
                match self.daemon_client.evaluate_profile_rule(rule).await {
                    Ok(matches) => {
                        sender.emit(ProfileRuleWindowMsg::EvaluationResult(matches));
                    }
                    Err(err) => {
                        warn!("{err:#}");
                    }
                }
            }
            AppMsg::SetProfileRule { name, rule, hooks } => {
                self.daemon_client
                    .set_profile_rule(name, rule, hooks)
                    .await?;
                self.reload_profiles(None).await?;
            }
            AppMsg::Crash(message) => {
                // we cannot be sure that the application is fully functional after a crash
                // even though the main loop is restored via crash handler, we want user to restart
                // this is why navigation controls are disabled
                widgets.sidebar_view.set_sensitive(false);
                widgets.content_header.set_sensitive(false);
                self.settings_changed.set_value(false);

                self.ui_sensitive.set_value(true);
                widgets.root_stack.set_visible_child_name("crash_page");
                self.crash_page.emit(message);

                if let Some(handle) = self.stats_task_handle.take() {
                    handle.abort();
                }
            }
        }
        Ok(())
    }

    async fn handle_cmd_output(
        &mut self,
        msg: CommandOutput,
        sender: &AsyncComponentSender<AppModel>,
    ) -> anyhow::Result<()> {
        match msg {
            CommandOutput::ProfileImport(path) => {
                let file_name = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("Imported profile");

                let contents = fs::read_to_string(&path).context("Could not read selected file")?;
                let profile = serde_json::from_str::<Profile>(&contents)
                    .context("Could not parse profile")?;
                let profile_name = file_name
                    .trim_start_matches("LACT-profile-")
                    .trim_end_matches(".json");

                self.daemon_client
                    .create_profile(profile_name.to_owned(), ProfileBase::Provided(profile))
                    .await
                    .context("Could not import profile")?;

                sender.input(AppMsg::ReloadProfiles { state_sender: None });
            }
            CommandOutput::Error(error) => return Err(error),
        }

        Ok(())
    }

    fn current_gpu_id(&self) -> anyhow::Result<String> {
        CONFIG
            .read()
            .selected_gpu
            .clone()
            .context("No GPU selected")
    }

    async fn reload_profiles(
        &mut self,
        state_sender: Option<relm4::Sender<ProfileRuleRowMsg>>,
    ) -> anyhow::Result<()> {
        let mut profiles = self
            .daemon_client
            .list_profiles(state_sender.is_some())
            .await?;

        if let Some(sender) = state_sender
            && let Some(state) = profiles.watcher_state.take()
        {
            let _ = sender.send(ProfileRuleRowMsg::WatcherState(state));
        }

        self.profile_selector
            .emit(ProfileSelectorMsg::Profiles(Box::new(profiles)));

        Ok(())
    }

    async fn update_gpu_data_full(
        &mut self,
        gpu_id: String,
        sender: AsyncComponentSender<AppModel>,
    ) -> anyhow::Result<()> {
        self.ui_sensitive.set_value(false);

        let daemon_client = self.daemon_client.clone();
        let info_buf = daemon_client
            .get_device_info(&gpu_id)
            .await
            .context("Could not fetch info")?;
        let info = Arc::new(info_buf);

        // Plain `nvidia` means that the nvidia driver is loaded, but it does not contain a version fetched from NVML
        if info.driver == "nvidia" {
            sender.input(AppMsg::Error(Arc::new(anyhow!("Nvidia driver detected, but the management library could not be loaded. Check lact service status for more information."))));
        } else if let Some(nvidia_version) = info.driver.strip_prefix("nvidia ")
            && let Some(major_version) = nvidia_version
                .split('.')
                .next()
                .and_then(|version| version.parse::<u32>().ok())
            && major_version < NVIDIA_RECOMMENDED_MIN_VERSION
        {
            sender.input(AppMsg::Error(Arc::new(anyhow!("Old Nvidia driver version detected ({major_version}), some features might be missing. Driver version {NVIDIA_RECOMMENDED_MIN_VERSION} or newer is recommended."))));
        }

        self.device_flags = info.flags.clone();
        self.device_driver = info.driver.clone();

        let update = PageUpdate::Info(info.clone());
        self.info_page.emit(update.clone());
        self.oc_page.emit(OcPageMsg::Update {
            update: update.clone(),
            initial: true,
        });
        self.software_page
            .emit(SoftwarePageMsg::DeviceInfo(info.clone()));
        self.thermals_page.emit(ThermalsPageMsg::Update {
            update: update.clone(),
            initial: true,
        });

        let vram_clock_ratio = info
            .drm_info
            .as_ref()
            .map(|info| info.vram_clock_ratio)
            .unwrap_or(1.0);
        self.graphs_window
            .emit(GraphsWindowMsg::VramClockRatio(vram_clock_ratio));

        let stats = self.update_gpu_data(gpu_id.clone(), sender).await?;

        self.graphs_window.emit(GraphsWindowMsg::Stats {
            stats,
            selected_gpu_id: Some(gpu_id),
        });

        self.ui_sensitive.set_value(true);

        Ok(())
    }

    async fn update_gpu_data(
        &mut self,
        gpu_id: String,
        sender: AsyncComponentSender<AppModel>,
    ) -> anyhow::Result<Arc<DeviceStats>> {
        if let Some(stats_task) = self.stats_task_handle.take() {
            stats_task.abort();
        }

        debug!("updating info for gpu {gpu_id}");

        let gpu_config = self
            .daemon_client
            .get_gpu_config(&gpu_id)
            .await
            .ok()
            .flatten();

        let stats = self
            .daemon_client
            .get_device_stats(&gpu_id)
            .await
            .context("Could not fetch stats")?;
        let stats = Arc::new(stats);

        let update = PageUpdate::Stats(stats.clone());
        self.info_page.emit(update.clone());
        self.thermals_page.emit(ThermalsPageMsg::Update {
            update: update.clone(),
            initial: true,
        });
        self.oc_page.emit(OcPageMsg::Update {
            update,
            initial: true,
        });

        let maybe_clocks_table = match self.daemon_client.get_device_clocks_info(&gpu_id).await {
            Ok(info) => info.table,
            Err(err) => {
                debug!("could not fetch clocks info: {err:?}");
                None
            }
        };
        self.oc_page
            .emit(OcPageMsg::ClocksTable(maybe_clocks_table));

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
            .emit(OcPageMsg::ProfileModesTable(maybe_modes_table));

        match self.daemon_client.get_power_states(&gpu_id).await {
            Ok(power_states) => {
                self.oc_page.emit(OcPageMsg::PowerStates {
                    pstates: power_states,
                    configured: gpu_config.is_some_and(|config| !config.power_states.is_empty()),
                });
            }
            Err(err) => warn!("could not get power states: {err:?}"),
        }

        self.stats_task_handle = Some(start_stats_update_loop(
            gpu_id.to_owned(),
            self.daemon_client.clone(),
            sender,
            self.profile_selector.sender().clone(),
        ));

        Ok(stats)
    }

    async fn apply_settings(
        &self,
        gpu_id: String,
        _root: &adw::ApplicationWindow,
        sender: &AsyncComponentSender<Self>,
    ) -> anyhow::Result<()> {
        debug!("applying settings on gpu {gpu_id}");

        let mut gpu_config = self
            .daemon_client
            .get_gpu_config(&gpu_id)
            .await
            .context("Could not get gpu config")?
            .unwrap_or_else(GpuConfig::default);

        let cap = self.oc_page.model().get_power_cap();
        if let Some(cap) = cap {
            gpu_config.power_cap = Some(cap);
        }

        let performance_level = self.oc_page.model().get_performance_level();
        if let Some(level) = performance_level {
            gpu_config.performance_level = Some(level);
            gpu_config.power_profile_mode_index = self.oc_page.model().get_power_profile_mode();

            gpu_config.custom_power_profile_mode_hueristics = self
                .oc_page
                .model()
                .get_power_profile_mode_custom_heuristics();
        }

        self.thermals_page.model().apply_config(&mut gpu_config);

        self.oc_page
            .model()
            .apply_clocks_config(&mut gpu_config.clocks_configuration);

        let enabled_power_states = self.oc_page.model().get_enabled_power_states();
        gpu_config.power_states = enabled_power_states;

        let delay = self
            .daemon_client
            .set_gpu_config(&gpu_id, gpu_config)
            .await
            .context("Could not apply settings")?;
        self.ask_settings_confirmation(delay, sender);

        sender.input(AppMsg::ReloadData { full: false });

        Ok(())
    }

    fn ask_settings_confirmation(&self, delay: u64, sender: &AsyncComponentSender<AppModel>) {
        let confirm_pending_config = |command| {
            let sender = sender.clone();
            let daemon_client = self.daemon_client.clone();

            Box::new(move || {
                relm4::spawn_local(async move {
                    if let Err(err) = daemon_client.confirm_pending_config(command).await {
                        sender.input(AppMsg::Error(Arc::new(err)));
                    }
                    sender.input(AppMsg::ReloadData { full: false });
                });
            }) as Box<dyn FnOnce()>
        };

        self.info_dialog.emit(InfoDialogMsg::ShowTimedConfirmation {
            data: InfoDialogData {
                id: InfoDialogId::SettingsConfirmation,
                heading: fl!(I18N, "confirm-settings"),
                body: fl!(I18N, "settings-confirmation", seconds_left = delay),
                stacktrace: None,
                selectable_text: None,
                confirmation: Some(InfoDialogConfirmation {
                    confirm_label: fl!(I18N, "confirm"),
                    cancel_label: fl!(I18N, "revert-button"),
                    appearance: adw::ResponseAppearance::Suggested,
                    timeout_seconds: Some(delay),
                }),
            },
            on_confirmed: confirm_pending_config(ConfirmCommand::Confirm),
            on_closed: confirm_pending_config(ConfirmCommand::Revert),
            on_timed_out: Box::new({
                let sender = sender.clone();

                move || {
                    sender.input(AppMsg::ReloadData { full: false });
                }
            }),
        });
    }

    async fn dump_vbios(
        &self,
        gpu_id: &str,
        root: &adw::ApplicationWindow,
        sender: AsyncComponentSender<Self>,
    ) -> anyhow::Result<()> {
        let vbios_data = self.daemon_client.dump_vbios(gpu_id).await?;

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
            sender,
            move |diag, response| {
                diag.close();

                if response == gtk::ResponseType::Accept
                    && let Some(file) = diag.file()
                {
                    match file.path() {
                        Some(path) => {
                            if let Err(err) = std::fs::write(path, vbios_data)
                                .context("Could not save vbios file")
                            {
                                sender.input(AppMsg::Error(Arc::new(err)));
                            }
                        }
                        None => {
                            sender.input(AppMsg::Error(Arc::new(anyhow!(
                                "Selected file has an invalid path"
                            ))));
                        }
                    }
                }
            }
        ));

        Ok(())
    }

    async fn generate_debug_snapshot(
        &self,
        root: &adw::ApplicationWindow,
        widgets: &AppModelWidgets,
    ) -> anyhow::Result<()> {
        let path = self.daemon_client.generate_debug_snapshot().await?;

        let toast = adw::Toast::builder()
            .title(format!("Debug snapshot saved at {path}"))
            .button_label("Copy path")
            .use_markup(false)
            .build();
        toast.connect_button_clicked(clone!(
            #[strong]
            root,
            #[strong]
            path,
            move |_| {
                root.clipboard().set_text(&path);
            }
        ));
        widgets.toast_overlay.add_toast(toast);

        Ok(())
    }
}

fn start_stats_update_loop(
    gpu_id: String,
    daemon_client: DaemonClient,
    sender: AsyncComponentSender<AppModel>,
    profiles_sender: relm4::Sender<ProfileSelectorMsg>,
) -> glib::JoinHandle<()> {
    debug!("spawning new stats update task");
    relm4::spawn_local(async move {
        loop {
            let duration = Duration::from_millis(CONFIG.read().stats_poll_interval_ms as u64);
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
                    let _ = profiles_sender.send(ProfileSelectorMsg::Profiles(Box::new(profiles)));
                }
                Err(err) => {
                    error!("could not fetch profile info: {err:#}");
                }
            }
        }
    })
}

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
            client_stream.set_nonblocking(true)?;
            server_stream.set_nonblocking(true)?;

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
