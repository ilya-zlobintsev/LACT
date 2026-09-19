use crate::{
    I18N,
    app::{DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH, pages::PageId},
};
use adw::prelude::*;
use gtk::glib;
use i18n_embed_fl::fl;
use relm4::{
    ComponentParts, ComponentSender, RelmObjectExt, RelmWidgetExt, binding::BoolBinding, css,
};

const CONTENT_MAXIMUM_WIDTH: i32 = 1200;

pub struct DetachablePage {
    pub init: DetachablePageInit,
    detached: bool,
}

pub struct DetachablePageInit {
    pub id: PageId,
    pub title: String,
    pub content: gtk::Widget,
    pub parent: adw::ApplicationWindow,
    pub sensitive: BoolBinding,
}

#[derive(Debug)]
pub enum DetachablePageMsg {
    ToggleDetached,
    Detach,
    Attach,
}

#[relm4::component(pub)]
impl relm4::Component for DetachablePage {
    type Init = DetachablePageInit;
    type Input = DetachablePageMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            #[name = "content"]
            gtk::ScrolledWindow {
                set_hscrollbar_policy: gtk::PolicyType::Never,
                set_vexpand: true,
                add_binding: (&model.init.sensitive, "sensitive"),

                adw::Clamp {
                    set_maximum_size: CONTENT_MAXIMUM_WIDTH,
                    set_tightening_threshold: CONTENT_MAXIMUM_WIDTH,
                    set_child: Some(&model.init.content),
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 15,
                set_margin_all: 15,
                set_align: gtk::Align::Center,
                set_vexpand: true,
                #[watch]
                set_visible: model.detached,

                gtk::Label {
                    set_label: &fl!(I18N, "page-detached"),
                },
                gtk::Button {
                    set_label: &fl!(I18N, "show-page-window"),
                    connect_clicked => DetachablePageMsg::Detach,
                },
                gtk::Button {
                    set_label: &fl!(I18N, "reattach-page"),
                    connect_clicked => DetachablePageMsg::Attach,
                },
            },
        },

        #[name = "row"]
        gtk::ListBoxRow {
            update_property: &[gtk::accessible::Property::Label(&model.init.title)],
            #[wrap(Some)]
            set_child = &gtk::Box {
                set_spacing: 5,
                gtk::Label {
                    set_label: &model.init.title,
                    set_halign: gtk::Align::Start,
                    set_hexpand: true,
                },
                gtk::Button {
                    set_icon_name: "window-new-symbolic",
                    #[watch]
                    set_tooltip_text: Some(&if model.detached {
                        fl!(I18N, "reattach-page")
                    } else {
                        fl!(I18N, "detach-page", page = model.init.title.as_str())
                    }),
                    add_css_class: css::FLAT,
                    add_css_class: "page-detach-button",
                    connect_clicked => DetachablePageMsg::ToggleDetached,
                },
            },
        },

        #[name = "window"]
        adw::Window {
            set_title: Some(&model.init.title),
            set_default_width: DEFAULT_WINDOW_WIDTH,
            set_default_height: DEFAULT_WINDOW_HEIGHT,
            set_application: model.init.parent.application().as_ref(),
            #[watch]
            set_visible: model.detached,
            connect_close_request[sender] => move |_| {
                sender.input(DetachablePageMsg::Attach);
                glib::Propagation::Stop
            },

            #[name = "toolbar"]
            adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {},
            },
        },

        #[local_ref]
        parent -> adw::ApplicationWindow {
            connect_close_request[sender, window] => move |_| {
                // Hide immediately, before the application's main loop can stop.
                window.set_visible(false);
                window.set_application(gtk::Application::NONE);
                sender.input(DetachablePageMsg::Attach);
                glib::Propagation::Proceed
            },
        },
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            init,
            detached: false,
        };
        let parent = &model.init.parent;
        let widgets = view_output!();
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
            DetachablePageMsg::ToggleDetached => {
                sender.input(if self.detached {
                    DetachablePageMsg::Attach
                } else {
                    DetachablePageMsg::Detach
                });
            }
            DetachablePageMsg::Detach => {
                if !self.detached {
                    root.remove(&widgets.content);
                    widgets.toolbar.set_content(Some(&widgets.content));
                    // Focusing the first input would scroll the page back to that input.
                    GtkWindowExt::set_focus(&widgets.window, Some(&widgets.content));
                    self.detached = true;
                }
                widgets.window.present();
            }
            DetachablePageMsg::Attach => {
                if self.detached {
                    widgets.toolbar.set_content(gtk::Widget::NONE);
                    root.append(&widgets.content);
                    self.detached = false;
                }
            }
        }
        self.update_view(widgets, sender);
    }

    fn shutdown(&mut self, widgets: &mut Self::Widgets, _output: relm4::Sender<Self::Output>) {
        widgets.window.destroy();
    }
}
