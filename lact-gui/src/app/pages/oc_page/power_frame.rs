use crate::{
    APP_BROKER, I18N,
    app::{
        components::{
            adjustment_card::AdjustmentCard,
            page_section::PageSection,
            setting_row::{
                AdjustmentInit, SelectInit, SelectOption, SettingRow, SettingRowInit,
                SettingRowMsg, SettingRowValue,
            },
        },
        msg::AppMsg,
        pages::oc_page::{
            OcPageMsg,
            power_profile_mode::{PerformanceFrame, PerformanceFrameMsg},
        },
        utils::ext::RelmDefaultLauchable,
    },
};
use adw::prelude::*;
use amdgpu_sysfs::gpu_handle::{PerformanceLevel, power_profile_mode::PowerProfileModesTable};
use i18n_embed_fl::fl;
use lact_schema::{DeviceStats, config::GpuConfig};
use nvml_wrapper::enums::device::PowerMizerMode;
use relm4::{
    ComponentController, ComponentParts, ComponentSender, WidgetTemplate, factory::FactoryHashMap,
};
use std::sync::Arc;

const PERFORMANCE_LEVELS: [PerformanceLevel; 8] = [
    PerformanceLevel::Auto,
    PerformanceLevel::High,
    PerformanceLevel::Low,
    PerformanceLevel::Manual,
    PerformanceLevel::ProfileStandard,
    PerformanceLevel::ProfileMinSclk,
    PerformanceLevel::ProfileMinMclk,
    PerformanceLevel::ProfilePeak,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PowerSetting {
    PowerCap,
    PerformanceLevel,
    PowerMizer,
}

pub struct PowerFrame {
    rows: FactoryHashMap<PowerSetting, SettingRow<PowerSetting>>,
    performance_levels: Vec<PerformanceLevel>,
    performance_frame: relm4::Controller<PerformanceFrame>,
    supported_power_mizer_modes: Vec<PowerMizerMode>,
}

#[derive(Debug)]
pub enum PowerFrameMsg {
    Stats(Arc<DeviceStats>),
    SetPerformanceLevel(PerformanceLevel),
    PowerProfileModes(Option<PowerProfileModesTable>),
    SettingsChanged,
    RefreshVisibility,
    Reset,
}

#[relm4::component(pub)]
impl relm4::Component for PowerFrame {
    type Init = ();
    type Input = PowerFrameMsg;
    type Output = OcPageMsg;
    type CommandOutput = ();

    view! {
        #[root]
        PageSection {
            set_name: fl!(I18N, "power-section"),
            set_hide_visible_container: true,
            #[watch]
            set_visible: !model.rows.is_empty() || model.performance_frame.widget().power_profile.get_visible(),

            append_header = &gtk::Button {
                set_label: &fl!(I18N, "default-button"),
                connect_clicked => PowerFrameMsg::Reset,
                set_halign: gtk::Align::End,
                set_hexpand: true,
                #[watch]
                set_visible: !model.rows.is_empty(),
            },
            append_child = model.rows.widget(),
        },
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            rows: FactoryHashMap::builder()
                .launch(AdjustmentCard::init(()).content.clone())
                .forward(sender.input_sender(), |()| PowerFrameMsg::SettingsChanged),
            performance_levels: PERFORMANCE_LEVELS.to_vec(),
            performance_frame: PerformanceFrame::launch_default()
                .forward(sender.output_sender(), |msg| msg),
            supported_power_mizer_modes: Vec::new(),
        };
        let row = &model.performance_frame.widget().power_profile;
        model.rows.widget().append(row);
        let profile_row = row.clone();
        model.rows.widget().set_sort_func(move |left, right| {
            // Keep the power profile row after the dynamically inserted setting rows.
            (left == &profile_row).cmp(&(right == &profile_row)).into()
        });
        let visibility_sender = sender.clone();
        row.connect_visible_notify(move |_| {
            visibility_sender.input(PowerFrameMsg::RefreshVisibility);
        });
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
            PowerFrameMsg::Stats(stats) => {
                self.rows.clear();
                let power = &stats.power;
                if let Some(value) = power.cap_current {
                    self.rows.insert(
                        PowerSetting::PowerCap,
                        SettingRowInit {
                            title: fl!(I18N, "power-cap"),
                            control: AdjustmentInit {
                                unit: fl!(I18N, "watt"),
                                value,
                                default_value: power.cap_default,
                                lower: power.cap_min.unwrap_or_default(),
                                upper: power.cap_max.unwrap_or_default(),
                                ..Default::default()
                            }
                            .into(),
                            ..Default::default()
                        },
                    );
                }

                self.performance_levels = PERFORMANCE_LEVELS.to_vec();
                if let Some(level) = stats.performance_level {
                    if !self.performance_levels.contains(&level) {
                        self.performance_levels.push(level);
                    }
                    self.rows.insert(
                        PowerSetting::PerformanceLevel,
                        SettingRowInit {
                            title: fl!(I18N, "performance-level"),
                            control: SelectInit {
                                default_value: self
                                    .performance_levels
                                    .iter()
                                    .position(|level| *level == PerformanceLevel::Auto)
                                    .map(|index| index as u32),
                                options: self
                                    .performance_levels
                                    .iter()
                                    .copied()
                                    .map(performance_option)
                                    .collect(),
                                selected: self
                                    .performance_levels
                                    .iter()
                                    .position(|candidate| *candidate == level)
                                    .map(|index| index as u32),
                            }
                            .into(),
                            ..Default::default()
                        },
                    );
                }
                self.performance_frame
                    .emit(PerformanceFrameMsg::PerformanceLevel(
                        stats.performance_level,
                    ));

                self.supported_power_mizer_modes = stats
                    .supported_power_mizer_modes
                    .clone()
                    .unwrap_or_default();
                if let Some(active) = stats.active_power_mizer_mode {
                    if !self.supported_power_mizer_modes.contains(&active) {
                        self.supported_power_mizer_modes.push(active);
                    }
                    self.rows.insert(
                        PowerSetting::PowerMizer,
                        SettingRowInit {
                            title: fl!(I18N, "power-mizer-mode"),
                            control: SelectInit {
                                default_value: self
                                    .supported_power_mizer_modes
                                    .iter()
                                    .position(|mode| *mode == PowerMizerMode::Auto)
                                    .map(|index| index as u32),
                                options: self
                                    .supported_power_mizer_modes
                                    .iter()
                                    .copied()
                                    .map(power_mizer_option)
                                    .collect(),
                                selected: self
                                    .supported_power_mizer_modes
                                    .iter()
                                    .position(|mode| *mode == active)
                                    .map(|index| index as u32),
                            }
                            .into(),
                            ..Default::default()
                        },
                    );
                }
            }
            PowerFrameMsg::SetPerformanceLevel(level) => {
                if self.rows.get(&PowerSetting::PerformanceLevel).is_some()
                    && let Some(index) = self
                        .performance_levels
                        .iter()
                        .position(|candidate| *candidate == level)
                {
                    self.rows.send(
                        &PowerSetting::PerformanceLevel,
                        SettingRowMsg::SetValue(SettingRowValue::Selected(index as u32)),
                    );
                }
            }
            PowerFrameMsg::PowerProfileModes(table) => {
                self.performance_frame
                    .emit(PerformanceFrameMsg::PowerProfileModes(table));
            }
            PowerFrameMsg::SettingsChanged => {
                APP_BROKER.send(AppMsg::SettingsChanged);
                self.performance_frame
                    .emit(PerformanceFrameMsg::PerformanceLevel(
                        self.performance_level(),
                    ));
            }
            PowerFrameMsg::RefreshVisibility => (),
            PowerFrameMsg::Reset => {
                for setting in self.rows.keys() {
                    self.rows.send(setting, SettingRowMsg::Reset);
                }
            }
        }

        self.update_view(widgets, sender);
    }
}

