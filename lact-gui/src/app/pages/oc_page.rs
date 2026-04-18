mod clocks_frame;
pub mod gpu_stats_section;
mod performance_frame;
mod power_cap_section;
mod power_states;
mod vf_curve;

use super::PageUpdate;
use crate::app::pages::oc_page::gpu_stats_section::GpuStatsSectionMsg;
use crate::app::pages::oc_page::vf_curve::{VfCurveEditor, VfCurveEditorMsg};
use crate::app::{ext::RelmDefaultLauchable, msg::AppMsg};
use amdgpu_sysfs::gpu_handle::{
    PerformanceLevel, PowerLevelKind, power_profile_mode::PowerProfileModesTable,
};
use clocks_frame::{ClocksFrame, ClocksFrameMsg};
use gpu_stats_section::GpuStatsSection;
use gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
use indexmap::IndexMap;
use lact_schema::config;
use lact_schema::{ClocksTable, DeviceInfo, PowerStates};
use performance_frame::{PerformanceFrame, PerformanceFrameMsg};
use power_cap_section::{PowerCapMsg, PowerCapSection};
use power_states::power_states_frame::{PowerStatesFrame, PowerStatesFrameMsg};
use relm4::binding::BoolBinding;
use relm4::{ComponentController, ComponentParts, ComponentSender, RelmWidgetExt};
use std::sync::Arc;
use tracing::debug;

pub struct OcPage {
    stats_section: relm4::Controller<GpuStatsSection>,
    device_info: Option<Arc<DeviceInfo>>,

    performance_frame: relm4::Controller<PerformanceFrame>,
    power_cap_section: relm4::Controller<PowerCapSection>,
    power_states_frame: relm4::Controller<PowerStatesFrame>,
    clocks_frame: relm4::Controller<ClocksFrame>,

    vf_curve_editor: relm4::Controller<VfCurveEditor>,
}

#[derive(Debug)]
pub enum OcPageMsg {
    Update {
        update: PageUpdate,
        initial: bool,
    },
    ClocksTable(Option<ClocksTable>),
    ProfileModesTable(Option<PowerProfileModesTable>),
    PowerStates {
        pstates: PowerStates,
        configured: bool,
    },
    PerformanceLevelChanged,
    ShowVfCurveEditor,
}

#[relm4::component(pub)]
impl relm4::Component for OcPage {
    type Init = BoolBinding;
    type Input = OcPageMsg;
    type Output = AppMsg;
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,
            set_margin_horizontal: 30,
            set_margin_top: 15,
            set_margin_bottom: 60,

            model.stats_section.widget(),
            model.power_cap_section.widget(),
            model.performance_frame.widget(),
            model.power_states_frame.widget(),
            model.clocks_frame.widget(),
        },
    }

    fn init(
        settings_changed: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let stats_section = GpuStatsSection::detach_default();
        let power_cap_section = PowerCapSection::detach_default();
        let clocks_frame = ClocksFrame::launch_default().forward(sender.input_sender(), |msg| msg);
        let power_states_frame = PowerStatesFrame::detach_default();
        let performance_frame =
            PerformanceFrame::launch_default().forward(sender.input_sender(), |msg| msg);

        let vf_curve_editor = VfCurveEditor::builder().launch(settings_changed).detach();

        let model = Self {
            stats_section,
            device_info: None,
            performance_frame,
            power_cap_section,
            power_states_frame,
            clocks_frame,
            vf_curve_editor,
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
            OcPageMsg::Update { update, initial } => match &update {
                PageUpdate::Stats(stats) => {
                    self.power_states_frame
                        .emit(PowerStatesFrameMsg::Stats(stats.clone()));

                    self.stats_section
                        .emit(GpuStatsSectionMsg::Stats(stats.clone()));

                    self.vf_curve_editor
                        .emit(VfCurveEditorMsg::Stats(stats.clone()));

                    if initial {
                        self.power_cap_section
                            .emit(PowerCapMsg::Update(update.clone()));

                        if stats.power.cap_current.is_some() {
                            self.power_cap_section.widget().set_visible(true);
                        } else {
                            self.power_cap_section.widget().set_visible(false);
                        }

                        self.performance_frame
                            .emit(PerformanceFrameMsg::PerformanceLevel(
                                stats.performance_level,
                            ));
                        sender.input(OcPageMsg::PerformanceLevelChanged);
                    }
                }
                PageUpdate::Info(info) => {
                    let vram_clock_ratio = info.vram_clock_ratio();

                    self.device_info = Some(info.clone());
                    self.stats_section
                        .emit(GpuStatsSectionMsg::Info(info.clone()));
                    self.power_states_frame
                        .emit(PowerStatesFrameMsg::VramClockRatio(vram_clock_ratio));
                    self.clocks_frame
                        .emit(ClocksFrameMsg::VramRatio(vram_clock_ratio));
                }
            },
            OcPageMsg::ClocksTable(table) => {
                let table = table.map(Arc::new);

                self.clocks_frame
                    .emit(ClocksFrameMsg::Clocks(table.clone()));
                self.vf_curve_editor
                    .emit(VfCurveEditorMsg::Clocks(table.clone()));
            }
            OcPageMsg::ProfileModesTable(modes_table) => {
                self.performance_frame
                    .emit(PerformanceFrameMsg::PowerProfileModes(modes_table));
            }
            OcPageMsg::PowerStates {
                pstates,
                configured,
            } => {
                self.power_states_frame
                    .emit(PowerStatesFrameMsg::PowerStates {
                        pstates: pstates.clone(),
                        configured,
                    });
                self.stats_section
                    .emit(GpuStatsSectionMsg::PowerStates(Arc::new(pstates)));
                sender.input(OcPageMsg::PerformanceLevelChanged);
            }
            OcPageMsg::PerformanceLevelChanged => {
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
            OcPageMsg::ShowVfCurveEditor => {
                self.vf_curve_editor.emit(VfCurveEditorMsg::Show);
            }
        }

        self.update_view(widgets, sender);
    }
}

impl OcPage {
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

    pub fn apply_clocks_config(&self, config: &mut config::ClocksConfiguration) {
        let commands = self.clocks_frame.model().get_commands();

        debug!("applying clocks commands {commands:#?}");

        for command in commands {
            config.apply_clocks_command(&command);
        }

        config.gpu_vf_curve = self.vf_curve_editor.model().get_configured_curve();
    }

    pub fn get_enabled_power_states(&self) -> IndexMap<PowerLevelKind, Vec<u8>> {
        if self.performance_frame.model().performance_level() == Some(PerformanceLevel::Manual) {
            self.power_states_frame.model().get_enabled_power_states()
        } else {
            IndexMap::new()
        }
    }
}
