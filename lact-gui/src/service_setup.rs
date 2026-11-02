pub mod systemd;

use crate::I18N;
use crate::app::components::info_row::{InfoRow, InfoRowExt};
use crate::app::utils::ext::FlowBoxExt;
use crate::service_setup::systemd::{ManagerProxy, START_MODE_REPLACE, UNIT_NAME, UnitProxy};
use adw::prelude::*;
use anyhow::{Context as _, anyhow};
use i18n_embed_fl::fl;
use lact_client::DaemonClient;
use lact_schema::{SystemInfo, VersionInfo};
use relm4::binding::{BoolBinding, ConnectBinding as _};
use relm4::{
    AsyncComponentSender, RelmWidgetExt,
    css::{self, ERROR, SUCCESS, WARNING},
    prelude::{AsyncComponent, AsyncComponentParts},
    tokio,
};
use std::fmt::Write;
use std::io;
use std::time::Duration;
use tracing::debug;

pub struct ServiceSetupDialog {
    connection_status: ConnectionStatus,
    manager_proxy: ManagerProxy<'static>,
    unit_proxy: UnitProxy<'static>,
    service_logs: gtk::TextBuffer,
    service_state: String,
    autostart_on_start: BoolBinding,
    autostart_on_stop: BoolBinding,
    setup_error: Option<anyhow::Error>,
}

pub struct ServiceSetupDialogParams {
    pub parent: gtk::ApplicationWindow,
    pub initial_client: anyhow::Result<(DaemonClient, SystemInfo)>,
    pub manager_proxy: ManagerProxy<'static>,
    pub unit_proxy: UnitProxy<'static>,
}

#[derive(Debug)]
pub enum ServiceSetupDialogMsg {
    Reconnect,
    StartService,
    RestartService,
    StopService,
    Close,
}

#[relm4::component(pub, async)]
impl AsyncComponent for ServiceSetupDialog {
    type Init = ServiceSetupDialogParams;
    type Input = ServiceSetupDialogMsg;
    type Output = Option<DaemonClient>;
    type CommandOutput = ();

