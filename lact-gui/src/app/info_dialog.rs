use crate::I18N;
use adw::prelude::*;
use gtk::glib::{self, ControlFlow, clone};
use i18n_embed_fl::fl;
use relm4::{Component, ComponentParts, ComponentSender};
use std::{cell::Cell, collections::HashMap, fmt, rc::Rc, sync::Arc, time::Duration};
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
    pub destructive: bool,
    pub suggested: bool,
    pub timeout_seconds: Option<u64>,
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
        let body = fl!(I18N, "embedded-daemon-info", error_info = error_text);

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
                cancel_label: fl!(I18N, "cancel"),
                destructive: true,
                suggested: false,
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
                destructive: false,
                suggested: true,
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
    active_dialogs: HashMap<InfoDialogId, relm4::Controller<InfoDialogEntry>>,
}

pub enum InfoDialogMsg {
    Show(InfoDialogData),
    ShowConfirmation(InfoDialogData, Box<dyn FnOnce() + 'static>),
    ShowTimedConfirmation {
        data: InfoDialogData,
        on_confirmed: Box<dyn FnOnce() + 'static>,
        on_closed: Box<dyn FnOnce() + 'static>,
        on_timed_out: Box<dyn FnOnce() + 'static>,
    },
    Response(InfoDialogEntryResponse),
}

impl fmt::Debug for InfoDialogMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Show(data) => f.debug_tuple("Show").field(data).finish(),
            Self::ShowConfirmation(data, _) => {
                f.debug_tuple("ShowConfirmation").field(data).finish()
            }
            Self::ShowTimedConfirmation { data, .. } => {
                f.debug_tuple("ShowTimedConfirmation").field(data).finish()
            }
            Self::Response(response) => f.debug_tuple("Response").field(response).finish(),
        }
    }
}

#[relm4::component(pub)]
impl relm4::Component for InfoDialog {
    type Init = adw::ApplicationWindow;
    type Input = InfoDialogMsg;
    type Output = ();
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
                self.show(data, None, sender);
            }
            InfoDialogMsg::ShowConfirmation(data, on_confirmed) => {
                self.show(
                    data,
                    Some(InfoDialogCallbacks::confirmed(on_confirmed)),
                    sender,
                );
            }
            InfoDialogMsg::ShowTimedConfirmation {
                data,
                on_confirmed,
                on_closed,
                on_timed_out,
            } => {
                self.show(
                    data,
                    Some(InfoDialogCallbacks {
                        on_confirmed: Some(on_confirmed),
                        on_closed: Some(on_closed),
                        on_timed_out: Some(on_timed_out),
                    }),
                    sender,
                );
            }
            InfoDialogMsg::Response(response) => {
                self.active_dialogs.remove(&response.id());
            }
        }
    }
}

impl InfoDialog {
    fn show(
        &mut self,
        data: InfoDialogData,
        callbacks: Option<InfoDialogCallbacks>,
        sender: ComponentSender<Self>,
    ) {
        if self.active_dialogs.contains_key(&data.id) {
            return;
        }

        let id = data.id;
        let mut callbacks = callbacks;
        let dialog = InfoDialogEntry::builder()
            .launch((self.parent.clone(), data))
            .connect_receiver(move |_, response| {
                let confirmed = matches!(response, InfoDialogEntryResponse::Confirmed(_));
                let closed = matches!(response, InfoDialogEntryResponse::Closed(_));
                let timed_out = matches!(response, InfoDialogEntryResponse::TimedOut(_));
                sender.input(InfoDialogMsg::Response(response));
                if let Some(callbacks) = callbacks.as_mut() {
                    if confirmed && let Some(on_confirmed) = callbacks.on_confirmed.take() {
                        on_confirmed();
                    } else if closed && let Some(on_closed) = callbacks.on_closed.take() {
                        on_closed();
                    } else if timed_out && let Some(on_timed_out) = callbacks.on_timed_out.take() {
                        on_timed_out();
                    }
                }
            });
        self.active_dialogs.insert(id, dialog);
    }
}

struct InfoDialogCallbacks {
    on_confirmed: Option<Box<dyn FnOnce() + 'static>>,
    on_closed: Option<Box<dyn FnOnce() + 'static>>,
    on_timed_out: Option<Box<dyn FnOnce() + 'static>>,
}

impl InfoDialogCallbacks {
    fn confirmed(on_confirmed: Box<dyn FnOnce() + 'static>) -> Self {
        Self {
            on_confirmed: Some(on_confirmed),
            on_closed: None,
            on_timed_out: None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum InfoDialogEntryResponse {
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
    parent: adw::ApplicationWindow,
    data: InfoDialogData,
    stacktrace_buffer: gtk::TextBuffer,
    completed: Rc<Cell<bool>>,
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
            completed: Rc::new(Cell::new(false)),
        };

        let widgets = view_output!();
        let id = model.data.id;
        let completed = model.completed.clone();

        match &model.data.confirmation {
            None => {
                root.add_response(RESPONSE_CLOSE, &fl!(I18N, "close"));
                root.set_close_response(RESPONSE_CLOSE);
                root.set_default_response(Some(RESPONSE_CLOSE));
            }
            Some(InfoDialogConfirmation {
                confirm_label,
                cancel_label,
                destructive,
                suggested,
                timeout_seconds,
            }) => {
                root.add_response(RESPONSE_CANCEL, cancel_label);
                root.add_response(RESPONSE_CONFIRM, confirm_label);
                root.set_close_response(RESPONSE_CANCEL);
                root.set_default_response(Some(RESPONSE_CANCEL));
                if *destructive {
                    root.set_response_appearance(
                        RESPONSE_CONFIRM,
                        adw::ResponseAppearance::Destructive,
                    );
                } else if *suggested {
                    root.set_response_appearance(
                        RESPONSE_CONFIRM,
                        adw::ResponseAppearance::Suggested,
                    );
                }

                if let Some(mut seconds_left) = *timeout_seconds {
                    glib::source::timeout_add_local(
                        Duration::from_secs(1),
                        clone!(
                            #[strong]
                            root,
                            #[strong]
                            sender,
                            #[strong]
                            completed,
                            #[strong(rename_to = body_label)]
                            widgets.body_label,
                            move || {
                                seconds_left -= 1;
                                body_label.set_label(&fl!(
                                    I18N,
                                    "settings-confirmation",
                                    seconds_left = seconds_left
                                ));

                                if seconds_left == 0 {
                                    completed.set(true);
                                    sender
                                        .output(InfoDialogEntryResponse::TimedOut(id))
                                        .unwrap();
                                    root.force_close();
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

        root.connect_response(None, move |_, response| {
            if completed.replace(true) {
                return;
            }

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
