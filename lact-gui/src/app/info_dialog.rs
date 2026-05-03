use crate::I18N;
use adw::prelude::*;
use gtk::glib::clone;
use i18n_embed_fl::fl;
use relm4::{ComponentParts, ComponentSender};
use std::{collections::HashMap, fmt};
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

#[derive(Clone)]
pub struct InfoDialogData {
    pub id: InfoDialogId,
    pub heading: String,
    pub body: String,
    pub stacktrace: Option<String>,
    pub selectable_text: Option<String>,
    pub confirmation: Option<InfoDialogConfirmation>,
}

impl fmt::Debug for InfoDialogData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InfoDialogData")
            .field("id", &self.id)
            .field("heading", &self.heading)
            .field("body", &self.body)
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
}

pub struct InfoDialog {
    parent: adw::ApplicationWindow,
    active_dialogs: HashMap<InfoDialogId, relm4::Controller<InfoDialogEntry>>,
}

pub enum InfoDialogMsg {
    Show(Box<InfoDialogRequest>),
    Response(InfoDialogEntryResponse),
}

pub struct InfoDialogRequest {
    pub data: InfoDialogData,
    pub callbacks: InfoDialogCallbacks,
}

impl InfoDialogRequest {
    pub fn new(data: InfoDialogData) -> Self {
        Self {
            data,
            callbacks: InfoDialogCallbacks::default(),
        }
    }

    pub fn confirmed(data: InfoDialogData, on_confirmed: Box<dyn FnOnce() + 'static>) -> Self {
        Self {
            data,
            callbacks: InfoDialogCallbacks::confirmed(on_confirmed),
        }
    }
}

impl fmt::Debug for InfoDialogMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Show(request) => f.debug_tuple("Show").field(&request.data).finish(),
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
            InfoDialogMsg::Show(request) => {
                let InfoDialogRequest { data, callbacks } = *request;
                self.show_entry(data, callbacks, &sender);
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
        callbacks: InfoDialogCallbacks,
        sender: &ComponentSender<Self>,
    ) {
        if self.active_dialogs.contains_key(&data.id) {
            return;
        }

        let id = data.id;

        let dialog = <InfoDialogEntry as relm4::Component>::builder()
            .launch((self.parent.clone(), data))
            .connect_receiver({
                let sender = sender.clone();
                let mut callbacks = callbacks;
                move |_, response| {
                    let confirmed = matches!(response, InfoDialogEntryResponse::Confirmed(_));
                    let closed = matches!(response, InfoDialogEntryResponse::Closed(_));
                    sender.input(InfoDialogMsg::Response(response));
                    if confirmed && let Some(callback) = callbacks.on_confirmed.take() {
                        callback();
                    } else if closed && let Some(callback) = callbacks.on_closed.take() {
                        callback();
                    }
                }
            });
        self.active_dialogs.insert(id, dialog);
    }
}

#[derive(Default)]
pub struct InfoDialogCallbacks {
    pub on_confirmed: Option<Box<dyn FnOnce() + 'static>>,
    pub on_closed: Option<Box<dyn FnOnce() + 'static>>,
}

impl InfoDialogCallbacks {
    pub fn confirmed(on_confirmed: Box<dyn FnOnce() + 'static>) -> Self {
        Self {
            on_confirmed: Some(on_confirmed),
            on_closed: None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum InfoDialogEntryResponse {
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

struct InfoDialogEntry {
    data: InfoDialogData,
    stacktrace_buffer: gtk::TextBuffer,
}

#[derive(Debug)]
enum InfoDialogEntryMsg {
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

        let model = Self {
            data,
            stacktrace_buffer,
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

                root.set_response_appearance(RESPONSE_CONFIRM, confirmation.appearance);
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
        _widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        let id = self.data.id;

        match msg {
            InfoDialogEntryMsg::Response(response) => {
                let output = match response.as_str() {
                    RESPONSE_CONFIRM => InfoDialogEntryResponse::Confirmed(id),
                    _ => InfoDialogEntryResponse::Closed(id),
                };
                let _ = sender.output(output);
            }
        }
    }
}
