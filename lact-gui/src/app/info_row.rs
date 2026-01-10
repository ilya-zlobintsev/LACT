use gtk::glib::{self, subclass::types::IsSubclassable, Object};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

glib::wrapper! {
    pub struct InfoRow(ObjectSubclass<imp::InfoRow>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Orientable, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
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

pub trait InfoRowExt {
    fn append_child(&self, widget: &impl IsA<gtk::Widget>);
    fn set_value_size_group(&self, size_group: &gtk::SizeGroup);

    fn set_name(&self, name: String);
    fn set_value(&self, value: String);
    fn set_icon(&self, icon: String);
    fn set_level_value(&self, level_value: f64);

    fn set_popover(&self, popover: &gtk::Popover);

    fn connect_clicked<F: Fn(&InfoRow) + 'static>(&self, f: F);
}

impl<T: IsA<InfoRow>> InfoRowExt for T {
    fn append_child(&self, widget: &impl IsA<gtk::Widget>) {
        self.as_ref().imp().value_box.append(widget);
    }

    fn set_value_size_group(&self, size_group: &gtk::SizeGroup) {
        size_group.add_widget(&self.as_ref().imp().value_label);
    }

    fn set_name(&self, name: String) {
        self.as_ref().set_property("name", name);
    }

    fn set_value(&self, value: String) {
        self.as_ref().set_property("value", value);
    }

    fn set_icon(&self, icon: String) {
        self.as_ref().set_property("icon", icon);
    }

    fn set_level_value(&self, level_value: f64) {
        self.as_ref().set_property("level-value", level_value);
    }

    fn set_popover(&self, popover: &gtk::Popover) {
        popover.set_parent(self.as_ref());
    }

    fn connect_clicked<F: Fn(&InfoRow) + 'static>(&self, f: F) {
        let gesture = gtk::GestureClick::new();
        let obj = self.as_ref().clone();
        gesture.connect_released(move |_, _, _, _| {
            f(&obj);
        });
        self.as_ref().add_controller(gesture);
        self.as_ref().set_cursor_from_name(Some("pointer"));
    }
}

unsafe impl<T: ObjectSubclass + BoxImpl> IsSubclassable<T> for InfoRow {}

pub struct InfoRowItem {
    pub name: String,
    pub value: String,
    pub note: Option<&'static str>,
}

#[relm4::factory(pub)]
impl relm4::factory::FactoryComponent for InfoRowItem {
    type Init = Self;
    type ParentWidget = gtk::FlowBox;
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
    use relm4::{css, view, RelmWidgetExt};
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
        #[property(get, set)]
        icon: RefCell<String>,

        pub(super) info_menubutton: gtk::MenuButton,
        pub(super) value_box: gtk::Box,
        pub(super) value_label: gtk::Label,
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
            let value_box = &self.value_box;
            let value_label = &self.value_label;
            let info_menubutton = &self.info_menubutton;

            view! {
                #[local_ref]
                obj {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_margin_all: 5,
                    set_spacing: 5,

                    append = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_hexpand: true,
                        set_valign: gtk::Align::Center,

                        append: name_label = &gtk::Label {
                            set_halign: gtk::Align::Start,
                            set_xalign: 0.0,
                            add_css_class: css::CAPTION,
                            add_css_class: css::DIM_LABEL,
                        },

                        #[local_ref]
                        append = value_box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 5,

                            #[local_ref]
                            append = value_label {
                                set_attributes: Some(&AttrList::from_str("0 -1 weight bold").unwrap()),
                                set_halign: gtk::Align::Start,
                                set_xalign: 0.0,
                                set_use_markup: true,
                                set_ellipsize: pango::EllipsizeMode::End,
                            },
                        },
                    },

                    #[local_ref]
                    append = info_menubutton {
                        set_icon_name: "dialog-information-symbolic",
                        add_css_class: css::FLAT,
                        set_valign: gtk::Align::Center,

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

                    append: icon_image = &gtk::Image {
                        set_valign: gtk::Align::Center,
                    },
                }
            }

            obj.bind_property("value", value_label, "visible")
                .transform_to(|_, text: String| Some(!text.is_empty()))
                .sync_create()
                .build();

            obj.bind_property("name", &name_label, "label")
                .sync_create()
                .build();

            obj.bind_property("name", &name_label, "visible")
                .transform_to(|_, text: String| Some(!text.is_empty()))
                .sync_create()
                .build();

            obj.bind_property("info-text", info_menubutton, "visible")
                .transform_to(|_, text: String| Some(!text.is_empty()))
                .sync_create()
                .build();

            obj.bind_property("info-text", &info_text_popover, "label")
                .sync_create()
                .build();

            obj.bind_property("value", value_label, "label")
                .sync_create()
                .build();

            obj.bind_property("icon", &icon_image, "icon-name")
                .sync_create()
                .build();

            obj.bind_property("icon", &icon_image, "visible")
                .transform_to(|_, text: String| Some(!text.is_empty()))
                .sync_create()
                .build();

            obj.bind_property("selectable", value_label, "selectable")
                .sync_create()
                .build();

            obj.bind_property("value", info_menubutton, "visible")
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
