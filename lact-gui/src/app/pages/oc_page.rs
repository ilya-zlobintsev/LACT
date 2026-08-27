mod clocks_frame;
mod performance_frame;
mod power_frame;
mod power_states;
mod vf_curve;

use crate::APP_BROKER;
use crate::app::components::gpu_stats_section::{
    GpuStat, GpuStatsSection, GpuStatsSectionConfig, GpuStatsSectionMsg,
};
use crate::app::pages::PageUpdate;
use crate::app::utils::ext::RelmLaunchable as _;
use crate::app::{msg::AppMsg, utils::ext::RelmDefaultLauchable};
use adw::prelude::*;
use amdgpu_sysfs::gpu_handle::{
    PerformanceLevel, PowerLevelKind, power_profile_mode::PowerProfileModesTable,
};
use clocks_frame::{ClockDomain, ClocksFrame, ClocksFrameInit, ClocksFrameMsg};
use indexmap::IndexMap;
use lact_schema::config;
use lact_schema::{ClocksTable, DeviceInfo, PowerStates};
use nvml_wrapper::enums::device::PowerMizerMode;
use performance_frame::PerformanceFrameMsg;
use power_frame::{PowerFrame, PowerFrameMsg};
use power_states::power_states_frame::{PowerStatesFrame, PowerStatesFrameMsg};
use relm4::binding::BoolBinding;
use relm4::{ComponentController, ComponentParts, ComponentSender, RelmWidgetExt};
use std::collections::HashSet;
use std::sync::Arc;
use tracing::debug;
use vf_curve::{VfCurveEditor, VfCurveEditorInit, VfCurveEditorMsg};

pub struct OcPage {
    stats_section: relm4::Controller<GpuStatsSection>,
    device_info: Option<Arc<DeviceInfo>>,

    power_frame: relm4::Controller<PowerFrame>,
    power_states_frame: relm4::Controller<PowerStatesFrame>,
    gpu_clocks_frame: relm4::Controller<ClocksFrame>,
    vram_clocks_frame: relm4::Controller<ClocksFrame>,

    vf_curve_editor: relm4::Controller<VfCurveEditor>,
}

#[derive(Debug)]
pub enum OcPageMsg {
    Update {
        update: PageUpdate,
        initial: bool,
    },
    ClocksTable {
        table: Option<ClocksTable>,
        vf_curve_is_configured: bool,
    },
    ProfileModesTable(Option<PowerProfileModesTable>),
    PowerStates {
        pstates: PowerStates,
        configured: bool,
    },
    PerformanceLevelChanged,
    SetPerformanceLevel(PerformanceLevel),
    ShowVfCurveEditor,
    VfCurveEditingToggled(bool),
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
            set_margin_all: 15,
            set_margin_top: 20, // align with gpu picker

            model.stats_section.widget(),

