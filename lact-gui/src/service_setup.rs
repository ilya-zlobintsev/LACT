pub mod systemd;

use std::io;
use std::time::Duration;

use crate::I18N;
use crate::service_setup::systemd::{START_MODE_REPLACE, UnitProxy};
use adw::prelude::*;
use anyhow::Context as _;
use i18n_embed_fl::fl;
use lact_client::DaemonClient;
use lact_schema::VersionInfo;
use relm4::css::{self, WARNING};
use relm4::{
    AsyncComponentSender, RelmWidgetExt,
    css::{ERROR, SUCCESS},
    prelude::{AsyncComponent, AsyncComponentParts},
    tokio,
};
use std::fmt::Write as _;
use tracing::debug;

pub struct ServiceSetupDialog {
    connection_status: ConnectionStatus,
    unit_proxy: UnitProxy<'static>,
    service_logs: gtk::TextBuffer,
    service_state: String,
}

pub struct ServiceSetupDialogParams {
    pub parent: gtk::ApplicationWindow,
    pub initial_error: anyhow::Error,
    pub unit_proxy: UnitProxy<'static>,
}

#[derive(Debug)]
pub enum ServiceSetupDialogMsg {
    Reconnect,
    StartService,
    RestartService,
    StopService,
    ServiceStateChanged,
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
            set_content_width: 500,
            set_title: "Service Setup",

            connect_closed => ServiceSetupDialogMsg::Close,

            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {},

                #[wrap(Some)]
                set_content = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 5,
                    set_margin_horizontal: 15,
                    set_margin_vertical: 5,

                    gtk::Label {
                        set_markup: &fl!(I18N, "service-explanation"),
                        set_wrap: true,
                        set_xalign: 0.0,
                        set_margin_all: 10,
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 5,
                        set_hexpand: true,
                        set_margin_horizontal: 10,

                        gtk::Label {
                            set_markup: &format!("<b>{}</b>", fl!(I18N, "service-connection-status")),
                            set_size_group: &label_size_group,
                            set_xalign: 0.0,
                            set_yalign: 0.0,
                        },

                        gtk::Label {
                            #[watch]
                            set_markup: &match &model.connection_status {
                                ConnectionStatus::Connected {..} => fl!(I18N, "service-connected"),
                                ConnectionStatus::Error(msg) => msg.clone(),
                            },
                            #[watch]
                            set_css_classes: if model.connection_status.is_connected() { &[SUCCESS] } else { &[ERROR] },
                            set_selectable: true,
                            set_hexpand: true,
                            set_halign: gtk::Align::End,
                        },
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 5,
                        set_hexpand: true,
                        set_margin_horizontal: 10,

                        gtk::Label {
                            set_markup: &format!("<b>{}</b>", fl!(I18N, "service-status")),
                            set_size_group: &label_size_group,
                            set_xalign: 0.0,
                            set_yalign: 0.0,
                        },

                        gtk::Label {
                            #[watch]
                            set_markup: &format!("<tt>{}</tt>", model.service_state),
                            set_wrap: true,
                            set_hexpand: true,
                            set_halign: gtk::Align::End,
                        },

                        gtk::MenuButton {
                            set_icon_name: "utilities-terminal-symbolic",
                            add_css_class: css::FLAT,
                            set_valign: gtk::Align::Center,

                            #[wrap(Some)]
                            set_popover = &gtk::Popover {
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 5,
                                    set_margin_all: 10,

                                    gtk::Label {
                                        set_label: &fl!(I18N, "service-logs"),
                                    },

                                    gtk::ScrolledWindow {
                                        set_min_content_width: 650,
                                        set_min_content_height: 250,

                                        gtk::TextView {
                                            set_editable: false,
                                            set_buffer: Some(&model.service_logs),
                                            set_top_margin: 5,
                                            set_bottom_margin: 5,
                                            set_left_margin: 5,
                                            set_right_margin: 5,
                                        },
                                    },
                                },

                                set_position: gtk::PositionType::Top,
                            },
                        }
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 5,
                        set_hexpand: true,
                        set_margin_horizontal: 10,

                        gtk::Label {
                            set_markup: &format!("<b>{}</b>", fl!(I18N, "service-version")),
                            set_size_group: &label_size_group,
                            set_xalign: 0.0,
                            set_yalign: 0.0,
                        },

