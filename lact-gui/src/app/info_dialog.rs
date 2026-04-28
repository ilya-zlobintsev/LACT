use crate::I18N;
use adw::prelude::*;
use i18n_embed_fl::fl;
use relm4::{ComponentParts, ComponentSender};
use std::{collections::HashMap, sync::Arc};
use tracing::warn;

const RESPONSE_CLOSE: &str = "close";
const RESPONSE_CANCEL: &str = "cancel";
const RESPONSE_CONFIRM: &str = "confirm";

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum InfoDialogId {
    Error,
    EmbeddedDaemonInfo,
    ResetConfigConfirmation,
    VersionMismatch,
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
    pub destructive: bool,
}

impl InfoDialogData {
    pub fn error(err: Arc<anyhow::Error>) -> Self {
        Self {
            id: InfoDialogId::Error,
            heading: "Error".to_string(),
            body: format!("{err:#}"),
            stacktrace: Some(format!("{err:?}")),
            selectable_text: None,
            confirmation: None,
        }
    }

    pub fn embedded_daemon_info(err: anyhow::Error) -> Self {
        let error_text = format!("Error info: {err:#}\n\n");
        let body = format!(
            "Could not connect to daemon, running in embedded mode. \n\
            Please make sure the lactd service is running. \n\
            Using embedded mode, you will not be able to change any settings. \n\n\
            {error_text}\
            To enable the daemon, run the following command, then restart LACT:"
        );

        Self {
            id: InfoDialogId::EmbeddedDaemonInfo,
            heading: "Daemon info".to_string(),
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
                destructive: true,
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
            heading: "Version mismatch".to_string(),
            body: format!(
                "Version mismatch between GUI and daemon ({gui_version}-{gui_commit} vs \
                {daemon_version}-{daemon_commit})! If you have updated LACT, you need to restart \
                the service with:"
            ),
            stacktrace: None,
            selectable_text: Some("sudo systemctl restart lactd".to_string()),
            confirmation: None,
        }
    }
}

pub struct InfoDialog {
    parent: adw::ApplicationWindow,
    active_dialogs: HashMap<InfoDialogId, relm4::Controller<InfoDialogEntry>>,
}

#[derive(Debug)]
pub enum InfoDialogMsg {
    Show(InfoDialogData),
    Response(InfoDialogEntryResponse),
}

#[derive(Clone, Debug)]
pub enum InfoDialogOutput {
    Closed(InfoDialogId),
    Confirmed(InfoDialogId),
}

#[relm4::component(pub)]
impl relm4::Component for InfoDialog {
    type Init = adw::ApplicationWindow;
    type Input = InfoDialogMsg;
    type Output = InfoDialogOutput;
    type CommandOutput = ();

    view! {
        gtk::Box {}
    }

    fn init(
        parent: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            parent,
            active_dialogs: HashMap::new(),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            InfoDialogMsg::Show(data) => {
                if self.active_dialogs.contains_key(&data.id) {
                    return;
                }

                let id = data.id;
                let dialog = InfoDialogEntry::builder()
                    .launch((self.parent.clone(), data))
                    .forward(sender.input_sender(), InfoDialogMsg::Response);
                self.active_dialogs.insert(id, dialog);
            }
            InfoDialogMsg::Response(response) => {
                let id = response.id();
                self.active_dialogs.remove(&id);
                sender.output(response.into()).unwrap();
            }
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum InfoDialogEntryResponse {
    Closed(InfoDialogId),
    Confirmed(InfoDialogId),
}

impl InfoDialogEntryResponse {
    fn id(&self) -> InfoDialogId {
        match self {
            Self::Closed(id) | Self::Confirmed(id) => *id,
        }
    }
}

impl From<InfoDialogEntryResponse> for InfoDialogOutput {
    fn from(value: InfoDialogEntryResponse) -> Self {
        match value {
            InfoDialogEntryResponse::Closed(id) => Self::Closed(id),
            InfoDialogEntryResponse::Confirmed(id) => Self::Confirmed(id),
        }
    }
}

struct InfoDialogEntry {
    parent: adw::ApplicationWindow,
    data: InfoDialogData,
    stacktrace_buffer: gtk::TextBuffer,
}

#[relm4::component]
impl relm4::Component for InfoDialogEntry {
    type Init = (adw::ApplicationWindow, InfoDialogData);
    type Input = ();
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
                        set_cursor_visible: true,
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
        let details_buffer = gtk::TextBuffer::new(None);

        if let Some(stacktrace) = &data.stacktrace {
            warn!("{stacktrace}");
            details_buffer.set_text(stacktrace);
        }

        let model = Self {
            parent,
            data,
            stacktrace_buffer: details_buffer,
        };

        let widgets = view_output!();
        let id = model.data.id;

        match &model.data.confirmation {
            None => {
                root.add_response(RESPONSE_CLOSE, &fl!(I18N, "close"));
                root.set_close_response(RESPONSE_CLOSE);
                root.set_default_response(Some(RESPONSE_CLOSE));
            }
            Some(InfoDialogConfirmation {
                confirm_label,
                destructive,
            }) => {
                root.add_response(RESPONSE_CANCEL, &fl!(I18N, "cancel"));
                root.add_response(RESPONSE_CONFIRM, confirm_label);
                root.set_close_response(RESPONSE_CANCEL);
                root.set_default_response(Some(RESPONSE_CANCEL));
                if *destructive {
                    root.set_response_appearance(
                        RESPONSE_CONFIRM,
                        adw::ResponseAppearance::Destructive,
                    );
                }
            }
        }

        root.connect_response(None, move |_, response| {
            let output = match response {
                RESPONSE_CONFIRM => InfoDialogEntryResponse::Confirmed(id),
                _ => InfoDialogEntryResponse::Closed(id),
            };
            sender.output(output).unwrap();
        });
        root.present(Some(&model.parent));

        ComponentParts { model, widgets }
    }
}
