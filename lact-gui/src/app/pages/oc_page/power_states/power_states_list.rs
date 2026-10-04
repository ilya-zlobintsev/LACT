use super::power_states_row::{PowerStateRow, PowerStateRowMsg, PowerStateRowOptions};
use amdgpu_sysfs::gpu_handle::PowerLevelId;
use gtk::prelude::WidgetExt;
use lact_schema::PowerState;
use relm4::{ComponentParts, ComponentSender, binding::BoolBinding, prelude::FactoryVecDeque};

pub struct PowerStatesList {
    states: FactoryVecDeque<PowerStateRow>,
    value_suffix: String,
    is_active_indicator_visible: BoolBinding,
    configurable: BoolBinding,
    active_state: Option<PowerLevelId>,
}

pub struct PowerStatesListOptions {
    pub value_suffix: String,
}

#[derive(Debug)]
pub enum PowerStatesListMsg {
    PowerStates(Vec<PowerState>, f64),
    ActiveState(Option<PowerLevelId>),
    Configurable(bool),
    ValueRatio(f64),
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for PowerStatesList {
    type Init = PowerStatesListOptions;
    type Input = PowerStatesListMsg;
    type Output = ();

    view! {
        gtk::Box {
            set_hexpand: true,
            #[local_ref]
            states_widget -> gtk::ListBox {
                set_hexpand: true,
                set_selection_mode: gtk::SelectionMode::None,
            },
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
            is_active_indicator_visible: BoolBinding::new(false),
            configurable: BoolBinding::new(true),
            active_state: None,
        };

        let states_widget = model.states.widget();

        let widgets = view_output!();

        ComponentParts { widgets, model }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            PowerStatesListMsg::PowerStates(new_pstates, value_ratio) => {
                let mut states = self.states.guard();
                states.clear();

                for power_state in new_pstates {
                    let opts = PowerStateRowOptions {
                        power_state,
                        value_suffix: self.value_suffix.clone(),
                        value_ratio,
                        active: self.active_state.is_some() && power_state.id == self.active_state,
                        show_active_indicator: self.is_active_indicator_visible.clone(),
                        configurable: self.configurable.clone(),
                    };
                    states.push_back(opts);
                }
            }
            PowerStatesListMsg::ActiveState(active_idx) => {
                self.active_state = active_idx;
                self.is_active_indicator_visible
                    .set_value(active_idx.is_some());
                for (i, row) in self.states.iter().enumerate() {
                    let is_active = active_idx.is_some() && row.power_state.id == active_idx;

                    self.states.send(i, PowerStateRowMsg::Active(is_active));
                }
            }
            PowerStatesListMsg::Configurable(configurable) => {
                self.configurable.set_value(configurable);
            }
            PowerStatesListMsg::ValueRatio(ratio) => {
                self.states.broadcast(PowerStateRowMsg::ValueRatio(ratio));
            }
        }
    }
}

impl PowerStatesList {
    pub fn get_enabled_power_states(&self) -> Vec<u8> {
        self.states
            .iter()
            .filter(|row| row.enabled.value())
            .filter_map(|row| match row.power_state.id? {
                PowerLevelId::Index(index) => Some(index),
                PowerLevelId::Sleep => None,
            })
            .collect()
    }
}
