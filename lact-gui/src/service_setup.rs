pub mod systemd;

use std::time::Duration;

use crate::service_setup::systemd::{START_MODE_REPLACE, UnitProxy};
use adw::prelude::*;
use anyhow::{Context as _, anyhow};
use futures::StreamExt as _;
use lact_client::DaemonClient;
use relm4::{
    AsyncComponentSender, RelmWidgetExt,
    prelude::{AsyncComponent, AsyncComponentParts},
    tokio,
};
use tracing::{debug, warn};

pub struct ServiceSetupDialog {
    current_client: anyhow::Result<DaemonClient>,
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

#[relm4::component(pub, async)]
impl AsyncComponent for ServiceSetupDialog {
    type Init = ServiceSetupDialogParams;
    type Input = ServiceSetupDialogMsg;
    type Output = Option<DaemonClient>;
    type CommandOutput = ();

    view! {
        adw::Dialog {
            set_content_width: 500,
            set_follows_content_size: true,
            set_title: "Service Setup",

            connect_closed => ServiceSetupDialogMsg::Close,

            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {},

                #[wrap(Some)]
                set_content = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 10,
                    set_margin_all: 10,

                    gtk::Label {
                        #[watch]
                        set_markup: &format!("Service Status: <tt>{}</tt>", model.service_state),
                    },

                    gtk::Label {
                        #[watch]
                        set_markup: &format!("Connection ok: <tt>{}</tt>", model.current_client.is_ok()),
                    },

                    gtk::Button {
                        set_label: "Start Service",
                        connect_clicked => ServiceSetupDialogMsg::StartService,
                    },

                    gtk::Button {
                        set_label: "Stop Service",
                        connect_clicked => ServiceSetupDialogMsg::StopService,
                    },

                    gtk::Button {
                        set_label: "Restart Service",
                        connect_clicked => ServiceSetupDialogMsg::RestartService,
                    },
                },

                add_bottom_bar = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 10,
                    set_halign: gtk::Align::Fill,
                    set_margin_horizontal: 10,
                    set_margin_bottom: 10,

                    gtk::Button {
                        set_halign: gtk::Align::End,
                        set_hexpand: true,
                        set_label: "Close",

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
}
