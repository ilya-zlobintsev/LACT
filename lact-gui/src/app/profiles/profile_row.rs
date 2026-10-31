use crate::I18N;
use crate::app::profiles::ProfileSelectorMsg;
use crate::app::{APP_BROKER, msg::AppMsg};
use gtk::{gdk, glib, graphene, pango, prelude::*};
use i18n_embed_fl::fl;
use lact_schema::{ProfileRule, config::ProfileHooks};
use relm4::{
    FactorySender, RelmWidgetExt, css,
    factory::{DynamicIndex, FactoryComponent},
};

pub struct ProfileRow {
    pub(super) row: ProfileRowType,
    pub(super) selected: bool,
    pub(super) activatable: bool,
}

#[derive(Debug)]
pub enum ProfileRowMsg {
    Selected(bool),
}

#[derive(Clone, glib::Boxed)]
#[boxed_type(name = "ProfileIndexValue")]
struct DynamicIndexValue(DynamicIndex);

#[derive(Debug, Clone)]
pub enum ProfileRowType {
    Default,
    Profile {
        name: String,
        rule: Option<ProfileRule>,
        hooks: ProfileHooks,
    },
}

impl ProfileRowType {
    pub fn name(&self) -> Option<String> {
        match self {
            Self::Default => None,
            Self::Profile { name, .. } => Some(name.clone()),
        }
    }
}

#[relm4::factory(pub)]
impl FactoryComponent for ProfileRow {
    type Init = Self;
    type Input = ProfileRowMsg;
    type Output = ProfileSelectorMsg;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        #[root]
        gtk::Box {
            set_spacing: 5,
            set_margin_all: 5,

            gtk::Image {
                set_icon_name: Some("list-drag-handle-symbolic"),
                set_opacity: if matches!(self.row, ProfileRowType::Profile { .. }) { 1.0 } else { 0.0 },
                add_css_class: css::DIM_LABEL,

                add_controller = gtk::DragSource {
                    set_actions: gdk::DragAction::MOVE,
                    connect_prepare[index, movable = matches!(self.row, ProfileRowType::Profile { .. })] => move |_, _, _| {
                        movable.then(|| gdk::ContentProvider::for_value(&DynamicIndexValue(index.clone()).to_value()))
                    },
                    connect_drag_begin[root = root.downgrade()] => move |source, drag| {
                        if let (Some(root), Some(handle)) = (root.upgrade(), source.widget()) {
                            relm4::view! {
                                icon = gtk::ListBox {
                                    gtk::ListBoxRow {
                                        set_state_flags: (gtk::StateFlags::DROP_ACTIVE, false),
                                        #[wrap(Some)]
                                        set_child = &gtk::Picture {
                                            set_paintable: Some(&gtk::WidgetPaintable::new(Some(&root))),
                                            set_margin_all: root.margin_top(),
                                        },
                                    },
                                }
                            }
                            gtk::DragIcon::for_drag(drag).set_child(Some(&icon));

                            if let Some(hotspot) = handle.compute_point(&root, &graphene::Point::new(handle.width() as f32 / 2.0, handle.height() as f32 / 2.0)) {
                                drag.set_hotspot(hotspot.x() as i32 + root.margin_start(), hotspot.y() as i32 + root.margin_top());
                            }
                        }
                    },
                },
            },

            #[name = "name_label"]
            gtk::Label {
                set_label: &match &self.row {
                    ProfileRowType::Default => fl!(I18N, "default-profile"),
                    ProfileRowType::Profile { name, .. } => name.clone(),
                },
                set_halign: gtk::Align::Start,
                set_hexpand: true,
                set_xalign: 0.0,
                set_ellipsize: pango::EllipsizeMode::End,
                set_width_request: 150,
            },

            gtk::MenuButton {
                set_icon_name: "open-menu-symbolic",
                #[wrap(Some)]
                set_popover = &gtk::Popover {
                    set_margin_all: 5,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 5,

                        gtk::Button {
                            set_label: &fl!(I18N, "rename-profile"),
                            set_visible: matches!(self.row, ProfileRowType::Profile { .. }),
                            connect_clicked[sender, index] => move |_| {
                                sender.output(ProfileSelectorMsg::RenameProfile(index.clone())).unwrap();
                            },
                            add_css_class: css::FLAT,
                        },

                        gtk::Button {
                            set_label: &fl!(I18N, "delete-profile"),
                            set_visible: matches!(self.row, ProfileRowType::Profile { .. }),
                            connect_clicked[profile = self.row.clone()] => move |_| {
                                if let ProfileRowType::Profile { name, .. } = profile.clone() {
                                    APP_BROKER.send(AppMsg::DeleteProfile(name));
                                }
                            },
                            add_css_class: css::FLAT,
                        },

                        gtk::Button {
                            set_label: &fl!(I18N, "edit-rules"),
                            connect_clicked[sender, index] => move |_| {
                                sender.output(ProfileSelectorMsg::ShowProfileEditor(index.clone())).unwrap();
                            },
                            add_css_class: css::FLAT,
                        },

                        gtk::Button {
                            set_label: &fl!(I18N, "export-to-file"),
                            connect_clicked[sender, index] => move |_| {
                                sender.output(ProfileSelectorMsg::ExportProfile(index.clone())).unwrap();
                            },
                            add_css_class: css::FLAT,
                        },
                    },
                }
            },
        },

        #[local_ref]
        returned_widget -> gtk::ListBoxRow {
            set_activatable: self.activatable,

            add_controller = gtk::DropTarget {
                set_actions: if matches!(self.row, ProfileRowType::Profile { .. }) { gdk::DragAction::MOVE } else { gdk::DragAction::empty() },
                set_types: &[DynamicIndexValue::static_type()],

                connect_drop[sender, index] => move |_, value, _, _| {
                    if let Ok(DynamicIndexValue(from)) = value.get::<DynamicIndexValue>() {
                        sender.output(ProfileSelectorMsg::MoveProfile { from, to: index.clone() }).unwrap();
                    }
                    true
                },
            },

            #[watch]
            set_state_flags: (
                if self.selected { gtk::StateFlags::SELECTED } else { gtk::StateFlags::empty() },
                false,
            ),
            #[watch]
            unset_state_flags: if self.selected { gtk::StateFlags::empty() } else { gtk::StateFlags::SELECTED },
        }
    }

    fn init_model(row: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        row
    }

    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        match msg {
            ProfileRowMsg::Selected(selected) => self.selected = selected,
        }
    }
}
