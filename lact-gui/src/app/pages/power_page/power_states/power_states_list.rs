use super::power_states_row::{PowerStateRow, PowerStateRowMsg, PowerStateRowOptions};
use amdgpu_sysfs::gpu_handle::PowerLevelId;
use gtk::{
    glib,
    prelude::{BoxExt, OrientableExt, WidgetExt},
};
use lact_schema::PowerState;
use relm4::{
    ComponentParts, ComponentSender, RelmWidgetExt, binding::BoolBinding, prelude::FactoryVecDeque,
};

pub struct PowerStatesList {
    states: FactoryVecDeque<PowerStateRow>,
    value_suffix: String,
    is_active_indicator_visible: BoolBinding,
    configurable: BoolBinding,
}

pub struct PowerStatesListOptions {
    pub title: String,
    pub value_suffix: String,
}

#[derive(Debug)]
pub enum PowerStatesListMsg {
    PowerStates(Vec<PowerState>, f64),
    ActiveState(Option<PowerLevelId>),
    Configurable(bool),
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for PowerStatesList {
    type Init = PowerStatesListOptions;
    type Input = PowerStatesListMsg;
    type Output = ();

    view! {
        gtk::Box {
            set_hexpand: true,
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 5,

            gtk::Label {
                set_use_markup: true,
                set_label: &format!(
                    "<span font_desc='13'><b>{}</b></span>",
                    glib::markup_escape_text(&opts.title)
                ),
                set_halign: gtk::Align::Start,
                set_margin_vertical: 5,
            },

            gtk::Frame {
                set_hexpand: true,
                #[local_ref]
                states_widget -> gtk::ListBox {
                    set_selection_mode: gtk::SelectionMode::None,
                },
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
                        show_active_indicator: self.is_active_indicator_visible.clone(),
                        configurable: self.configurable.clone(),
                    };
                    states.push_back(opts);
                }
            }
            PowerStatesListMsg::ActiveState(active_idx) => {
                self.is_active_indicator_visible
                    .set_value(active_idx.is_some());
                for (i, row) in self.states.iter().enumerate() {
                    let is_active = row.power_state.id == active_idx;

                    self.states.send(i, PowerStateRowMsg::Active(is_active));
                }
            }
            PowerStatesListMsg::Configurable(configurable) => {
                self.configurable.set_value(configurable);
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

    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }
}