            gtk::FlowBox {
                set_orientation: gtk::Orientation::Horizontal,
                set_selection_mode: gtk::SelectionMode::None,
                set_min_children_per_line: 1,
                set_max_children_per_line: 2,
                set_column_spacing: 10,
                set_row_spacing: 10,
                set_homogeneous: false,
                set_valign: gtk::Align::Start,
                set_hexpand: true,

                append: gpu_clocks_child = &gtk::FlowBoxChild {
                    add_css_class: "oc-page-section",
                    set_valign: gtk::Align::Start,
                    set_hexpand: true,

                    model.gpu_clocks_frame.widget(),
                },

                append: vram_clocks_child = &gtk::FlowBoxChild {
                    add_css_class: "oc-page-section",
                    set_valign: gtk::Align::Start,
                    set_hexpand: true,

                    model.vram_clocks_frame.widget(),
                },

                append: power_child = &gtk::FlowBoxChild {
                    add_css_class: "oc-page-section",
                    set_valign: gtk::Align::Start,
                    set_hexpand: true,

                    model.power_frame.widget(),
                },

                append: power_states_child = &gtk::FlowBoxChild {
                    add_css_class: "oc-page-section",
                    set_valign: gtk::Align::Start,
                    set_hexpand: true,

                    model.power_states_frame.widget(),
                },
            },
        },
    }

    fn init(
        settings_changed: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let stats_section = GpuStatsSection::detach(GpuStatsSectionConfig {
            stats: HashSet::from([
                GpuStat::DeviceName,
                GpuStat::Throttling,
                GpuStat::GpuClockTarget,
                GpuStat::GpuVoltage,
                GpuStat::Temperature,
                GpuStat::GpuClock,
                GpuStat::VramClock,
                GpuStat::GpuUsage,
                GpuStat::VramUsage,
                GpuStat::GttUsage,
                GpuStat::PowerUsage,
                GpuStat::FanSpeed,
                GpuStat::ExtraClocks,
            ]),
        });
        let vf_curve_editing = BoolBinding::new(false);
        let gpu_clocks_frame = ClocksFrame::launch(ClocksFrameInit {
            domain: ClockDomain::Gpu,
            vf_curve_editing: vf_curve_editing.clone(),
            show_all_pstates: BoolBinding::new(false),
        })
        .forward(sender.input_sender(), |msg| msg);
        let vram_clocks_frame = ClocksFrame::launch(ClocksFrameInit {
            domain: ClockDomain::Vram,
            vf_curve_editing: BoolBinding::new(false),
            show_all_pstates: BoolBinding::new(false),
        })
        .forward(sender.input_sender(), |msg| msg);
        let power_states_frame =
            PowerStatesFrame::launch_default().forward(sender.input_sender(), |msg| msg);
        let power_frame = PowerFrame::launch_default().forward(sender.input_sender(), |msg| msg);

        let vf_curve_editor = VfCurveEditor::detach(VfCurveEditorInit {
            global_settings_changed: settings_changed,
            allow_editing: vf_curve_editing,
        });

        let model = Self {
            stats_section,
            device_info: None,
            power_frame,
            power_states_frame,
            gpu_clocks_frame,
            vram_clocks_frame,
            vf_curve_editor,
        };

        let widgets = view_output!();

        let section_size_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);
        section_size_group.add_widget(&widgets.gpu_clocks_child);
        section_size_group.add_widget(&widgets.vram_clocks_child);
        section_size_group.add_widget(&widgets.power_child);
        section_size_group.add_widget(&widgets.power_states_child);

        model
            .gpu_clocks_frame
            .widget()
            .bind_property("visible", &widgets.gpu_clocks_child, "visible")
            .sync_create()
            .build();
        model
            .vram_clocks_frame
            .widget()
            .bind_property("visible", &widgets.vram_clocks_child, "visible")
            .sync_create()
            .build();
        model
            .power_frame
            .widget()
            .bind_property("visible", &widgets.power_child, "visible")
            .sync_create()
            .build();
        model
            .power_states_frame
            .widget()
            .bind_property("visible", &widgets.power_states_child, "visible")
            .sync_create()
            .build();

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
                        self.power_frame
                            .emit(PowerFrameMsg::PowerStats(stats.power.clone()));
                        self.power_frame.emit(PowerFrameMsg::Performance(
                            PerformanceFrameMsg::PerformanceLevel(stats.performance_level),
                        ));
                        self.power_frame.emit(PowerFrameMsg::Performance(
                            PerformanceFrameMsg::PowerMizerInfo {
                                active: stats.active_power_mizer_mode,
                                supported: stats.supported_power_mizer_modes.clone(),
                            },
                        ));
                    }
                }
                PageUpdate::Info(info) => {
                    let vram_clock_ratio = info.vram_clock_ratio();

                    self.device_info = Some(info.clone());
                    self.stats_section
                        .emit(GpuStatsSectionMsg::Info(info.clone()));
                    self.power_states_frame
                        .emit(PowerStatesFrameMsg::VramClockRatio(vram_clock_ratio));
                    self.vram_clocks_frame
                        .emit(ClocksFrameMsg::VramRatio(vram_clock_ratio));
                }
            },
            OcPageMsg::ClocksTable {
                table,
                vf_curve_is_configured,
            } => {
                let table = table.map(Arc::new);

                self.gpu_clocks_frame.emit(ClocksFrameMsg::Clocks {
                    table: table.clone(),
                    vf_curve_is_configured,
                });
                self.vram_clocks_frame.emit(ClocksFrameMsg::Clocks {
                    table: table.clone(),
                    vf_curve_is_configured,
                });
                self.vf_curve_editor
                    .emit(VfCurveEditorMsg::Clocks(table.clone()));
            }
            OcPageMsg::ProfileModesTable(modes_table) => {
                self.power_frame.emit(PowerFrameMsg::Performance(
                    PerformanceFrameMsg::PowerProfileModes(modes_table),
                ));
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
            OcPageMsg::SetPerformanceLevel(level) => {
                self.power_frame.emit(PowerFrameMsg::Performance(
                    PerformanceFrameMsg::PerformanceLevel(Some(level)),
                ));
                APP_BROKER.send(AppMsg::SettingsChanged);
            }
            OcPageMsg::ShowVfCurveEditor => {
                self.vf_curve_editor.emit(VfCurveEditorMsg::Show);
            }
            OcPageMsg::VfCurveEditingToggled(enabled) => {
                if enabled {
                    self.gpu_clocks_frame
                        .emit(ClocksFrameMsg::ResetGpuClockOffsets);
                } else {
                    self.vf_curve_editor.emit(VfCurveEditorMsg::ResetCurve);
                }
            }
        }

        self.update_view(widgets, sender);
    }
}

impl OcPage {
    pub fn get_performance_level(&self) -> Option<PerformanceLevel> {
        self.power_frame.model().performance_level()
    }

    pub fn get_active_power_mizer_mode(&self) -> Option<PowerMizerMode> {
        self.power_frame.model().active_power_mizer_mode()
    }

    pub fn get_power_profile_mode(&self) -> Option<u16> {
        self.power_frame.model().power_profile_mode()
    }

    pub fn get_power_profile_mode_custom_heuristics(&self) -> Vec<Vec<Option<i32>>> {
        self.power_frame
            .model()
            .power_profile_mode_custom_heuristics()
    }

    pub fn get_power_cap(&self) -> Option<f64> {
        self.power_frame.model().get_user_cap()
    }

    pub fn apply_clocks_config(&self, config: &mut config::ClocksConfiguration) {
        let mut commands = self.gpu_clocks_frame.model().get_commands();
        commands.extend(self.vram_clocks_frame.model().get_commands());

        debug!("applying clocks commands {commands:#?}");

        for command in commands {
            config.apply_clocks_command(&command);
        }

        if !self.vf_curve_editor.model().is_empty() {
            config.nvidia_gpu_vf_curve = self.vf_curve_editor.model().get_configured_curve();
        }
    }

    pub fn get_enabled_power_states(&self) -> IndexMap<PowerLevelKind, Vec<u8>> {
        if self.get_performance_level() == Some(PerformanceLevel::Manual) {
            self.power_states_frame.model().get_enabled_power_states()
        } else {
            IndexMap::new()
        }
    }
}
