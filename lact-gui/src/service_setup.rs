pub mod systemd;

use std::io;
use std::time::Duration;

use crate::I18N;
use crate::service_setup::systemd::{START_MODE_REPLACE, UnitProxy};
use adw::prelude::*;
use futures::StreamExt as _;
use i18n_embed_fl::fl;
use lact_client::DaemonClient;
use relm4::{
    AsyncComponentSender, RelmWidgetExt,
    css::{ERROR, SUCCESS},
    prelude::{AsyncComponent, AsyncComponentParts},
    tokio,
};
use tracing::{debug, warn};

pub struct ServiceSetupDialog {
    current_client: anyhow::Result<DaemonClient>,
    connection_status: ConnectionStatus,
    unit_proxy: UnitProxy<'static>,

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
    ServiceState(String),
    Close,
    // Show,
}

enum ConnectionStatus {
    Connected,
    ConnectedMismatched,
    Error(String),
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
            // set_follows_content_size: true,
            set_title: "Service Setup",

            connect_closed => ServiceSetupDialogMsg::Close,

            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {},

                #[wrap(Some)]
                set_content = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 5,
                    set_margin_all: 15,

                    gtk::Label {
                        set_markup: &fl!(I18N, "service-explanation"),
                        set_wrap: true,
                        set_xalign: 0.0,
                        set_margin_bottom: 10,
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 5,
                        set_hexpand: true,

                        gtk::Label {
                            set_markup: &format!("<b>{}</b>", fl!(I18N, "service-connection-status")),
                            set_size_group: &label_size_group,
                            set_xalign: 0.0,
                            set_yalign: 0.0,
                        },

                        gtk::Label {
                            #[watch]
                            set_markup: &model.connection_status_text(),
                            #[watch]
                            set_css_classes: if model.current_client.is_ok() { &[SUCCESS] } else { &[ERROR] },
                        },
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 5,
                        set_hexpand: true,

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
                        },
                    },
                },

                add_bottom_bar = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 5,
                    set_halign: gtk::Align::Fill,
                    set_margin_horizontal: 10,
                    set_margin_bottom: 10,

                    gtk::Button {
                        set_label: &fl!(I18N, "service-start"),
                        connect_clicked => ServiceSetupDialogMsg::StartService,
                        add_css_class: "suggested-action",
                    },

                    gtk::Button {
                        set_label: &fl!(I18N, "service-stop"),
                        connect_clicked => ServiceSetupDialogMsg::StopService,
                    },

                    gtk::Button {
                        set_label: &fl!(I18N, "service-restart"),
                        connect_clicked => ServiceSetupDialogMsg::RestartService,
                    },

                    gtk::Button {
                        set_halign: gtk::Align::End,
                        set_hexpand: true,
                        set_label: &fl!(I18N, "close"),

                        connect_clicked[root] => move |_| {
                            root.close();
                        }
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
        let mut state_stream = params.unit_proxy.receive_active_state_changed().await;

        let input_sender = sender.input_sender().clone();
        relm4::spawn(async move {
            while let Some(property) = state_stream.next().await {
                match property.get().await {
                    Ok(state) => {
                        if input_sender
                            .send(ServiceSetupDialogMsg::ServiceState(state))
                            .is_err()
                        {
                            debug!("service setup dialog closed, exiting service state watcher");
                            break;
                        }
                    }
                    Err(err) => {
                        warn!("could not get service state: {err:#}");
                    }
                }
            }
        });

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

        let model = Self {
            current_client: Err(params.initial_error),
            unit_proxy: params.unit_proxy,
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
            ServiceSetupDialogMsg::ServiceState(state) => {
                self.service_state = state;
            }
            ServiceSetupDialogMsg::Close => {
                let client = self.current_client.as_ref().ok().cloned();
                let _ = sender.output(client);
                return Ok(());
            }
        }
        self.reconnect().await?;

        Ok(())
    }

    async fn reconnect(&mut self) -> anyhow::Result<()> {
        self.current_client = DaemonClient::connect().await;

        Ok(())
    }

    fn connection_status_style(&self) -> &'static str {
        match &self.current_client {
            Ok(client) => {
                let pong = client.ping()
            }
            Err(_) => ERROR,
        }
    }

    fn connection_status_text(&self) -> String {
        match &self.current_client {
            Ok(_client) => fl!(I18N, "service-connected"),
            Err(err) => {
                if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
                    match io_err.kind() {
                        io::ErrorKind::NotFound => fl!(I18N, "service-not-running"),
                        io::ErrorKind::PermissionDenied => fl!(I18N, "service-permission-denied"),
                        _ => format!("{} (IO {io_err:#})", fl!(I18N, "error-heading")),
                    }
                } else {
                    format!("{} ({err:#})", fl!(I18N, "error-heading"))
                }
            }
        }
    }
}