impl PowerFrame {
    pub fn serialize(&self) -> GpuConfig {
        let value = |setting| self.rows.get(&setting).and_then(SettingRow::serialize);
        let selected = |setting| value(setting).and_then(SettingRowValue::as_selected);
        let power_profile_mode_index = self.performance_frame.model().power_profile_mode();
        GpuConfig {
            power_cap: value(PowerSetting::PowerCap).and_then(SettingRowValue::as_numeric),
            performance_level: selected(PowerSetting::PerformanceLevel)
                .and_then(|index| self.performance_levels.get(index as usize).copied()),
            power_mizer_mode: selected(PowerSetting::PowerMizer).and_then(|index| {
                self.supported_power_mizer_modes
                    .get(index as usize)
                    .copied()
            }),
            power_profile_mode_index,
            custom_power_profile_mode_hueristics: if power_profile_mode_index.is_some() {
                self.performance_frame
                    .model()
                    .power_profile_mode_custom_heuristics()
            } else {
                Vec::new()
            },
            ..Default::default()
        }
    }

    fn selected(&self, setting: PowerSetting) -> Option<u32> {
        self.rows.get(&setting)?.get_value().as_selected()
    }

    pub fn performance_level(&self) -> Option<PerformanceLevel> {
        self.performance_levels
            .get(self.selected(PowerSetting::PerformanceLevel)? as usize)
            .copied()
    }
}

