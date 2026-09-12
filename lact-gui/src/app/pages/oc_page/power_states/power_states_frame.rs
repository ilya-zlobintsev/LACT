use super::power_states_list::PowerStatesList;
use super::power_states_list::{PowerStatesListMsg, PowerStatesListOptions};
use crate::{
    APP_BROKER, I18N,
    app::{
        components::{adjustment_card::AdjustmentCard, page_section::PageSection},
        msg::AppMsg,
        pages::oc_page::OcPageMsg,
        utils::ext::RelmLaunchable as _,
    },
};
use adw::prelude::*;
use amdgpu_sysfs::gpu_handle::{PerformanceLevel, PowerLevelKind};
use gtk::glib::{SignalHandlerId, object::ObjectExt};
use i18n_embed_fl::fl;
use indexmap::IndexMap;
use lact_schema::{DeviceStats, PowerStates};
use relm4::{
    ComponentController, ComponentParts, ComponentSender,
    binding::{Binding, BoolBinding},
};
use std::sync::Arc;

pub struct PowerStatesFrame {
    core_states_list: relm4::Controller<PowerStatesList>,
    vram_states_list: relm4::Controller<PowerStatesList>,
    states_configurable: BoolBinding,
    states_configuration_enabled: BoolBinding,
    performance_level: Option<PerformanceLevel>,
    configured_signal: SignalHandlerId,
    vram_clock_ratio: f64,
}

#[derive(Debug)]
pub enum PowerStatesFrameMsg {
    PowerStates {
        pstates: PowerStates,
        configured: bool,
    },
    Stats(Arc<DeviceStats>),
    PerformanceLevel(Option<PerformanceLevel>),
    VramClockRatio(f64),
    Configurable(bool),
    ConfiguredToggled {
        configured: bool,
    },
    EnableWithManualPerformanceLevel,
    InternalConfigurableChanged(bool),
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for PowerStatesFrame {
    type Init = ();
    type Input = PowerStatesFrameMsg;
    type Output = OcPageMsg;

    view! {
        PageSection::new(&fl!(I18N, "pstates")) {
            #[template]
            append_child = &AdjustmentCard {
                #[template_child]
                advanced_features {
                    #[watch]
                    set_visible: model.performance_level.is_some(),
                },

                #[template_child]
                controls {
                    gtk::ToggleButton {
                        set_halign: gtk::Align::Start,
                        add_css_class: "adjustment-card-option-toggle",

                        #[watch]
                        #[block_signal(configured_toggled_handler)]
                        set_active: model.states_configuration_enabled.value(),

                        connect_toggled[sender] => move |button| {
                            sender.input(PowerStatesFrameMsg::ConfiguredToggled {
                                configured: button.is_active(),
                            });
                        } @ configured_toggled_handler,

                        #[wrap(Some)]
                        set_child = &gtk::Box {
                            gtk::Label {
                                set_label: &fl!(I18N, "enable-pstate-config"),
                            },
                        },
                    },
                },

                #[template_child]
                content {
                    gtk::Box {
                        set_spacing: 10,
                        set_orientation: gtk::Orientation::Horizontal,
                        set_homogeneous: true,

                        gtk::Box {
                            #[watch]
                            set_visible: !model.core_states_list.model().is_empty(),
                            append = model.core_states_list.widget(),
                        },

                        gtk::Box {
                            #[watch]
                            set_visible: !model.vram_states_list.model().is_empty(),
                            append = model.vram_states_list.widget(),
                        },
                    },
                },
            },
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let core_states_list = PowerStatesList::detach(PowerStatesListOptions {
            title: fl!(I18N, "gpu-pstates"),
            value_suffix: fl!(I18N, "mhz"),
        });
        let vram_states_list = PowerStatesList::detach(PowerStatesListOptions {
            title: fl!(I18N, "vram-pstates"),
            value_suffix: fl!(I18N, "mhz"),
        });

        let states_configuration_enabled = BoolBinding::new(false);

        let configured_sender = sender.clone();
        let configured_signal =
            states_configuration_enabled.connect_value_notify(move |states_configured| {
                configured_sender.input(PowerStatesFrameMsg::InternalConfigurableChanged(
                    states_configured.get(),
                ));
                APP_BROKER.send(AppMsg::SettingsChanged);
            });

        let model = Self {
            core_states_list,
            vram_states_list,
            states_configurable: BoolBinding::new(false),
            states_configuration_enabled,
            configured_signal,
            performance_level: None,
            vram_clock_ratio: 1.0,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            PowerStatesFrameMsg::PowerStates {
                pstates,
                configured,
            } => {
                self.states_configuration_enabled
                    .block_signal(&self.configured_signal);
                self.states_configuration_enabled.set_value(configured);
                self.states_configuration_enabled
                    .unblock_signal(&self.configured_signal);

                self.core_states_list
                    .emit(PowerStatesListMsg::PowerStates(pstates.core, 1.0));
                self.vram_states_list.emit(PowerStatesListMsg::PowerStates(
                    pstates.vram,
                    self.vram_clock_ratio,
                ));
            }
            PowerStatesFrameMsg::Stats(stats) => {
                self.core_states_list.emit(PowerStatesListMsg::ActiveState(
                    stats.active_power_states.and_then(|states| states.core),
                ));
                self.vram_states_list.emit(PowerStatesListMsg::ActiveState(
                    stats.active_power_states.and_then(|states| states.memory),
                ));
            }
            PowerStatesFrameMsg::VramClockRatio(ratio) => {
                self.vram_clock_ratio = ratio;
            }
            PowerStatesFrameMsg::Configurable(is_plvl_manual) => {
                let configurable = is_plvl_manual
                    && (!self.core_states_list.model().is_empty()
                        || !self.vram_states_list.model().is_empty());
                self.states_configurable.set_value(configurable);

                if !configurable {
                    self.states_configuration_enabled
                        .block_signal(&self.configured_signal);
                    self.states_configuration_enabled.set_value(false);
                    self.states_configuration_enabled
                        .unblock_signal(&self.configured_signal);
                }

                self.core_states_list.emit(PowerStatesListMsg::Configurable(
                    configurable && self.states_configuration_enabled.value(),
                ));
                self.vram_states_list.emit(PowerStatesListMsg::Configurable(
                    configurable && self.states_configuration_enabled.value(),
                ));
            }
            PowerStatesFrameMsg::PerformanceLevel(level) => {
                self.performance_level = level;
            }
            PowerStatesFrameMsg::ConfiguredToggled { configured } => {
                if !configured || self.performance_level == Some(PerformanceLevel::Manual) {
                    self.states_configuration_enabled.set_value(configured);
                } else {
                    APP_BROKER.send(AppMsg::EnablePstateConfig);
                }
            }
            PowerStatesFrameMsg::EnableWithManualPerformanceLevel => {
                sender
                    .output(OcPageMsg::SetPerformanceLevel(PerformanceLevel::Manual))
                    .unwrap();
                self.states_configuration_enabled.set_value(true);
            }
            PowerStatesFrameMsg::InternalConfigurableChanged(configurable) => {
                self.core_states_list
                    .emit(PowerStatesListMsg::Configurable(configurable));
                self.vram_states_list
                    .emit(PowerStatesListMsg::Configurable(configurable));
            }
        }
    }
}

impl PowerStatesFrame {
    pub fn get_enabled_power_states(&self) -> IndexMap<PowerLevelKind, Vec<u8>> {
        if self.states_configuration_enabled.value() {
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
        } else {
            IndexMap::new()
        }
    }
}