    view! {
        adw::Dialog {
            set_content_width: 600,
            set_title: &fl!(I18N, "service-setup-title"),

            connect_closed => ServiceSetupDialogMsg::Close,

            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {},

                #[wrap(Some)]
                set_content = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 15,
                    set_margin_horizontal: 15,
                    set_margin_bottom: 5,

                    gtk::Label {
                        set_markup: &fl!(I18N, "service-explanation"),
                        add_css_class: css::DIM_LABEL,
                        set_wrap: true,
                        set_xalign: 0.0,
                    },

                    gtk::FlowBox {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_column_spacing: 10,
                        set_row_spacing: 15,
                        set_homogeneous: true,
                        set_min_children_per_line: 2,
                        set_max_children_per_line: 2,
                        set_selection_mode: gtk::SelectionMode::None,

                        append_child = &InfoRow {
                            set_name: fl!(I18N, "service-connection-status"),
                            #[watch]
                            set_value: model.connection_status.status_text(),
                            #[watch]
                            set_value_css_classes: if model.connection_status.is_connected() { &[SUCCESS] } else { &[ERROR] },
                            set_selectable: true,
                        },

                        append_child = &InfoRow {
                            set_name: fl!(I18N, "service-status"),
                            #[watch]
                            set_value: format!("<tt>{}</tt>", model.service_state),
                            set_selectable: true,
                        },

                        append_child = &InfoRow {
                            #[watch]
                            set_name: if model.version_mismatched() {
                                format!("{} ({})", fl!(I18N, "service-version"), fl!(I18N, "service-version-mismatch"))
                            } else {
                                fl!(I18N, "service-version")
                            },
                            #[watch]
                            set_value: model.service_version_text().unwrap_or_else(|| fl!(I18N, "missing-stat")),
                            #[watch]
                            set_value_css_classes: match &model.connection_status {
                                ConnectionStatus::Connected { version, .. } => {
                                    if version.is_current() {
                                        &[SUCCESS]
                                    } else {
                                        &[WARNING]
                                    }
                                }
                                ConnectionStatus::Error(_) => &[],
                            },
                            set_selectable: true,
                        },

                        append_child = &InfoRow {
                            set_name: fl!(I18N, "gui-version"),
                            set_value: format_version(&VersionInfo::current()),
                            set_selectable: true,
                        },
                    },

                    gtk::Label {
                        #[watch]
                        set_visible: model.connection_status.error_text().is_some(),
                        #[watch]
                        set_markup: model.connection_status.error_text().unwrap_or_default(),
                        set_css_classes: &[ERROR],
                        set_wrap: true,
                        set_xalign: 0.0,
                        set_selectable: true,
                    },

                    gtk::Label {
                        #[watch]
                        set_visible: model.setup_error.is_some(),
                        #[watch]
                        set_text: &model
                            .setup_error
                            .as_ref()
                            .map(|err| fl!(I18N, "setup-error", error = err.to_string()))
                            .unwrap_or_default(),
                        set_css_classes: &[ERROR],
                        set_wrap: true,
                        set_xalign: 0.0,
                        set_selectable: true,
                    },
                },

                add_bottom_bar = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 10,
                    set_margin_horizontal: 15,
                    set_margin_vertical: 15,

                    gtk::MenuButton {
                        set_label: &fl!(I18N, "service-logs"),
                        set_valign: gtk::Align::Center,

                        #[wrap(Some)]
                        set_popover = &gtk::Popover {
                            gtk::ScrolledWindow {
                                set_min_content_width: 850,
                                set_min_content_height: 250,

                                gtk::TextView {
                                    set_editable: false,
                                    set_monospace: true,
                                    set_buffer: Some(&model.service_logs),
                                    set_top_margin: 5,
                                    set_bottom_margin: 5,
                                    set_left_margin: 5,
                                    set_right_margin: 5,
                                },
                            },
                        },
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 10,
                        set_hexpand: true,
                        set_halign: gtk::Align::End,

                        gtk::CheckButton {
                            set_label: Some(&fl!(I18N, "service-autostart")),
                            bind: &model.autostart_on_start,

                            #[watch]
                            set_visible: model.service_state != systemd::UNIT_STATE_ACTIVE,
                        },

                        gtk::Button {
                            set_label: &fl!(I18N, "service-start"),
                            connect_clicked => ServiceSetupDialogMsg::StartService,
                            add_css_class: "suggested-action",

                            #[watch]
                            set_visible: model.service_state != systemd::UNIT_STATE_ACTIVE,
                        },

                        gtk::CheckButton {
                            set_label: Some(&fl!(I18N, "service-autostart-disable")),
                            bind: &model.autostart_on_stop,

                            #[watch]
                            set_visible: model.service_state == systemd::UNIT_STATE_ACTIVE,
                        },

                        gtk::Button {
                            set_label: &fl!(I18N, "service-stop"),
                            connect_clicked => ServiceSetupDialogMsg::StopService,

                            #[watch]
                            set_visible: model.service_state == systemd::UNIT_STATE_ACTIVE,
                        },

                        gtk::Button {
                            set_label: &fl!(I18N, "service-restart"),
                            connect_clicked => ServiceSetupDialogMsg::RestartService,
                        },
                    },
                },
            },
        }
    }

    async fn init(
        params: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let input_sender = sender.input_sender().clone();
        relm4::spawn(async move {
            loop {
                if input_sender.send(ServiceSetupDialogMsg::Reconnect).is_err() {
                    debug!("service setup dialog closed, exiting client watcher");
                    break;
                }
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
        });

        let (service_state, setup_error) = params
            .unit_proxy
            .active_state()
            .await
            .map(|state| (state, None))
            .unwrap_or_else(|err| {
                (
                    "unknown".to_owned(),
                    Some(anyhow!("Could not fetch service status: {err}")),
                )
            });

        let service_logs_handle = tokio::spawn(service_logs_text());

        let connection_status = match params.initial_client {
            Ok((client, info)) => ConnectionStatus::Connected {
                client: client.clone(),
                version: info.version,
            },
            Err(err) => ConnectionStatus::from_result(Err(err)).await,
        };

        let model = Self {
            connection_status,
            manager_proxy: params.manager_proxy,
            unit_proxy: params.unit_proxy,
            autostart_on_start: BoolBinding::new(true),
            autostart_on_stop: BoolBinding::new(true),
            service_logs: gtk::TextBuffer::builder()
                .text(service_logs_handle.await.unwrap())
                .build(),
            service_state,
            setup_error,
        };

        let widgets = view_output!();

        root.present(Some(&params.parent));

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        if let Err(err) = self.handle_msg(msg, sender).await {
            self.setup_error = Some(err);
        }
    }
}

