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
use clocks_frame::ClocksFrame;
use gpu_stats_section::GpuStatsSection;
use gtk::{
    glib::object::ObjectExt,
    pango,
    prelude::{BoxExt, ButtonExt, FrameExt, OrientableExt, WidgetExt},
};
use lact_daemon::MODULE_CONF_PATH;
use lact_schema::{ClocksTable, SystemInfo};
use performance_frame::PerformanceFrame;
use power_cap_section::PowerCapSection;
use power_states::power_states_frame::PowerStatesFrame;
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt};
use std::{cell::Cell, collections::HashMap, rc::Rc};
use tracing::warn;

const OVERCLOCKING_DISABLED_TEXT: &str = "Overclocking support is not enabled! \
You can still change basic settings, but the more advanced clocks and voltage control will not be available.";

pub struct OcPage {
    stats_section: GpuStatsSection,
    pub performance_frame: PerformanceFrame,
    power_cap_section: PowerCapSection,
    pub power_states_frame: PowerStatesFrame,
    pub clocks_frame: ClocksFrame,
    // TODO: refactor this out when child components use senders
    signals_blocked: Rc<Cell<bool>>,
}

#[derive(Debug)]
pub enum OcPageMsg {
    Update { update: PageUpdate, initial: bool },
    ClocksTable(Option<ClocksTable>),
    ProfileModesTable(Option<PowerProfileModesTable>),
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

                model.power_cap_section.clone() {
                    connect_current_value_notify[sender] => move |_| {
                        sender.output(AppMsg::SettingsChanged).unwrap();
                    } @power_cap_notify,
                },

                model.performance_frame.container.clone(),

                model.power_states_frame.clone(),

                model.clocks_frame.container.clone(),
            }
        }
    }

    fn init(
        system_info: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            stats_section: GpuStatsSection::new(),
            performance_frame: PerformanceFrame::new(),
            power_cap_section: PowerCapSection::new(),
            power_states_frame: PowerStatesFrame::new(),
            clocks_frame: ClocksFrame::new(),
            signals_blocked: Rc::new(Cell::new(false)),
        };

        let widgets = view_output!();

        model.clocks_frame.connect_clocks_reset(move || {
            sender.output(AppMsg::ResetClocks).expect("Channel closed")
        });

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
            OcPageMsg::Update { update, initial } => match update {
                PageUpdate::Stats(stats) => {
                    self.stats_section.set_stats(&stats);
                    self.power_states_frame.set_stats(&stats);

                    if initial {
                        self.power_cap_section
                            .block_signal(&widgets.power_cap_notify);

                        self.power_cap_section
                            .set_max_value(stats.power.cap_max.unwrap_or_default());
                        self.power_cap_section
                            .set_min_value(stats.power.cap_min.unwrap_or_default());
                        self.power_cap_section
                            .set_default_value(stats.power.cap_default.unwrap_or_default());

                        if let Some(current_cap) = stats.power.cap_current {
                            self.power_cap_section.set_initial_value(current_cap);
                            self.power_cap_section.set_visible(true);
                        } else {
                            self.power_cap_section.set_visible(false);
                        }

                        match stats.performance_level {
                            Some(profile) => {
                                self.performance_frame.show();
                                self.performance_frame.set_active_level(profile);
                            }
                            None => self.performance_frame.hide(),
                        }

                        self.power_cap_section
                            .unblock_signal(&widgets.power_cap_notify);
                    }
                }
                PageUpdate::Info(info) => {
                    let vram_clock_ratio = info
                        .drm_info
                        .as_ref()
                        .map(|info| info.vram_clock_ratio)
                        .unwrap_or(1.0);

                    self.power_states_frame
                        .set_vram_clock_ratio(vram_clock_ratio);
                    self.stats_section.set_vram_clock_ratio(vram_clock_ratio);
                    self.clocks_frame.set_vram_clock_ratio(vram_clock_ratio);
                }
            },
            OcPageMsg::ClocksTable(table) => match table {
                Some(table) => match self.clocks_frame.set_table(table) {
                    Ok(()) => {
                        self.clocks_frame.show();
                    }
                    Err(err) => {
                        warn!("got invalid clocks table: {err:?}");
                        self.clocks_frame.hide();
                    }
                },
                None => {
                    self.clocks_frame.hide();
                }
            },
            OcPageMsg::ProfileModesTable(modes_table) => {
                self.performance_frame.set_power_profile_modes(modes_table);
            }
        }

        self.signals_blocked.set(false);

        let signals_blocked = self.signals_blocked.clone();
        let f = move || {
            if !signals_blocked.get() {
                sender
                    .output(AppMsg::SettingsChanged)
                    .expect("Channel closed")
            }
        };
        self.performance_frame.connect_settings_changed(f.clone());
        self.clocks_frame.connect_clocks_changed(f.clone());
        self.power_states_frame.connect_values_changed(f);
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
        self.power_cap_section.get_user_cap()
    }

    pub fn get_enabled_power_states(&self) -> HashMap<PowerLevelKind, Vec<u8>> {
        if self.performance_frame.get_selected_performance_level() == PerformanceLevel::Manual {
            self.power_states_frame.get_enabled_power_states()
        } else {
            HashMap::new()
        }
    }
}
