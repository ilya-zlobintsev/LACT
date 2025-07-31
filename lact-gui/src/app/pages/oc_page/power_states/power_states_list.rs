use gtk::prelude::{BoxExt, CheckButtonExt, FrameExt, OrientableExt, WidgetExt};
use lact_schema::PowerState;
use relm4::{
    binding::BoolBinding, prelude::FactoryVecDeque, ComponentParts, ComponentSender, RelmObjectExt,
};

use crate::{app::msg::AppMsg, APP_BROKER};

pub struct PowerStatesList {
    states: FactoryVecDeque<PowerStateRow>,
    value_suffix: String,
}

pub struct PowerStatesListOptions {
    pub title: String,
    pub value_suffix: String,
}

#[derive(Debug)]
pub enum PowerStatesListMsg {
    PowerStates(Vec<PowerState>, f64),
    ActiveState(Option<usize>),
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for PowerStatesList {
    type Init = PowerStatesListOptions;
    type Input = PowerStatesListMsg;
    type Output = ();

    view! {
        gtk::Frame {
            set_hexpand: true,
            set_label: Some(&opts.title),
            set_child: Some(model.states.widget()),
        }
    }

    fn init(
        opts: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let states = FactoryVecDeque::builder().launch_default().detach();

        let model = Self {
            states,
            value_suffix: opts.value_suffix,
        };

        let widgets = view_output!();

        ComponentParts { widgets, model }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            PowerStatesListMsg::PowerStates(new_pstates, value_ratio) => {
                let mut states = self.states.guard();
                states.clear();

                for mut power_state in new_pstates {
                    power_state.value = (power_state.value as f64 * value_ratio) as u64;
                    let opts = PowerStateRowOptions {
                        power_state,
                        value_suffix: self.value_suffix.clone(),
                        active: false,
                    };
                    states.push_back(opts);
                }
            }
            PowerStatesListMsg::ActiveState(active_idx) => {
                for i in 0..self.states.len() {
                    let active = Some(i) == active_idx;
                    self.states.send(i, active);
                }
            }
        }
    }
}

impl PowerStatesList {
    pub fn get_enabled_power_states(&self) -> Vec<u8> {
        self.states
            .iter()
            .filter(|row| row.enabled.value())
            .filter_map(|row| row.power_state.index)
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }
}

struct PowerStateRow {
    active: BoolBinding,
    enabled: BoolBinding,
    power_state: PowerState,
    value_suffix: String,
}

pub struct PowerStateRowOptions {
    pub power_state: PowerState,
    pub value_suffix: String,
    pub active: bool,
}

#[relm4::factory]
impl relm4::factory::FactoryComponent for PowerStateRow {
    type ParentWidget = gtk::ListBox;
    type CommandOutput = ();
    type Input = bool;
    type Output = ();
    type Init = PowerStateRowOptions;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 5,

            append = &gtk::CheckButton {
                set_hexpand: true,
                add_binding: (&self.enabled, "active"),
                set_label: {
                    let value_text = match self.power_state.min_value {
                        Some(min) if min != self.power_state.value => format!("{min}-{}", self.power_state.value),
                        _ => self.power_state.value.to_string(),
                    };
                    Some(&format!("{}: {value_text} {}", index.current_index(), self.value_suffix))
                }
            },

            append: image = &gtk::Image {
                set_icon_name: Some("pan-start-symbolic"),
                add_binding: (&self.active, "visible"),
            },
        }
    }

    fn init_model(
        opts: Self::Init,
        _index: &Self::Index,
        _sender: relm4::FactorySender<Self>,
    ) -> Self {
        let enabled = BoolBinding::new(opts.power_state.enabled);
        enabled.connect_value_notify(|_| APP_BROKER.send(AppMsg::SettingsChanged));

        Self {
            enabled,
            active: BoolBinding::new(opts.active),
            power_state: opts.power_state,
            value_suffix: opts.value_suffix,
        }
    }

    fn update(&mut self, active: Self::Input, _: relm4::FactorySender<Self>) {
        self.active.set_value(active);
    }
}