fn performance_option(level: PerformanceLevel) -> SelectOption {
    let (label, description) = match level {
        PerformanceLevel::Auto => (
            fl!(I18N, "performance-level-auto"),
            fl!(I18N, "performance-level-auto-description"),
        ),
        PerformanceLevel::High => (
            fl!(I18N, "performance-level-high"),
            fl!(I18N, "performance-level-high-description"),
        ),
        PerformanceLevel::Low => (
            fl!(I18N, "performance-level-low"),
            fl!(I18N, "performance-level-low-description"),
        ),
        PerformanceLevel::Manual => (
            fl!(I18N, "performance-level-manual"),
            fl!(I18N, "performance-level-manual-description"),
        ),
        PerformanceLevel::ProfileStandard => (
            fl!(I18N, "performance-level-profile-standard"),
            fl!(I18N, "performance-level-profile-standard-description"),
        ),
        PerformanceLevel::ProfileMinSclk => (
            fl!(I18N, "performance-level-profile-min-sclk"),
            fl!(I18N, "performance-level-profile-min-sclk-description"),
        ),
        PerformanceLevel::ProfileMinMclk => (
            fl!(I18N, "performance-level-profile-min-mclk"),
            fl!(I18N, "performance-level-profile-min-mclk-description"),
        ),
        PerformanceLevel::ProfilePeak => (
            fl!(I18N, "performance-level-profile-peak"),
            fl!(I18N, "performance-level-profile-peak-description"),
        ),
        PerformanceLevel::PerfDeterminism | PerformanceLevel::ProfileExit => {
            (level.to_string(), String::new())
        }
    };
    SelectOption { label, description }
}

fn power_mizer_option(mode: PowerMizerMode) -> SelectOption {
    let (label, description) = match mode {
        PowerMizerMode::Auto => (
            fl!(I18N, "power-mizer-mode-auto"),
            fl!(I18N, "power-mizer-mode-auto-description"),
        ),
        PowerMizerMode::Adaptive => (
            fl!(I18N, "power-mizer-mode-adaptive"),
            fl!(I18N, "power-mizer-mode-adaptive-description"),
        ),
        PowerMizerMode::PreferMaximumPerformance => (
            fl!(I18N, "power-mizer-mode-prefer-maximum-performance"),
            fl!(
                I18N,
                "power-mizer-mode-prefer-maximum-performance-description"
            ),
        ),
        PowerMizerMode::PreferConsistentPerformance => (
            fl!(I18N, "power-mizer-mode-prefer-consistent-performance"),
            fl!(
                I18N,
                "power-mizer-mode-prefer-consistent-performance-description"
            ),
        ),
    };
    SelectOption { label, description }
}