impl ServiceSetupDialog {
    async fn handle_msg(
        &mut self,
        msg: ServiceSetupDialogMsg,
        sender: AsyncComponentSender<Self>,
    ) -> anyhow::Result<()> {
        match msg {
            ServiceSetupDialogMsg::Reconnect => (),
            ServiceSetupDialogMsg::StartService => {
                // Note: this order is important, doing it the other way around causes 2 polkit prompts
                if self.autostart_on_start.value() {
                    self.manager_proxy
                        .enable_unit_files(&[UNIT_NAME], false, true)
                        .await
                        .context("could not enable unit")?;
                }

                self.unit_proxy
                    .start(START_MODE_REPLACE)
                    .await
                    .context("could not start unit")?;
            }
            ServiceSetupDialogMsg::RestartService => {
                self.unit_proxy.restart(START_MODE_REPLACE).await?;
            }
            ServiceSetupDialogMsg::StopService => {
                if self.autostart_on_stop.value() {
                    self.manager_proxy
                        .disable_unit_files(&[UNIT_NAME], false)
                        .await
                        .context("could not disable unit")?;
                }

                self.unit_proxy.stop(START_MODE_REPLACE).await?;
            }
            ServiceSetupDialogMsg::Close => {
                let client = match &self.connection_status {
                    ConnectionStatus::Connected { client, .. } => Some(client.clone()),
                    ConnectionStatus::Error(_) => None,
                };
                let _ = sender.output(client);
                return Ok(());
            }
        }
        self.reconnect().await?;

        Ok(())
    }

    async fn reconnect(&mut self) -> anyhow::Result<()> {
        let logs_handle = tokio::spawn(service_logs_text());

        let mut changed = false;

        let new_state = self
            .unit_proxy
            .active_state()
            .await
            .context("Could not update unit state")?;

        if self.service_state != new_state {
            self.service_state = new_state;
            changed = true;
        }

        let client = DaemonClient::connect_with_reconnect(false).await;

        let connection_status = ConnectionStatus::from_result(client).await;
        changed |= !self.connection_status.roughly_eq(&connection_status);
        self.connection_status = connection_status;

        let logs = logs_handle.await.unwrap();

        let current_text = self.service_logs.slice(
            &self.service_logs.start_iter(),
            &self.service_logs.end_iter(),
            true,
        );

        if logs != current_text.as_str() {
            self.service_logs.set_text(&logs);
        }

        if changed {
            self.setup_error = None;
        }

        Ok(())
    }

    fn service_version_text(&self) -> Option<String> {
        match &self.connection_status {
            ConnectionStatus::Connected { version, .. } => Some(format_version(version)),
            ConnectionStatus::Error(_) => None,
        }
    }

    fn version_mismatched(&self) -> bool {
        match &self.connection_status {
            ConnectionStatus::Connected { version, .. } => !version.is_current(),
            ConnectionStatus::Error(_) => false,
        }
    }
}

enum ConnectionStatus {
    Connected {
        client: DaemonClient,
        version: VersionInfo,
    },
    Error(Option<String>),
}

impl ConnectionStatus {
    async fn from_result(result: anyhow::Result<DaemonClient>) -> Self {
        match result {
            Ok(client) => match client.get_system_info().await {
                Ok(info) => Self::Connected {
                    client,
                    version: info.version,
                },
                Err(err) => Self::from_err(err),
            },
            Err(err) => Self::from_err(err),
        }
    }

    fn from_err(err: anyhow::Error) -> Self {
        let msg = if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
            match io_err.kind() {
                io::ErrorKind::NotFound => None,
                io::ErrorKind::PermissionDenied => Some(fl!(I18N, "service-permission-denied")),
                _ => Some(format!("{} (IO {io_err:#})", fl!(I18N, "error-heading"))),
            }
        } else {
            Some(format!("{} ({err:#})", fl!(I18N, "error-heading")))
        };
        Self::Error(msg)
    }

    fn status_text(&self) -> String {
        match self {
            Self::Connected { .. } => fl!(I18N, "service-connected"),
            Self::Error(_) => fl!(I18N, "service-disconnected"),
        }
    }

    fn error_text(&self) -> Option<&str> {
        match self {
            Self::Connected { .. } => None,
            Self::Error(msg) => msg.as_deref(),
        }
    }

    fn is_connected(&self) -> bool {
        match self {
            Self::Connected { .. } => true,
            Self::Error(_) => false,
        }
    }

    fn roughly_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                ConnectionStatus::Connected {
                    version: version_l, ..
                },
                ConnectionStatus::Connected {
                    version: version_r, ..
                },
            ) => version_l == version_r,
            (ConnectionStatus::Error(err_l), ConnectionStatus::Error(err_r)) => err_l == err_r,
            _ => false,
        }
    }
}

fn format_version(version: &VersionInfo) -> String {
    let mut text = format!("{}-{}", version.version, version.profile);

    if let Some(commit) = &version.commit {
        write!(text, " (commit {commit})").unwrap();
    }

    text
}

async fn service_logs_text() -> String {
    systemd::fetch_logs()
        .await
        .unwrap_or_else(|err| format!("Could not fetch logs: {err:#}"))
}