                        gtk::Label {
                            #[watch]
                            set_markup: &model.service_version_text().unwrap_or_else(|| fl!(I18N, "missing-stat")),
                            #[watch]
                            set_css_classes: match &model.connection_status {
                                ConnectionStatus::Connected { version, .. } => {
                                    if version.is_current() {
                                        &[SUCCESS]
                                    } else {
                                        &[WARNING]
                                    }
                                }
                                ConnectionStatus::Error(_) => &[],
                            },
                            set_wrap: true,
                            set_hexpand: true,
                            set_halign: gtk::Align::End,
                        },
                    },
                },

                add_bottom_bar = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 5,
                    set_halign: gtk::Align::End,
                    set_margin_horizontal: 25,
                    set_margin_vertical: 20,

                    gtk::Button {
                        set_label: &fl!(I18N, "service-start"),
                        connect_clicked => ServiceSetupDialogMsg::StartService,
                        add_css_class: "suggested-action",
                        #[watch]
                        set_visible: model.service_state != systemd::UNIT_STATE_ACTIVE,
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
                tokio::time::sleep(Duration::from_millis(250)).await;
                if input_sender.send(ServiceSetupDialogMsg::Reconnect).is_err() {
                    debug!("service setup dialog closed, exiting client watcher");
                    break;
                }
            }
        });

        let service_state = params
            .unit_proxy
            .active_state()
            .await
            .unwrap_or_else(|err| {
                // TODO: show error, APP_BROKER does not work yet because app is not initialized
                // APP_BROKER.send(AppMsg::Error(Arc::new(anyhow!("systemd error: {err:#}"))));
                panic!("{err:#}");
            });

        let service_logs = service_logs_text().await;

        let model = Self {
            connection_status: ConnectionStatus::from_err(params.initial_error),
            unit_proxy: params.unit_proxy,
            service_logs: gtk::TextBuffer::builder().text(service_logs).build(),
            service_state,
        };

        let label_size_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);

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
            // TODO
            panic!("{err:#}");
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
                self.unit_proxy.start(START_MODE_REPLACE).await?;
            }
            ServiceSetupDialogMsg::RestartService => {
                self.unit_proxy.restart(START_MODE_REPLACE).await?;
            }
            ServiceSetupDialogMsg::StopService => {
                self.unit_proxy.stop(START_MODE_REPLACE).await?;
            }
            ServiceSetupDialogMsg::ServiceStateChanged => {}
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

        self.service_state = self
            .unit_proxy
            .active_state()
            .await
            .context("Could not update unit state")?;

        let client = DaemonClient::connect_with_reconnect(false).await;
        self.connection_status = ConnectionStatus::from_result(client).await;

        let logs = logs_handle.await.unwrap();

        let current_text = self.service_logs.slice(
            &self.service_logs.start_iter(),
            &self.service_logs.end_iter(),
            true,
        );

        if logs != current_text.as_str() {
            self.service_logs.set_text(&logs);
        }

        Ok(())
    }

    fn service_version_text(&self) -> Option<String> {
        match &self.connection_status {
            ConnectionStatus::Connected { version, .. } => {
                Some(format!("<tt>{}</tt>", format_version(version)))
            }
            ConnectionStatus::Error(_) => None,
        }
    }
}

enum ConnectionStatus {
    Connected {
        client: DaemonClient,
        version: VersionInfo,
    },
    Error(String),
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
                io::ErrorKind::NotFound => fl!(I18N, "service-not-running"),
                io::ErrorKind::PermissionDenied => fl!(I18N, "service-permission-denied"),
                _ => format!("{} (IO {io_err:#})", fl!(I18N, "error-heading")),
            }
        } else {
            format!("{} ({err:#})", fl!(I18N, "error-heading"))
        };
        Self::Error(msg)
    }

    fn is_connected(&self) -> bool {
        match self {
            Self::Connected { .. } => true,
            Self::Error(_) => false,
        }
    }
}

fn format_version(version: &VersionInfo) -> String {
    format!("{}-{}", version.version, version.profile)
}

async fn service_logs_text() -> String {
    systemd::fetch_logs()
        .await
        .unwrap_or_else(|err| format!("Could not fetch logs: {err:#}"))
}
