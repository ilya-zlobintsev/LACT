use super::power_states_list::PowerStatesList;
use crate::app::pages::oc_page::power_states::power_states_list::{
    PowerStatesListMsg, PowerStatesListOptions,
};
use amdgpu_sysfs::gpu_handle::PowerLevelKind;
use gtk::prelude::{BoxExt, CheckButtonExt, OrientableExt, WidgetExt};
use lact_schema::{DeviceStats, PowerStates};
use relm4::{
    binding::BoolBinding, Component, ComponentController, ComponentParts, ComponentSender,
    RelmObjectExt, RelmWidgetExt,
};
use std::{collections::HashMap, sync::Arc};

pub struct PowerStatesFrame {
    core_states_list: relm4::Controller<PowerStatesList>,
    vram_states_list: relm4::Controller<PowerStatesList>,
    states_configurable: BoolBinding,
    states_expanded: BoolBinding,
    vram_clock_ratio: f64,
}

#[derive(Debug)]
pub enum PowerStatesFrameMsg {
    PowerStates(PowerStates),
    Stats(Arc<DeviceStats>),
    VramClockRatio(f64),
    Configurable(bool),
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for PowerStatesFrame {
    type Init = ();
    type Input = PowerStatesFrameMsg;
    type Output = ();

    view! {
        gtk::Expander {
            set_label: Some("Power states"),
            add_binding: (&model.states_expanded, "expanded"),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 10,
                set_spacing: 5,
                add_binding: (&model.states_configurable, "sensitive"),

                gtk::Label {
                    set_label: "Note: performance level must be set to 'manual' to toggle power states",
                    set_margin_horizontal: 10,
                    set_halign: gtk::Align::Start,
                },

                gtk::CheckButton {
                    // TODO: connect this
                    set_label: Some("Enable power state configuration"),
                },

                gtk::Box {
                    set_spacing: 10,
                    set_orientation: gtk::Orientation::Horizontal,

                    append = model.core_states_list.widget(),
                    append = model.vram_states_list.widget(),
                }
            }
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let core_states_list = PowerStatesList::builder()
            .launch(PowerStatesListOptions {
                title: "GPU Power States",
                value_suffix: "MHz",
            })
            .detach();
        let vram_states_list = PowerStatesList::builder()
            .launch(PowerStatesListOptions {
                title: "VRAM Power States",
                value_suffix: "MHz",
            })
            .detach();

        let model = Self {
            core_states_list,
            vram_states_list,
            states_configurable: BoolBinding::new(false),
            states_expanded: BoolBinding::new(false),
            vram_clock_ratio: 1.0,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            PowerStatesFrameMsg::PowerStates(pstates) => {
                self.states_configurable.set_value(!pstates.is_empty());
                // if !self.states_available.value() {
                //     self.states_expanded.set_value(false);
                // }

                self.core_states_list
                    .emit(PowerStatesListMsg::PowerStates(pstates.core, 1.0));
                self.vram_states_list.emit(PowerStatesListMsg::PowerStates(
                    pstates.vram,
                    self.vram_clock_ratio,
                ));
            }
            PowerStatesFrameMsg::Stats(stats) => {
                self.core_states_list
                    .emit(PowerStatesListMsg::ActiveState(stats.core_power_state));
                self.vram_states_list
                    .emit(PowerStatesListMsg::ActiveState(stats.memory_power_state));
            }
            PowerStatesFrameMsg::VramClockRatio(ratio) => {
                self.vram_clock_ratio = ratio;
            }
            PowerStatesFrameMsg::Configurable(configurable) => {
                self.states_configurable.set_value(configurable);
            }
        }
    }
}

impl PowerStatesFrame {
    pub fn get_enabled_power_states(&self) -> HashMap<PowerLevelKind, Vec<u8>> {
        let state_types = [
            (PowerLevelKind::CoreClock, &self.core_states_list),
            (PowerLevelKind::MemoryClock, &self.vram_states_list),
        ];

        if self.states_configurable.value() {
            state_types
                .into_iter()
                .map(|(kind, child)| (kind, child.model().get_enabled_power_states()))
                .collect()
        } else {
            state_types
                .into_iter()
                .map(|(kind, _)| (kind, vec![]))
                .collect()
        }
    }
}
