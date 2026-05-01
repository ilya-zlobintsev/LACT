use crate::I18N;
use adw::prelude::*;
use gtk::glib::{self, ControlFlow, clone};
use i18n_embed_fl::fl;
use lact_client::DaemonClient;
use lact_schema::request::ConfirmCommand;
use relm4::{ComponentParts, ComponentSender};
use std::{collections::HashMap, fmt, sync::Arc, time::Duration};
use tracing::warn;

const RESPONSE_CLOSE: &str = "close";
const RESPONSE_CANCEL: &str = "cancel";
const RESPONSE_CONFIRM: &str = "confirm";

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum InfoDialogId {
    Error,
    EmbeddedDaemonInfo,
    ResetConfigConfirmation,
    SettingsConfirmation,
    VersionMismatch,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum ConfirmAppearance {
    #[default]
    Default,
    Destructive,
    Suggested,
}

#[derive(Clone, Debug)]
pub struct InfoDialogData {
    pub id: InfoDialogId,
    pub heading: String,
    pub body: String,
    pub stacktrace: Option<String>,
    pub selectable_text: Option<String>,
    pub confirmation: Option<InfoDialogConfirmation>,
}

#[derive(Clone, Debug)]
pub struct InfoDialogConfirmation {
    pub confirm_label: String,
    pub cancel_label: String,
    pub appearance: ConfirmAppearance,
    pub timeout_seconds: Option<u64>,
}

impl InfoDialogData {
    pub fn error(err: Arc<anyhow::Error>) -> Self {
        Self {
            id: InfoDialogId::Error,
            heading: fl!(I18N, "error-heading"),
            body: format!("{err:#}"),
            stacktrace: Some(format!("{err:?}")),
            selectable_text: None,
            confirmation: None,
        }
    }

    pub fn embedded_daemon_info(err: anyhow::Error) -> Self {
        let error_text = format!("Error info: {err:#}\n\n");
        let body = fl!(I18N, "embedded-daemon-info", error_info = error_text);

        Self {
            id: InfoDialogId::EmbeddedDaemonInfo,
            heading: fl!(I18N, "daemon-info-heading"),
            body,
            stacktrace: None,
            selectable_text: Some("sudo systemctl enable --now lactd".to_string()),
            confirmation: None,
        }
    }

    pub fn reset_config_confirmation(heading: String, body: String, confirm_label: String) -> Self {
        Self {
            id: InfoDialogId::ResetConfigConfirmation,
            heading,
            body,
            stacktrace: None,
            selectable_text: None,
            confirmation: Some(InfoDialogConfirmation {
                confirm_label,
                cancel_label: fl!(I18N, "cancel"),
                appearance: ConfirmAppearance::Destructive,
                timeout_seconds: None,
            }),
        }
    }

    pub fn settings_confirmation(delay: u64) -> Self {
        Self {
            id: InfoDialogId::SettingsConfirmation,
            heading: fl!(I18N, "confirm-settings"),
            body: fl!(I18N, "settings-confirmation", seconds_left = delay),
            stacktrace: None,
            selectable_text: None,
            confirmation: Some(InfoDialogConfirmation {
                confirm_label: fl!(I18N, "confirm"),
                cancel_label: fl!(I18N, "revert-button"),
                appearance: ConfirmAppearance::Suggested,
                timeout_seconds: Some(delay),
            }),
        }
    }

    pub fn version_mismatch(
        gui_version: &str,
        gui_commit: &str,
        daemon_version: &str,
        daemon_commit: &str,
    ) -> Self {
        Self {
            id: InfoDialogId::VersionMismatch,
            heading: fl!(I18N, "version-mismatch"),
            body: fl!(
                I18N,
                "version-mismatch-description",
                gui_version = gui_version,
                gui_commit = gui_commit,
                daemon_version = daemon_version,
                daemon_commit = daemon_commit
            ),
            stacktrace: None,
            selectable_text: Some("sudo systemctl restart lactd".to_string()),
            confirmation: None,
        }
    }
}

pub struct InfoDialog {
    parent: adw::ApplicationWindow,
    daemon_client: DaemonClient,
    active_dialogs: HashMap<InfoDialogId, relm4::Controller<InfoDialogEntry>>,
    pending_callback: Option<Box<dyn FnOnce() + 'static>>,
}

pub enum InfoDialogMsg {
    Show(InfoDialogData),
    ShowConfirmation(InfoDialogData, Box<dyn FnOnce() + 'static>),
    Response(InfoDialogEntryResponse),
}

impl fmt::Debug for InfoDialogMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Show(data) => f.debug_tuple("Show").field(data).finish(),
            Self::ShowConfirmation(data, _) => {
                f.debug_tuple("ShowConfirmation").field(data).finish()
            }
            Self::Response(resp) => f.debug_tuple("Response").field(resp).finish(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum InfoDialogOutput {
    ReloadData,
    Error(Arc<anyhow::Error>),
}

#[relm4::component(pub)]
impl relm4::Component for InfoDialog {
    type Init = (adw::ApplicationWindow, DaemonClient);
    type Input = InfoDialogMsg;
    type Output = InfoDialogOutput;
    type CommandOutput = ();

    view! {
        gtk::Box {}
    }

    fn init(
        (parent, daemon_client): Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            parent,
            daemon_client,
            active_dialogs: HashMap::new(),
            pending_callback: None,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            InfoDialogMsg::Show(data) => {
                self.show_entry(data, &sender);
            }
            InfoDialogMsg::ShowConfirmation(data, callback) => {
                self.pending_callback = Some(callback);
                self.show_entry(data, &sender);
            }
            InfoDialogMsg::Response(response) => {
                self.active_dialogs.remove(&response.id());

                match response {
                    InfoDialogEntryResponse::Confirmed(InfoDialogId::ResetConfigConfirmation) => {
                        if let Some(callback) = self.pending_callback.take() {
                            callback();
                        }
                    }
                    InfoDialogEntryResponse::Confirmed(InfoDialogId::SettingsConfirmation) => {
                        self.confirm_pending_config(ConfirmCommand::Confirm, &sender);
                    }
                    InfoDialogEntryResponse::Closed(InfoDialogId::SettingsConfirmation) => {
                        self.confirm_pending_config(ConfirmCommand::Revert, &sender);
                    }
                    InfoDialogEntryResponse::TimedOut(InfoDialogId::SettingsConfirmation) => {
                        let _ = sender.output(InfoDialogOutput::ReloadData);
                    }
                    _ => {}
                }
            }
        }
    }
}

impl InfoDialog {
    fn show_entry(&mut self, data: InfoDialogData, sender: &ComponentSender<Self>) {
        if self.active_dialogs.contains_key(&data.id) {
            return;
        }

        let id = data.id;
        let dialog = <InfoDialogEntry as relm4::Component>::builder()
            .launch((self.parent.clone(), data))
            .forward(sender.input_sender(), InfoDialogMsg::Response);
        self.active_dialogs.insert(id, dialog);
    }

    fn confirm_pending_config(&self, command: ConfirmCommand, sender: &ComponentSender<Self>) {
        let daemon_client = self.daemon_client.clone();
        let output_sender = sender.output_sender().clone();
        relm4::spawn_local(async move {
            if let Err(err) = daemon_client.confirm_pending_config(command).await {
                let _ = output_sender.send(InfoDialogOutput::Error(Arc::new(err)));
            }
            let _ = output_sender.send(InfoDialogOutput::ReloadData);
        });
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum InfoDialogEntryResponse {
    Closed(InfoDialogId),
    Confirmed(InfoDialogId),
    TimedOut(InfoDialogId),
}

impl InfoDialogEntryResponse {
    fn id(&self) -> InfoDialogId {
        match self {
            Self::Closed(id) | Self::Confirmed(id) | Self::TimedOut(id) => *id,
        }
    }
}

struct InfoDialogEntry {
    data: InfoDialogData,
    stacktrace_buffer: gtk::TextBuffer,
    completed: bool,
    seconds_left: Option<u64>,
}

#[derive(Debug)]
enum InfoDialogEntryMsg {
    Tick,
    Response(String),
}

#[relm4::component]
impl relm4::Component for InfoDialogEntry {
    type Init = (adw::ApplicationWindow, InfoDialogData);
    type Input = InfoDialogEntryMsg;
    type Output = InfoDialogEntryResponse;
    type CommandOutput = ();

    view! {
        adw::AlertDialog {
            set_heading: Some(&model.data.heading),
            #[wrap(Some)]
            set_extra_child = &gtk::Box {
                set_width_request: 500,
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 10,

                #[name = "body_label"]
                gtk::Label {
                    set_label: &model.data.body,
                    set_wrap: true,
                    set_xalign: 0.0,
                },

                gtk::Entry {
                    set_visible: model.data.selectable_text.is_some(),
                    set_text: model.data.selectable_text.as_deref().unwrap_or_default(),
                    set_editable: false,
                },

                gtk::ScrolledWindow {
                    set_visible: cfg!(debug_assertions) && model.data.stacktrace.is_some(),
                    set_min_content_width: 600,
                    set_min_content_height: 240,
                    set_max_content_height: 360,
                    set_hscrollbar_policy: gtk::PolicyType::Automatic,
                    set_vscrollbar_policy: gtk::PolicyType::Automatic,

                    gtk::TextView {
                        set_buffer: Some(&model.stacktrace_buffer),
                        set_editable: false,
                        set_monospace: true,
                        set_wrap_mode: gtk::WrapMode::None,
                    }
                }
            }
        }
    }

    fn init(
        (parent, data): Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let stacktrace_buffer = gtk::TextBuffer::new(None);

        if let Some(stacktrace) = &data.stacktrace {
            warn!("{stacktrace}");
            stacktrace_buffer.set_text(stacktrace);
        }

        let seconds_left = data.confirmation.as_ref().and_then(|c| c.timeout_seconds);

        let model = Self {
            data,
            stacktrace_buffer,
            completed: false,
            seconds_left,
        };

        let widgets = view_output!();

        match &model.data.confirmation {
            None => {
                root.add_response(RESPONSE_CLOSE, &fl!(I18N, "close"));
                root.set_close_response(RESPONSE_CLOSE);
                root.set_default_response(Some(RESPONSE_CLOSE));
            }
            Some(confirmation) => {
                root.add_response(RESPONSE_CANCEL, &confirmation.cancel_label);
                root.add_response(RESPONSE_CONFIRM, &confirmation.confirm_label);
                root.set_close_response(RESPONSE_CANCEL);
                root.set_default_response(Some(RESPONSE_CANCEL));

                match confirmation.appearance {
                    ConfirmAppearance::Destructive => {
                        root.set_response_appearance(
                            RESPONSE_CONFIRM,
                            adw::ResponseAppearance::Destructive,
                        );
                    }
                    ConfirmAppearance::Suggested => {
                        root.set_response_appearance(
                            RESPONSE_CONFIRM,
                            adw::ResponseAppearance::Suggested,
                        );
                    }
                    ConfirmAppearance::Default => {}
                }

                if let Some(mut remaining) = seconds_left {
                    glib::source::timeout_add_local(
                        Duration::from_secs(1),
                        clone!(
                            #[strong]
                            sender,
                            move || {
                                remaining -= 1;
                                sender.input(InfoDialogEntryMsg::Tick);
                                if remaining == 0 {
                                    ControlFlow::Break
                                } else {
                                    ControlFlow::Continue
                                }
                            }
                        ),
                    );
                }
            }
        }

        root.connect_response(
            None,
            clone!(
                #[strong]
                sender,
                move |_, response| {
                    sender.input(InfoDialogEntryMsg::Response(response.to_owned()));
                }
            ),
        );
        root.present(Some(&parent));

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        if self.completed {
            return;
        }

        let id = self.data.id;

        match msg {
            InfoDialogEntryMsg::Tick => {
                if let Some(seconds_left) = &mut self.seconds_left {
                    *seconds_left -= 1;
                    let secs = *seconds_left;
                    widgets.body_label.set_label(&fl!(
                        I18N,
                        "settings-confirmation",
                        seconds_left = secs
                    ));
                    if secs == 0 {
                        self.completed = true;
                        let _ = sender.output(InfoDialogEntryResponse::TimedOut(id));
                        root.force_close();
                    }
                }
            }
            InfoDialogEntryMsg::Response(response) => {
                self.completed = true;
                let output = match response.as_str() {
                    RESPONSE_CONFIRM => InfoDialogEntryResponse::Confirmed(id),
                    _ => InfoDialogEntryResponse::Closed(id),
                };
                let _ = sender.output(output);
            }
        }
    }
}
