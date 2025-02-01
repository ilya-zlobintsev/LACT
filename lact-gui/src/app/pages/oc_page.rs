mod clocks_frame;
mod gpu_stats_section;
mod performance_frame;
mod power_cap_section;
mod power_profile;
mod power_states;

use super::PageUpdate;
use crate::app::msg::AppMsg;
use amdgpu_sysfs::gpu_handle::{
    power_profile_mode::PowerProfileModesTable, PerformanceLevel, PowerLevelKind,
};
use clocks_frame::{ClocksFrame, ClocksFrameMsg};
use gpu_stats_section::GpuStatsSection;
use gtk::{
    pango,
    prelude::{BoxExt, ButtonExt, FrameExt, OrientableExt, WidgetExt},
};
use lact_daemon::MODULE_CONF_PATH;
use lact_schema::{request::SetClocksCommand, ClocksTable, PowerStates, SystemInfo};
use performance_frame::PerformanceFrame;
use power_cap_section::{PowerCapMsg, PowerCapSection};
use power_states::power_states_frame::PowerStatesFrame;
use relm4::{ComponentController, ComponentParts, ComponentSender, RelmWidgetExt};
use std::{cell::Cell, collections::HashMap, rc::Rc};

const OVERCLOCKING_DISABLED_TEXT: &str = "Overclocking support is not enabled! \
You can still change basic settings, but the more advanced clocks and voltage control will not be available.";

pub struct OcPage {
    stats_section: GpuStatsSection,
    pub performance_frame: PerformanceFrame,
    power_cap_section: relm4::Controller<PowerCapSection>,
    power_states_frame: PowerStatesFrame,
    clocks_frame: relm4::Controller<ClocksFrame>,
    // TODO: refactor this out when child components use senders
    signals_blocked: Rc<Cell<bool>>,
}

#[derive(Debug)]
pub enum OcPageMsg {
    Update { update: PageUpdate, initial: bool },
    ClocksTable(Option<ClocksTable>),
    ProfileModesTable(Option<PowerProfileModesTable>),
    PowerStates(PowerStates),
}

#[relm4::component(pub)]
impl relm4::Component for OcPage {
    type Init = Rc<SystemInfo>;
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
                    set_visible: system_info.amdgpu_overdrive_enabled == Some(false),
                    set_label_align: 0.3,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 2,
                        set_margin_all: 10,

                        gtk::Label {
                            set_markup: OVERCLOCKING_DISABLED_TEXT,
                            set_wrap: true,
                            set_wrap_mode: pango::WrapMode::Word,
                        },

                        gtk::Button {
                            set_label: "Enable Overclocking",
                            set_halign: gtk::Align::End,

                            connect_clicked[sender] => move |_| {
                                sender.output(AppMsg::ask_confirmation(
                                    AppMsg::EnableOverdrive,
                                    "Enable Overclocking",
                                    format!("This will enable the overdrive feature of the amdgpu driver by creating a file at <b>{MODULE_CONF_PATH}</b> and updating the initramfs. Are you sure you want to do this?"),
                                    gtk::ButtonsType::OkCancel,
                                )).expect("Channel closed");
                            }
                        },
                    },
                },

                model.stats_section.clone(),
                model.power_cap_section.widget(),
                model.performance_frame.container.clone(),
                model.power_states_frame.clone(),
                model.clocks_frame.widget(),
            }
        }
    }

    fn init(
        system_info: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let power_cap_section = PowerCapSection::builder().launch(()).detach();
        let clocks_frame = ClocksFrame::builder().launch(()).detach();

        let model = Self {
            stats_section: GpuStatsSection::new(),
            performance_frame: PerformanceFrame::new(),
            power_cap_section,
            power_states_frame: PowerStatesFrame::new(),
            clocks_frame,
            signals_blocked: Rc::new(Cell::new(false)),
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
        self.signals_blocked.set(true);
        match msg {
            OcPageMsg::Update { update, initial } => match &update {
                PageUpdate::Stats(stats) => {
                    self.stats_section.set_stats(stats);
                    self.power_states_frame.set_stats(stats);

                    if initial {
                        self.power_cap_section
                            .emit(PowerCapMsg::Update(update.clone()));

                        if stats.power.cap_current.is_some() {
                            self.power_cap_section.widget().set_visible(true);
                        } else {
                            self.power_cap_section.widget().set_visible(false);
                        }

                        match stats.performance_level {
                            Some(profile) => {
                                self.performance_frame.show();
                                self.performance_frame.set_active_level(profile);
                            }
                            None => self.performance_frame.hide(),
                        }
                    }
                }
                PageUpdate::Info(info) => {
                    let vram_clock_ratio = info.vram_clock_ratio();

                    self.power_states_frame
                        .set_vram_clock_ratio(vram_clock_ratio);
                    self.stats_section.set_vram_clock_ratio(vram_clock_ratio);
                    self.clocks_frame
                        .emit(ClocksFrameMsg::VramRatio(vram_clock_ratio));
                }
            },
            OcPageMsg::ClocksTable(table) => {
                self.clocks_frame.emit(ClocksFrameMsg::Clocks(table));
            }
            OcPageMsg::ProfileModesTable(modes_table) => {
                self.performance_frame.set_power_profile_modes(modes_table);
            }
            OcPageMsg::PowerStates(states) => {
                self.power_states_frame.set_power_states(states);
            }
        }

        self.signals_blocked.set(false);

        let signals_blocked = self.signals_blocked.clone();
        let signals_sender = sender.clone();
        let f = move || {
            if !signals_blocked.get() {
                signals_sender
                    .output(AppMsg::SettingsChanged)
                    .expect("Channel closed")
            }
        };
        self.performance_frame.connect_settings_changed(f.clone());
        self.power_states_frame.connect_values_changed(f);

        self.update_view(widgets, sender);
    }
}

impl OcPage {
    pub fn get_performance_level(&self) -> Option<PerformanceLevel> {
        if self.performance_frame.get_visibility() {
            let level = self.performance_frame.get_selected_performance_level();
            Some(level)
        } else {
            None
        }
    }

    pub fn get_power_cap(&self) -> Option<f64> {
        self.power_cap_section.model().get_user_cap()
    }

    pub fn get_clocks_commands(&self) -> Vec<SetClocksCommand> {
        self.clocks_frame.model().get_commands()
    }

    pub fn get_enabled_power_states(&self) -> HashMap<PowerLevelKind, Vec<u8>> {
        if self.performance_frame.get_selected_performance_level() == PerformanceLevel::Manual {
            self.power_states_frame.get_enabled_power_states()
        } else {
            HashMap::new()
        }
    }
}
