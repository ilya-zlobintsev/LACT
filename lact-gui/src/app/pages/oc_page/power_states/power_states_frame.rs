use super::power_states_list::PowerStatesList;
use crate::{
    app::{
        msg::AppMsg,
        pages::oc_page::power_states::power_states_list::{
            PowerStatesListMsg, PowerStatesListOptions,
        },
    },
    APP_BROKER, I18N,
};
use amdgpu_sysfs::gpu_handle::{PerformanceLevel, PowerLevelKind};
use gtk::{
    glib::{object::ObjectExt, SignalHandlerId},
    prelude::{BoxExt, CheckButtonExt, OrientableExt, WidgetExt},
};
use i18n_embed_fl::fl;
use indexmap::IndexMap;
use lact_schema::{DeviceStats, PowerStates};
use relm4::{
    binding::BoolBinding, Component, ComponentController, ComponentParts, ComponentSender,
    RelmObjectExt, RelmWidgetExt,
};
use std::sync::Arc;

pub struct PowerStatesFrame {
    core_states_list: relm4::Controller<PowerStatesList>,
    vram_states_list: relm4::Controller<PowerStatesList>,
    states_configurable: BoolBinding,
    states_configured: BoolBinding,
    states_expanded: BoolBinding,
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
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for PowerStatesFrame {
    type Init = ();
    type Input = PowerStatesFrameMsg;
    type Output = ();

    view! {
        gtk::Expander {
            set_label: Some(&fl!(I18N, "pstates")),
            add_binding: (&model.states_expanded, "expanded"),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 10,
                set_spacing: 5,
                add_binding: (&model.states_configurable, "sensitive"),

                gtk::Label {
                    set_label: &fl!(I18N, "pstates-manual-needed"),
                    set_margin_horizontal: 10,
                    set_halign: gtk::Align::Start,
                    #[watch]
                    set_visible: model.performance_level.is_some_and(|level| level != PerformanceLevel::Manual),
                },

                gtk::CheckButton {
                    set_label: Some(&fl!(I18N, "enable-pstate-config")),
                    add_binding: (&model.states_configured, "active"),
                    #[watch]
                    set_visible: model.performance_level.is_some(),
                },

                gtk::Box {
                    set_spacing: 10,
                    set_orientation: gtk::Orientation::Horizontal,
                    add_binding: (&model.states_configured, "sensitive"),

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
                title: fl!(I18N, "gpu-pstates"),
                value_suffix: fl!(I18N, "mhz"),
            })
            .detach();
        let vram_states_list = PowerStatesList::builder()
            .launch(PowerStatesListOptions {
                title: fl!(I18N, "vram-pstates"),
                value_suffix: fl!(I18N, "mhz"),
            })
            .detach();

        let states_configured = BoolBinding::new(false);

        let configured_signal = states_configured.connect_value_notify(|_| {
            APP_BROKER.send(AppMsg::SettingsChanged);
        });

        let model = Self {
            core_states_list,
            vram_states_list,
            states_configurable: BoolBinding::new(false),
            states_configured,
            configured_signal,
            states_expanded: BoolBinding::new(false),
            performance_level: None,
            vram_clock_ratio: 1.0,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            PowerStatesFrameMsg::PowerStates {
                pstates,
                configured,
            } => {
                self.states_configured.block_signal(&self.configured_signal);
                self.states_configured.set_value(configured);
                self.states_configured
                    .unblock_signal(&self.configured_signal);

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
                let value = configurable
                    && (!self.core_states_list.model().is_empty()
                        || !self.vram_states_list.model().is_empty());
                self.states_configurable.set_value(value);

                if !value {
                    self.states_configured.block_signal(&self.configured_signal);
                    self.states_configured.set_value(false);
                    self.states_configured
                        .unblock_signal(&self.configured_signal);
                }
            }
            PowerStatesFrameMsg::PerformanceLevel(level) => {
                self.performance_level = level;
            }
        }
    }
}

impl PowerStatesFrame {
    pub fn get_enabled_power_states(&self) -> IndexMap<PowerLevelKind, Vec<u8>> {
        if self.states_configured.value() {
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
