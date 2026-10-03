use super::power_states_list::PowerStatesList;
use super::power_states_list::{PowerStatesListMsg, PowerStatesListOptions};
use crate::{
    APP_BROKER, I18N,
    app::{
        components::{adjustment_card::AdjustmentCard, page_section::PageSection},
        msg::AppMsg,
        pages::oc_page::{OcPageMsg, clocks_frame::ClockDomain},
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
    ComponentController, ComponentParts, ComponentSender, RelmWidgetExt,
    binding::{Binding, BoolBinding},
};
use std::sync::Arc;

pub struct PowerStatesDialog {
    domain: ClockDomain,
    core_states_list: relm4::Controller<PowerStatesList>,
    vram_states_list: relm4::Controller<PowerStatesList>,
    states_configurable: BoolBinding,
    states_configuration_enabled: BoolBinding,
    performance_level: Option<PerformanceLevel>,
    configured_signal: SignalHandlerId,
    vram_clock_ratio: f64,
    has_core_states: bool,
    has_vram_states: bool,
}

#[derive(Debug)]
pub enum PowerStatesDialogMsg {
    Show {
        domain: ClockDomain,
        parent: gtk::Widget,
    },
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
impl relm4::Component for PowerStatesDialog {
    type Init = ();
    type Input = PowerStatesDialogMsg;
    type Output = OcPageMsg;
    type CommandOutput = ();

    view! {
        adw::Dialog {
            set_content_width: 420,
            set_follows_content_size: true,
            #[watch]
            set_title: &match model.domain {
                ClockDomain::Gpu => fl!(I18N, "gpu-pstates"),
                ClockDomain::Vram => fl!(I18N, "vram-pstates"),
            },

            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {},

                #[wrap(Some)]
                set_content = &gtk::ScrolledWindow {
                    set_hscrollbar_policy: gtk::PolicyType::Never,
                    set_propagate_natural_height: true,
                    set_max_content_height: 500,

                    PageSection {
                        set_hide_visible_container: true,
                        set_margin_all: 15,
                        add_css_class: "power-states-dialog-card",

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
                                    set_label: &fl!(I18N, "enable-pstate-config"),

                                    #[watch]
                                    #[block_signal(configured_toggled_handler)]
                                    set_active: model.states_configuration_enabled.value(),

                                    connect_toggled[sender] => move |button| {
                                        sender.input(PowerStatesDialogMsg::ConfiguredToggled {
                                            configured: button.is_active(),
                                        });
                                    } @ configured_toggled_handler,
                                },
                            },

                            #[template_child]
                            content {
                                gtk::ListBoxRow {
                                    set_activatable: false,
                                    set_selectable: false,
                                    #[watch]
                                    set_visible: model.domain == ClockDomain::Gpu && model.has_core_states,
                                    #[wrap(Some)]
                                    set_child = model.core_states_list.widget(),
                                },

                                gtk::ListBoxRow {
                                    set_activatable: false,
                                    set_selectable: false,
                                    #[watch]
                                    set_visible: model.domain == ClockDomain::Vram && model.has_vram_states,
                                    #[wrap(Some)]
                                    set_child = model.vram_states_list.widget(),
                                },
                            },
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
            value_suffix: fl!(I18N, "mhz"),
        });
        let vram_states_list = PowerStatesList::detach(PowerStatesListOptions {
            value_suffix: fl!(I18N, "mhz"),
        });

        let states_configuration_enabled = BoolBinding::new(false);

        let configured_sender = sender.clone();
        let configured_signal =
            states_configuration_enabled.connect_value_notify(move |states_configured| {
                configured_sender.input(PowerStatesDialogMsg::InternalConfigurableChanged(
                    states_configured.get(),
                ));
                APP_BROKER.send(AppMsg::SettingsChanged);
            });

        let model = Self {
            domain: ClockDomain::Gpu,
            core_states_list,
            vram_states_list,
            states_configurable: BoolBinding::new(false),
            states_configuration_enabled,
            configured_signal,
            performance_level: None,
            vram_clock_ratio: 1.0,
            has_core_states: false,
            has_vram_states: false,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match msg {
            PowerStatesDialogMsg::Show { domain, parent } => {
                self.domain = domain;
                self.update_view(widgets, sender.clone());
                root.present(Some(&parent));
            }
            PowerStatesDialogMsg::PowerStates {
                pstates,
                configured,
            } => {
                // Child updates are queued; use the received states for availability
                self.has_core_states = !pstates.core.is_empty();
                self.has_vram_states = !pstates.vram.is_empty();
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
            PowerStatesDialogMsg::Stats(stats) => {
                self.core_states_list.emit(PowerStatesListMsg::ActiveState(
                    stats.active_power_states.and_then(|states| states.core),
                ));
                self.vram_states_list.emit(PowerStatesListMsg::ActiveState(
                    stats.active_power_states.and_then(|states| states.memory),
                ));
            }
            PowerStatesDialogMsg::VramClockRatio(ratio) => {
                self.vram_clock_ratio = ratio;
                self.vram_states_list
                    .emit(PowerStatesListMsg::ValueRatio(ratio));
            }
            PowerStatesDialogMsg::Configurable(is_plvl_manual) => {
                let configurable = is_plvl_manual && (self.has_core_states || self.has_vram_states);
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
            PowerStatesDialogMsg::PerformanceLevel(level) => {
                self.performance_level = level;
            }
            PowerStatesDialogMsg::ConfiguredToggled { configured } => {
                if !configured || self.performance_level == Some(PerformanceLevel::Manual) {
                    self.states_configuration_enabled.set_value(configured);
                } else {
                    APP_BROKER.send(AppMsg::EnablePstateConfig);
                }
            }
            PowerStatesDialogMsg::EnableWithManualPerformanceLevel => {
                sender
                    .output(OcPageMsg::SetPerformanceLevel(PerformanceLevel::Manual))
                    .unwrap();
                self.states_configuration_enabled.set_value(true);
            }
            PowerStatesDialogMsg::InternalConfigurableChanged(configurable) => {
                self.core_states_list
                    .emit(PowerStatesListMsg::Configurable(configurable));
                self.vram_states_list
                    .emit(PowerStatesListMsg::Configurable(configurable));
            }
        }
        self.update_view(widgets, sender);
    }
}

impl PowerStatesDialog {
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

#[cfg(all(test, feature = "gtk-tests"))]
mod tests {
    use super::*;
    use amdgpu_sysfs::gpu_handle::PowerLevelId;
    use lact_schema::PowerState;

    #[test]
    #[ignore = "requires a GTK display; run explicitly with --ignored"]
    fn restores_configured_states_before_child_lists_update() {
        adw::init().unwrap();
        let context = gtk::glib::MainContext::default();
        let _guard = context.acquire().unwrap();
        let dialog = PowerStatesDialog::detach(());
        let states = PowerStates {
            core: vec![],
            vram: (0..4)
                .map(|index| PowerState {
                    enabled: index == 3,
                    min_value: None,
                    value: 100 * u64::from(index + 1),
                    id: Some(PowerLevelId::Index(index)),
                })
                .collect(),
        };

        // Queue the startup messages together, before child components can update
        dialog.emit(PowerStatesDialogMsg::PowerStates {
            pstates: states,
            configured: true,
        });
        dialog.emit(PowerStatesDialogMsg::Configurable(true));
        while context.pending() {
            context.iteration(false);
        }
        assert!(dialog.model().states_configuration_enabled.value());
        assert_eq!(
            dialog.model().get_enabled_power_states()[&PowerLevelKind::MemoryClock],
            vec![3],
        );

        // Changing to a GPU without power states must clear the previous capability
        dialog.emit(PowerStatesDialogMsg::PowerStates {
            pstates: PowerStates::default(),
            configured: false,
        });
        dialog.emit(PowerStatesDialogMsg::Configurable(true));
        while context.pending() {
            context.iteration(false);
        }
        assert!(!dialog.model().states_configurable.value());
        assert!(dialog.model().get_enabled_power_states().is_empty());
    }
}
