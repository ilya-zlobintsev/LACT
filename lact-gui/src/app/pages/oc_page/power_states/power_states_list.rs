use crate::app::pages::oc_page::power_states::power_states_row::{
    PowerStateRow, PowerStateRowMsg, PowerStateRowOptions,
};
use gtk::prelude::{FrameExt, WidgetExt};
use lact_schema::PowerState;
use relm4::{
    ComponentParts, ComponentSender, RelmWidgetExt, binding::BoolBinding, css,
    prelude::FactoryVecDeque,
};

pub struct PowerStatesList {
    states: FactoryVecDeque<PowerStateRow>,
    value_suffix: String,
    show_active_indicator: BoolBinding,
}

pub struct PowerStatesListOptions {
    pub title: String,
    pub value_suffix: String,
}

#[derive(Debug)]
pub enum PowerStatesListMsg {
    PowerStates(Vec<PowerState>, f64),
    ActiveState(Option<usize>),
    Configurable(bool),
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for PowerStatesList {
    type Init = PowerStatesListOptions;
    type Input = PowerStatesListMsg;
    type Output = ();

    view! {
        gtk::Frame {
            set_hexpand: true,
            #[wrap(Some)]
            set_label_widget = &gtk::Label {
                set_label: &opts.title,
                set_margin_horizontal: 5,
                add_css_class: css::CAPTION_HEADING,
            },
            #[local_ref]
            states_widget -> gtk::ListBox {
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
        let show_active_indicator = BoolBinding::new(false);

        let model = Self {
            states,
            value_suffix: opts.value_suffix,
            show_active_indicator,
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

                for mut power_state in new_pstates {
                    power_state.value = (power_state.value as f64 * value_ratio) as u64;
                    let opts = PowerStateRowOptions {
                        power_state,
                        value_suffix: self.value_suffix.clone(),
                        active: false,
                        show_active_indicator: self.show_active_indicator.clone(),
                    };
                    states.push_back(opts);
                }
            }
            PowerStatesListMsg::ActiveState(active_idx) => {
                self.show_active_indicator.set_value(active_idx.is_some());
                for (i, row) in self.states.iter().enumerate() {
                    let is_active = row
                        .power_state
                        .index
                        .is_some_and(|index| Some(usize::from(index)) == active_idx);

                    self.states.send(i, PowerStateRowMsg::Active(is_active));
                }
            }
            PowerStatesListMsg::Configurable(configurable) => {
                self.states
                    .broadcast(PowerStateRowMsg::Configurable(configurable));
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
