use super::ControlInit;
use adw::prelude::*;
use relm4::{
    Component, ComponentParts, ComponentSender, RelmObjectExt,
    binding::{Binding, U32Binding},
    css,
};

#[derive(Default)]
pub struct SelectOption {
    pub label: String,
    pub description: String,
}

impl From<String> for SelectOption {
    fn from(label: String) -> Self {
        Self {
            label,
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct SelectInit {
    pub options: Vec<SelectOption>,
    /// Index into `options`. Missing or invalid indices select the first option.
    pub selected: Option<u32>,
    pub default_value: Option<u32>,
}

pub struct SelectControl {
    value: U32Binding,
    default_value: Option<u32>,
    options: Vec<SelectOption>,
    title: gtk::Box,
    pub(super) size_group_widgets: Vec<(gtk::SizeGroup, gtk::Widget)>,
}

#[derive(Debug)]
pub enum SelectControlMsg {
    Changed,
}

#[relm4::component(pub)]
impl Component for SelectControl {
    type Init = ControlInit<SelectInit>;
    type Input = SelectControlMsg;
    type Output = u32;
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_spacing: 12,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_hexpand: true,
                set_valign: gtk::Align::Center,
                append: &model.title,

                #[name = "description_label"]
                gtk::Label {
                    #[watch]
                    set_label: model.description(),
                    #[watch]
                    set_visible: !model.description().is_empty(),
                    set_xalign: 0.0,
                    set_wrap: true,
                    add_css_class: css::DIM_LABEL,
                    add_css_class: css::CAPTION,
                },
            },

            #[name = "dropdown"]
            gtk::DropDown {
                set_valign: gtk::Align::Center,
                set_model: Some(&model.options.iter().map(|option| option.label.clone()).collect::<gtk::StringList>()),
                add_binding: (&model.value, "selected"),
            },
        },
    }

    fn init(
        ControlInit {
            init,
            title,
            label_group,
            input_group,
        }: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = Self {
            value: U32Binding::new(
                init.selected
                    .filter(|selected| *selected < init.options.len() as u32)
                    .unwrap_or(0),
            ),
            default_value: init.default_value,
            options: init.options,
            title,
            size_group_widgets: Vec::new(),
        };
        let widgets = view_output!();
        for (group, widget) in [
            (label_group, model.title.clone().upcast::<gtk::Widget>()),
            (input_group, widgets.dropdown.clone().upcast()),
        ] {
            if let Some(group) = group {
                group.add_widget(&widget);
                model.size_group_widgets.push((group, widget));
            }
        }
        model.value.connect_value_notify(move |_| {
            sender.input(SelectControlMsg::Changed);
        });
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            SelectControlMsg::Changed => {
                let _ = sender.output(self.get_value());
            }
        }
    }
}

impl SelectControl {
    pub(super) fn get_value(&self) -> u32 {
        self.value.get()
    }

    pub(super) fn set_value(&self, value: u32) {
        if self.get_value() != value {
            self.value.set(value);
        }
    }

    pub(super) fn reset(&self) {
        if let Some(value) = self.default_value {
            self.set_value(value);
        }
    }

    pub(super) fn serialize(&self) -> Option<u32> {
        let value = self.get_value();
        (Some(value) != self.default_value).then_some(value)
    }

    fn description(&self) -> &str {
        self.options
            .get(self.get_value() as usize)
            .map(|option| option.description.as_str())
            .unwrap_or_default()
    }
}
