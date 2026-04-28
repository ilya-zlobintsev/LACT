use adw::prelude::*;
use relm4::{ComponentParts, ComponentSender};
use std::sync::Arc;
use tracing::warn;

const RESPONSE_CLOSE: &str = "close";
const RESPONSE_COPY_CLOSE: &str = "copy-close";

pub struct ErrorDialog {
    parent: adw::ApplicationWindow,
    summary: String,
    details: String,
    details_buffer: gtk::TextBuffer,
}

#[derive(Debug)]
pub enum ErrorDialogMsg {
    Show(Arc<anyhow::Error>),
    CopyAndClose,
}

#[relm4::component(pub)]
impl relm4::Component for ErrorDialog {
    type Init = adw::ApplicationWindow;
    type Input = ErrorDialogMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        adw::AlertDialog {
            set_heading: Some("Error"),
            #[watch]
            set_body: &model.summary,
            set_close_response: RESPONSE_CLOSE,
            set_default_response: Some(RESPONSE_CLOSE),
            set_content_width: 700,

            add_response: (RESPONSE_CLOSE, "Close"),

            #[wrap(Some)]
            set_extra_child = &gtk::ScrolledWindow {
                set_visible: cfg!(debug_assertions),
                set_min_content_width: 600,
                set_min_content_height: 240,
                set_max_content_height: 360,
                set_hscrollbar_policy: gtk::PolicyType::Automatic,
                set_vscrollbar_policy: gtk::PolicyType::Automatic,

                gtk::TextView {
                    set_buffer: Some(&model.details_buffer),
                    set_editable: false,
                    set_cursor_visible: true,
                    set_monospace: true,
                    set_wrap_mode: gtk::WrapMode::None,
                }
            }
        }
    }

    fn init(
        parent: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            parent,
            summary: String::new(),
            details: String::new(),
            details_buffer: gtk::TextBuffer::new(None),
        };

        let widgets = view_output!();

        root.connect_response(Some(RESPONSE_COPY_CLOSE), move |_, _| {
            sender.input(ErrorDialogMsg::CopyAndClose);
        });

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match msg {
            ErrorDialogMsg::Show(err) => {
                self.summary = format!("{err:#}");
                self.details = format!("{err:?}");
                warn!("{}", self.details);
                self.details_buffer.set_text(&self.details);
                self.update_view(widgets, sender);
                root.present(Some(&self.parent));
            }
            ErrorDialogMsg::CopyAndClose => {
                root.clipboard().set_text(&self.details);
            }
        }
    }
}
