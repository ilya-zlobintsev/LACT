mod clocks_frame;
pub mod gpu_stats_section;
mod performance_frame;
mod power_cap_section;
mod power_states;

use super::PageUpdate;
use crate::{
    app::{ext::RelmDefaultLauchable, msg::AppMsg},
    I18N,
};
use amdgpu_sysfs::gpu_handle::{
    power_profile_mode::PowerProfileModesTable, PerformanceLevel, PowerLevelKind,
};
use clocks_frame::{ClocksFrame, ClocksFrameMsg};
use gpu_stats_section::GpuStatsSection;
use gtk::{
    pango,
    prelude::{BoxExt, ButtonExt, FrameExt, OrientableExt, WidgetExt},
};
use i18n_embed_fl::fl;
use indexmap::IndexMap;
use lact_daemon::BASE_MODULE_CONF_PATH;
use lact_schema::{request::SetClocksCommand, ClocksTable, DeviceInfo, PowerStates, SystemInfo};
use performance_frame::{PerformanceFrame, PerformanceFrameMsg};
use power_cap_section::{PowerCapMsg, PowerCapSection};
use power_states::power_states_frame::{PowerStatesFrame, PowerStatesFrameMsg};
use relm4::{ComponentController, ComponentParts, ComponentSender, RelmWidgetExt};
use std::sync::Arc;

pub struct OcPage {
    stats_section: relm4::Controller<GpuStatsSection>,
    system_info: SystemInfo,
    device_info: Option<Arc<DeviceInfo>>,

    performance_frame: relm4::Controller<PerformanceFrame>,
    power_cap_section: relm4::Controller<PowerCapSection>,
    power_states_frame: relm4::Controller<PowerStatesFrame>,
    clocks_frame: relm4::Controller<ClocksFrame>,
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
}

#[relm4::component(pub)]
impl relm4::Component for OcPage {
    type Init = SystemInfo;
    type Input = OcPageMsg;
    type Output = AppMsg;
    type CommandOutput = ();

    view! {
        gtk::ScrolledWindow {
            set_hscrollbar_policy: gtk::PolicyType::Never,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 15,
                set_margin_horizontal: 20,

                gtk::Frame {
                    #[watch]
                    set_visible: model.system_info.amdgpu_overdrive_enabled == Some(false) && model.device_info.as_ref().is_some_and(|info| info.driver == "amdgpu"),
                    set_label_align: 0.3,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 2,
                        set_margin_all: 10,

                        gtk::Label {
                            set_markup: &fl!(I18N, "amd-oc-disabled"),
                            set_wrap: true,
                            set_wrap_mode: pango::WrapMode::Word,
                        },

                        gtk::Button {
                            set_label: &fl!(I18N, "enable-amd-oc"),
                            set_halign: gtk::Align::End,

                            connect_clicked[sender] => move |_| {
                                sender.output(AppMsg::ask_confirmation(
                                    AppMsg::EnableOverdrive,
                                    fl!(I18N, "enable-amd-oc"),
                                    fl!(I18N, "enable-amd-oc-description", path = BASE_MODULE_CONF_PATH),
                                    gtk::ButtonsType::OkCancel,
                                )).expect("Channel closed");
                            }
                        },
                    },
                },

                model.stats_section.widget(),
                model.power_cap_section.widget(),
                model.performance_frame.widget(),
                model.power_states_frame.widget(),
                model.clocks_frame.widget(),
            }
        }
    }

    fn init(
        system_info: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let stats_section = GpuStatsSection::detach_default();
        let power_cap_section = PowerCapSection::detach_default();
        let clocks_frame = ClocksFrame::detach_default();
        let power_states_frame = PowerStatesFrame::detach_default();
        let performance_frame = PerformanceFrame::builder()
            .launch(())
            .forward(sender.input_sender(), |msg| msg);

        let model = Self {
            stats_section,
            device_info: None,
            system_info,
            performance_frame,
            power_cap_section,
            power_states_frame,
            clocks_frame,
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
            OcPageMsg::Update { update, initial } => {
                self.stats_section.emit(update.clone());
                match &update {
                    PageUpdate::Stats(stats) => {
                        self.power_states_frame
                            .emit(PowerStatesFrameMsg::Stats(stats.clone()));

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
                        self.power_states_frame
                            .emit(PowerStatesFrameMsg::VramClockRatio(vram_clock_ratio));
                        self.clocks_frame
                            .emit(ClocksFrameMsg::VramRatio(vram_clock_ratio));
                    }
                }
            }
            OcPageMsg::ClocksTable(table) => {
                self.clocks_frame.emit(ClocksFrameMsg::Clocks(table));
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
                        pstates,
                        configured,
                    });
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

    pub fn get_clocks_commands(&self) -> Vec<SetClocksCommand> {
        self.clocks_frame.model().get_commands()
    }

    pub fn get_enabled_power_states(&self) -> IndexMap<PowerLevelKind, Vec<u8>> {
        if self.performance_frame.model().performance_level() == Some(PerformanceLevel::Manual) {
            self.power_states_frame.model().get_enabled_power_states()
        } else {
            IndexMap::new()
        }
    }
}
