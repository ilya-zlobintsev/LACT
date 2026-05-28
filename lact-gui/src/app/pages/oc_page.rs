mod clocks_frame;
mod vf_curve;

use crate::app::pages::PageUpdate;
use crate::app::pages::gpu_stats_section::{
    GpuStat, GpuStatsSection, GpuStatsSectionConfig, GpuStatsSectionMsg,
};
use crate::app::utils::ext::RelmLaunchable as _;
use crate::app::{msg::AppMsg, utils::ext::RelmDefaultLauchable};
use clocks_frame::{ClocksFrame, ClocksFrameMsg};
use gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
use lact_schema::config;
use lact_schema::{ClocksTable, DeviceInfo, PowerStates};
use relm4::binding::BoolBinding;
use relm4::{ComponentController, ComponentParts, ComponentSender, RelmWidgetExt};
use std::sync::Arc;
use tracing::debug;
use vf_curve::{VfCurveEditor, VfCurveEditorMsg};

pub struct OcPage {
    stats_section: relm4::Controller<GpuStatsSection>,
    device_info: Option<Arc<DeviceInfo>>,

    clocks_frame: relm4::Controller<ClocksFrame>,

    vf_curve_editor: relm4::Controller<VfCurveEditor>,
}

#[derive(Debug)]
pub enum OcPageMsg {
    Update { update: PageUpdate, initial: bool },
    ClocksTable(Option<ClocksTable>),
    PowerStates { pstates: PowerStates },
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
            set_margin_all: 15,
            set_margin_top: 20, // align with gpu picker

            model.stats_section.widget(),
            model.clocks_frame.widget(),
        },
    }

    fn init(
        settings_changed: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let stats_section = GpuStatsSection::detach(GpuStatsSectionConfig {
            stats: vec![
                GpuStat::DeviceName,
                GpuStat::Throttling,
                GpuStat::GpuClockTarget,
                GpuStat::GpuVoltage,
                GpuStat::Temperature,
                GpuStat::GpuClock,
                GpuStat::VramClock,
                GpuStat::GpuUsage,
                GpuStat::VramUsage,
                GpuStat::PowerUsage,
                GpuStat::FanSpeed,
            ],
        });
        let clocks_frame = ClocksFrame::launch_default().forward(sender.input_sender(), |msg| msg);

        let vf_curve_editor = VfCurveEditor::detach(settings_changed);

        let model = Self {
            stats_section,
            device_info: None,
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
            OcPageMsg::Update {
                update,
                initial: _initial,
            } => match &update {
                PageUpdate::Stats(stats) => {
                    self.stats_section
                        .emit(GpuStatsSectionMsg::Stats(stats.clone()));

                    self.vf_curve_editor
                        .emit(VfCurveEditorMsg::Stats(stats.clone()));
                }
                PageUpdate::Info(info) => {
                    let vram_clock_ratio = info.vram_clock_ratio();

                    self.device_info = Some(info.clone());
                    self.stats_section
                        .emit(GpuStatsSectionMsg::Info(info.clone()));
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
            OcPageMsg::PowerStates { pstates } => {
                self.stats_section
                    .emit(GpuStatsSectionMsg::PowerStates(Arc::new(pstates)));
            }
            OcPageMsg::ShowVfCurveEditor => {
                self.vf_curve_editor.emit(VfCurveEditorMsg::Show);
            }
        }

        self.update_view(widgets, sender);
    }
}

impl OcPage {
    pub fn apply_clocks_config(&self, config: &mut config::ClocksConfiguration) {
        let commands = self.clocks_frame.model().get_commands();

        debug!("applying clocks commands {commands:#?}");

        for command in commands {
            config.apply_clocks_command(&command);
        }

        if !self.vf_curve_editor.model().is_empty() {
            config.gpu_vf_curve = self.vf_curve_editor.model().get_configured_curve();
        }
    }
}
