use gtk::glib::{self, Object};

glib::wrapper! {
    pub struct InfoRow(ObjectSubclass<imp::InfoRow>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Orientable, gtk::Accessible, gtk::Buildable;
}

impl InfoRow {
    pub fn new(name: &str, value: &str) -> Self {
        Object::builder()
            .property("name", name)
            .property("value", value)
            .build()
    }

    pub fn new_selectable(name: &str, value: &str) -> Self {
        Object::builder()
            .property("name", name)
            .property("value", value)
            .property("selectable", true)
            .build()
    }
}

pub struct InfoRowItem {
    pub name: String,
    pub value: String,
    pub note: Option<&'static str>,
}

#[relm4::factory(pub)]
impl relm4::factory::FactoryComponent for InfoRowItem {
    type Init = Self;
    type ParentWidget = gtk::Box;
    type CommandOutput = ();
    type Input = ();
    type Output = ();

    fn init_model(
        init: Self::Init,
        _index: &Self::Index,
        _sender: relm4::FactorySender<Self>,
    ) -> Self {
        init
    }

    view! {
        InfoRow {
            set_selectable: true,
            set_name: self.name.clone(),
            set_value: self.value.clone(),
            set_info_text: self.note.unwrap_or_default(),
        }
    }
}

impl Default for InfoRow {
    fn default() -> Self {
        Object::builder().build()
    }
}

mod imp {
    use glib::Properties;
    use gtk::{
        glib,
        pango::{self, AttrList},
        prelude::*,
        subclass::{prelude::*, widget::WidgetImpl},
        Label,
    };
    use relm4::{view, RelmWidgetExt};
    use std::{cell::RefCell, str::FromStr};

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::InfoRow)]
    pub struct InfoRow {
        #[property(get, set)]
        name: RefCell<String>,
        #[property(get, set)]
        value: RefCell<String>,
        #[property(get, set)]
        selectable: RefCell<bool>,
        #[property(get, set)]
        info_text: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for InfoRow {
        const NAME: &'static str = "InfoRow";
        type Type = super::InfoRow;
        type ParentType = gtk::Box;
    }

    #[glib::derived_properties]
    impl ObjectImpl for InfoRow {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            view! {
                #[local_ref]
                obj {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_hexpand: true,

                    append: name_label = &gtk::Label {
                        set_halign: gtk::Align::Start,
                        set_hexpand: true,
                    },

                    append: info_menubutton = &gtk::MenuButton {
                        set_icon_name: "dialog-information-symbolic",
                        set_margin_horizontal: 5,

                        #[wrap(Some)]
                        set_popover = &gtk::Popover {
                            #[name(info_text_popover)]
                            Label {
                                set_wrap: true,
                                set_wrap_mode: pango::WrapMode::Word,
                                set_max_width_chars: 55,
                            }
                        },
                    },

                    append: value_label = &gtk::Label {
                        set_attributes: Some(&AttrList::from_str("0 -1 weight bold").unwrap()),
                        set_halign: gtk::Align::End,
                        set_use_markup: true,
                        set_ellipsize: pango::EllipsizeMode::End,
                    }
                }
            }

            obj.bind_property("name", &name_label, "label")
                .transform_to(|_, value: &str| Some(format!("{value}:")))
                .sync_create()
                .build();

            obj.bind_property("info-text", &info_menubutton, "visible")
                .transform_to(|_, text: String| Some(!text.is_empty()))
                .sync_create()
                .build();

            obj.bind_property("info-text", &info_text_popover, "label")
                .sync_create()
                .build();

            obj.bind_property("value", &value_label, "label")
                .sync_create()
                .build();

            obj.bind_property("selectable", &value_label, "selectable")
                .sync_create()
                .build();

            obj.bind_property("value", &info_menubutton, "visible")
                .transform_to(|_, text: String| {
                    if text.starts_with("Unknown ") {
                        Some(false)
                    } else {
                        None
                    }
                })
                .sync_create()
                .build();
        }
    }

    impl WidgetImpl for InfoRow {}
    impl BoxImpl for InfoRow {}
}
