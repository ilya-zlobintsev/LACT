use super::{ControlInit, boolean_value::BooleanValue, setting_value::SettingValue};
use adw::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, binding::ConnectBinding};

#[derive(Default)]
pub struct BooleanInit {
    pub value: bool,
    pub default_value: Option<bool>,
}

pub struct BooleanControl {
    pub(super) value: BooleanValue,
    title: gtk::Box,
    pub(super) size_group_widgets: Vec<(gtk::SizeGroup, gtk::Widget)>,
}

#[derive(Debug)]
pub enum BooleanControlMsg {
    Changed,
}

#[relm4::component(pub)]
impl Component for BooleanControl {
    type Init = ControlInit<BooleanInit>;
    type Input = BooleanControlMsg;
    type Output = bool;
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_spacing: 12,
            append: &model.title,

            #[name = "switch"]
            gtk::Switch {
                set_valign: gtk::Align::Center,
                set_halign: gtk::Align::End,
                bind: model.value.binding(),
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
            value: BooleanValue::new(init.value, init.default_value),
            title,
            size_group_widgets: Vec::new(),
        };
        let widgets = view_output!();
        for (group, widget) in [
            (label_group, model.title.clone().upcast::<gtk::Widget>()),
            (input_group, widgets.switch.clone().upcast()),
        ] {
            if let Some(group) = group {
                group.add_widget(&widget);
                model.size_group_widgets.push((group, widget));
            }
        }
        model.value.binding().connect_value_notify(move |_| {
            sender.input(BooleanControlMsg::Changed);
        });
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            BooleanControlMsg::Changed => {
                let _ = sender.output(self.value.get_value());
            }
        }
    }
}
