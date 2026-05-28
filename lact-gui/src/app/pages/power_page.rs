mod performance_frame;
mod power_cap_section;
mod power_states;
mod stats_section;

use super::PageUpdate;
use crate::app::ext::RelmDefaultLauchable;
use adw::prelude::*;
use amdgpu_sysfs::gpu_handle::{
    PerformanceLevel, PowerLevelKind, power_profile_mode::PowerProfileModesTable,
};
use indexmap::IndexMap;
use lact_schema::{PowerStates, config};
use performance_frame::{PerformanceFrame, PerformanceFrameMsg};
use power_cap_section::{PowerCapMsg, PowerCapSection};
use power_states::power_states_frame::{PowerStatesFrame, PowerStatesFrameMsg};
use relm4::{ComponentController, ComponentParts, ComponentSender, RelmWidgetExt};
use stats_section::{PowerStatsSection, PowerStatsSectionMsg};

pub struct PowerPage {
    stats_section: relm4::Controller<PowerStatsSection>,
    performance_frame: relm4::Controller<PerformanceFrame>,
    power_cap_section: relm4::Controller<PowerCapSection>,
    power_states_frame: relm4::Controller<PowerStatesFrame>,
}

#[derive(Debug)]
pub enum PowerPageMsg {
    Update {
        update: PageUpdate,
        initial: bool,
    },
    ProfileModesTable(Option<PowerProfileModesTable>),
    PowerStates {
        pstates: PowerStates,
        configured: bool,
    },
    PerformanceLevelChanged,
}

#[relm4::component(pub)]
impl relm4::Component for PowerPage {
    type Init = ();
    type Input = PowerPageMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,
            set_margin_all: 15,
            set_margin_top: 20,

            model.stats_section.widget(),
            model.power_cap_section.widget(),
            model.performance_frame.widget(),
            model.power_states_frame.widget(),
        },
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let power_cap_section = PowerCapSection::detach_default();
        let power_states_frame = PowerStatesFrame::detach_default();
        let stats_section = PowerStatsSection::detach_default();
        let performance_frame =
            PerformanceFrame::launch_default().forward(sender.input_sender(), |msg| msg);

        let model = Self {
            stats_section,
            performance_frame,
            power_cap_section,
            power_states_frame,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            PowerPageMsg::Update { update, initial } => match &update {
                PageUpdate::Stats(stats) => {
                    self.stats_section
                        .emit(PowerStatsSectionMsg::Stats(stats.clone()));
                    self.power_states_frame
                        .emit(PowerStatesFrameMsg::Stats(stats.clone()));

                    if initial {
                        self.power_cap_section
                            .emit(PowerCapMsg::Update(update.clone()));

                        self.power_cap_section
                            .widget()
                            .set_visible(stats.power.cap_current.is_some());

                        self.performance_frame
                            .emit(PerformanceFrameMsg::PerformanceLevel(
                                stats.performance_level,
                            ));
                        sender.input(PowerPageMsg::PerformanceLevelChanged);
                    }
                }
                PageUpdate::Info(info) => {
                    self.power_states_frame
                        .emit(PowerStatesFrameMsg::VramClockRatio(info.vram_clock_ratio()));
                }
            },
            PowerPageMsg::ProfileModesTable(modes_table) => {
                self.performance_frame
                    .emit(PerformanceFrameMsg::PowerProfileModes(modes_table));
            }
            PowerPageMsg::PowerStates {
                pstates,
                configured,
            } => {
                self.power_states_frame
                    .emit(PowerStatesFrameMsg::PowerStates {
                        pstates,
                        configured,
                    });
                sender.input(PowerPageMsg::PerformanceLevelChanged);
            }
            PowerPageMsg::PerformanceLevelChanged => {
                let custom_pstates_configurable =
                    self.get_performance_level() == Some(PerformanceLevel::Manual);
                self.power_states_frame
                    .emit(PowerStatesFrameMsg::Configurable(
                        custom_pstates_configurable,
                    ));

                self.power_states_frame
                    .emit(PowerStatesFrameMsg::PerformanceLevel(
                        self.get_performance_level(),
                    ));
            }
        }

        self.update_view(widgets, sender);
    }
}

impl PowerPage {
    pub fn get_performance_level(&self) -> Option<PerformanceLevel> {
        self.performance_frame.model().performance_level()
    }

    pub fn get_power_profile_mode(&self) -> Option<u16> {
        self.performance_frame.model().power_profile_mode()
    }

    pub fn get_power_profile_mode_custom_heuristics(&self) -> Vec<Vec<Option<i32>>> {
        self.performance_frame
            .model()
            .power_profile_mode_custom_heuristics()
    }

    pub fn get_power_cap(&self) -> Option<f64> {
        self.power_cap_section.model().get_user_cap()
    }

    pub fn apply_config(&self, config: &mut config::GpuConfig) {
        if let Some(cap) = self.get_power_cap() {
            config.power_cap = Some(cap);
        }

        if let Some(level) = self.get_performance_level() {
            config.performance_level = Some(level);
            config.power_profile_mode_index = self.get_power_profile_mode();
            config.custom_power_profile_mode_hueristics =
                self.get_power_profile_mode_custom_heuristics();
        }

        config.power_states = self.get_enabled_power_states();
    }

    fn get_enabled_power_states(&self) -> IndexMap<PowerLevelKind, Vec<u8>> {
        if self.performance_frame.model().performance_level() == Some(PerformanceLevel::Manual) {
            self.power_states_frame.model().get_enabled_power_states()
        } else {
            IndexMap::new()
        }
    }
}
