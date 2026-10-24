use super::{select_value::SelectValue, setting_value::SettingValue};
use adw::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, RelmObjectExt, css};

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
    pub(super) value: SelectValue,
    options: Vec<SelectOption>,
    title: gtk::Box,
    pub(super) size_group_widgets: Vec<(gtk::SizeGroup, gtk::Widget)>,
    change_signal: Option<gtk::glib::SignalHandlerId>,
}

#[derive(Debug)]
pub enum SelectControlMsg {
    Changed,
}

#[relm4::component(pub)]
impl Component for SelectControl {
    type Init = (
        SelectInit,
        gtk::Box,
        Option<gtk::SizeGroup>,
        Option<gtk::SizeGroup>,
    );
    type Input = SelectControlMsg;
    type Output = ();
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
                set_model: Some(&options),
                add_binding: (model.value.binding(), "selected"),
            },

        },
    }

    fn init(
        (init, title, label_group, input_group): Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let options = gtk::StringList::new(
            &init
                .options
                .iter()
                .map(|option| option.label.as_str())
                .collect::<Vec<_>>(),
        );
        let mut model = Self {
            value: SelectValue::new(init.selected, init.default_value, init.options.len() as u32),
            options: init.options,
            title,
            size_group_widgets: Vec::new(),
            change_signal: None,
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
        model.change_signal = Some(model.value.binding().connect_value_notify(move |_| {
            sender.input(SelectControlMsg::Changed);
        }));
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            SelectControlMsg::Changed => {
                let _ = sender.output(());
            }
        }
    }

    fn shutdown(&mut self, _widgets: &mut Self::Widgets, _output: relm4::Sender<Self::Output>) {
        if let Some(signal) = self.change_signal.take() {
            self.value.binding().disconnect(signal);
        }
    }
}

impl SelectControl {
    fn description(&self) -> &str {
        self.options
            .get(self.value.get_value() as usize)
            .map(|option| option.description.as_str())
            .unwrap_or_default()
    }
}
