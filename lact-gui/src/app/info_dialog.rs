use crate::I18N;
use adw::prelude::*;
use gtk::glib::{self, ControlFlow, clone};
use i18n_embed_fl::fl;
use relm4::{ComponentParts, ComponentSender};
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

#[derive(Clone)]
pub struct InfoDialogData {
    pub id: InfoDialogId,
    pub heading: String,
    pub body: Arc<dyn Fn(u64) -> String + Send + Sync>,
    pub stacktrace: Option<String>,
    pub selectable_text: Option<String>,
    pub confirmation: Option<InfoDialogConfirmation>,
}

impl InfoDialogData {
    fn body_text(&self, seconds_left: u64) -> String {
        (self.body)(seconds_left)
    }
}

impl fmt::Debug for InfoDialogData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InfoDialogData")
            .field("id", &self.id)
            .field("heading", &self.heading)
            .field("body", &"<body callback>")
            .field("stacktrace", &self.stacktrace)
            .field("selectable_text", &self.selectable_text)
            .field("confirmation", &self.confirmation)
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct InfoDialogConfirmation {
    pub confirm_label: String,
    pub cancel_label: String,
    pub appearance: adw::ResponseAppearance,
    pub timeout_seconds: Option<u64>,
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
            Self::Response(resp) => f.debug_tuple("Response").field(resp).finish(),
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
                self.show_entry(data, None, &sender);
            }
            InfoDialogMsg::ShowConfirmation(data, callback) => {
                self.show_entry(
                    data,
                    Some(InfoDialogCallbacks::confirmed(callback)),
                    &sender,
                );
            }
            InfoDialogMsg::ShowTimedConfirmation {
                data,
                on_confirmed,
                on_closed,
                on_timed_out,
            } => {
                self.show_entry(
                    data,
                    Some(InfoDialogCallbacks {
                        on_confirmed: Some(on_confirmed),
                        on_closed: Some(on_closed),
                        on_timed_out: Some(on_timed_out),
                    }),
                    &sender,
                );
            }
            InfoDialogMsg::Response(response) => {
                self.active_dialogs.remove(&response.id());
            }
        }
    }
}

impl InfoDialog {
    fn show_entry(
        &mut self,
        data: InfoDialogData,
        callbacks: Option<InfoDialogCallbacks>,
        sender: &ComponentSender<Self>,
    ) {
        if self.active_dialogs.contains_key(&data.id) {
            return;
        }

        let id = data.id;
        let mut callbacks = callbacks;

        let dialog = <InfoDialogEntry as relm4::Component>::builder()
            .launch((self.parent.clone(), data))
            .connect_receiver({
                let sender = sender.clone();
                move |_, response| {
                    let confirmed = matches!(response, InfoDialogEntryResponse::Confirmed(_));
                    let closed = matches!(response, InfoDialogEntryResponse::Closed(_));
                    let timed_out = matches!(response, InfoDialogEntryResponse::TimedOut(_));
                    sender.input(InfoDialogMsg::Response(response));
                    if let Some(callbacks) = callbacks.as_mut() {
                        if confirmed && let Some(callback) = callbacks.on_confirmed.take() {
                            callback();
                        } else if closed && let Some(callback) = callbacks.on_closed.take() {
                            callback();
                        } else if timed_out && let Some(callback) = callbacks.on_timed_out.take() {
                            callback();
                        }
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
    completed: Rc<Cell<bool>>,
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
                    set_label: &model.data.body_text(model.seconds_left.unwrap_or_default()),
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
            completed: Rc::new(Cell::new(false)),
            seconds_left,
        };

        let widgets = view_output!();
        let completed = model.completed.clone();

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

                root.set_response_appearance(RESPONSE_CONFIRM, confirmation.appearance);

                if let Some(mut remaining) = seconds_left.filter(|seconds_left| *seconds_left > 0) {
                    glib::source::timeout_add_local(
                        Duration::from_secs(1),
                        clone!(
                            #[strong]
                            sender,
                            #[strong]
                            completed,
                            move || {
                                if completed.get() {
                                    return ControlFlow::Break;
                                }

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
        if self.completed.get() {
            return;
        }

        let id = self.data.id;

        match msg {
            InfoDialogEntryMsg::Tick => {
                if let Some(seconds_left) = &mut self.seconds_left {
                    *seconds_left = seconds_left.saturating_sub(1);
                    let secs = *seconds_left;
                    widgets.body_label.set_label(&self.data.body_text(secs));
                    if secs == 0 {
                        self.completed.set(true);
                        let _ = sender.output(InfoDialogEntryResponse::TimedOut(id));
                        root.force_close();
                    }
                }
            }
            InfoDialogEntryMsg::Response(response) => {
                self.completed.set(true);
                let output = match response.as_str() {
                    RESPONSE_CONFIRM => InfoDialogEntryResponse::Confirmed(id),
                    _ => InfoDialogEntryResponse::Closed(id),
                };
                let _ = sender.output(output);
            }
        }
    }
}
