use crate::I18N;
use crate::app::SharedStatsHistory;
use crate::app::utils::stats::StatKind;
use crate::app::{
    components::{
        info_row::{InfoRow, InfoRowExt},
        info_row_level::InfoRowLevel,
        page_section::PageSection,
    },
    utils::{
        ext::FlowBoxExt,
        formatting::{self, Mono},
    },
};
use gtk::pango::AttrList;
use gtk::prelude::{BoxExt, OrientableExt, PopoverExt as _, WidgetExt};
use i18n_embed_fl::fl;
use lact_schema::{DeviceInfo, DeviceStats, PowerStates};
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt as _};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::str::FromStr as _;
use std::sync::Arc;
use tracing::error;

pub struct GpuStatsSection {
    stats_history: SharedStatsHistory,
    latest_stats_snapshot: HashMap<StatKind, StatValue>,
    /// Most things should be accessed through `latest_stats_snapshot`
    stats_raw: Arc<DeviceStats>,
    vram_clock_ratio: f64,
    gpu_model: String,
    max_gpu_clock: Option<u64>,
    max_vram_clock: Option<u64>,
    min_gpu_clock: Option<u64>,
    min_vram_clock: Option<u64>,
}

#[derive(Debug)]
pub enum GpuStatsSectionMsg {
    Info(Arc<DeviceInfo>),
    Stats(Arc<DeviceStats>),
    PowerStates(Arc<PowerStates>),
}

#[derive(Debug)]
enum StatValue {
    Single(f64),
    /// Labeled stat values (like temperature)
    Multi(Vec<(String, f64)>),
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for GpuStatsSection {
    type Input = GpuStatsSectionMsg;
    type Output = ();
    type Init = SharedStatsHistory;

    view! {
        gtk::Box {
            add_css_class: "gpu-stats-section",
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,

            PageSection {
                append_child = &gtk::FlowBox {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_column_spacing: 10,
                    set_row_spacing: 10,
                    set_homogeneous: true,
                    set_selection_mode: gtk::SelectionMode::None,

                    append = &InfoRow {
                        set_name: fl!(I18N, "device-name"),
                        #[watch]
                        set_value: model.gpu_model.clone(),
                    },

                    append = &InfoRow {
                        set_name: fl!(I18N, "throttling"),
                        #[watch]
                        set_value: formatting::fmt_throttling_text(&model.stats_raw),
                    },

                    append_child = &InfoRow {
                        set_name: fl!(I18N, "gpu-clock-target"),
                        #[watch]
                        set_value: format_current_gfxclk(model.read_plain_stat(StatKind::GpuTargetClock)),
                    } -> clockspeed_target_item: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: model.has_stat(StatKind::GpuTargetClock)
                    },

                    append_child = &InfoRow {
                        set_name: fl!(I18N, "gpu-voltage"),
                        #[watch]
                        set_value: format!("{} V", Mono::float(model.read_plain_stat(StatKind::GpuVoltage).unwrap_or(0.0) / 1000f64, 3)),
                    } -> gpu_voltage_item: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: model.has_stat(StatKind::GpuVoltage),
                    },


                    append_child = &InfoRow {
                        set_name: fl!(I18N, "gpu-temp"),
                        #[watch]
                        set_value: if primary_temperatures.is_empty() {
                            "N/A".to_owned()
                        } else {
                            primary_temperatures.join(", ")
                        },
                    } -> basic_temps_item: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: secondary_temperatures.is_empty(),
                    },

                    append_child = &InfoRow {
                        set_name: fl!(I18N, "gpu-temp"),
                        #[watch]
                        set_value: if primary_temperatures.is_empty() {
                            "N/A".to_owned()
                        } else {
                            primary_temperatures.join(", ")
                        },

                        set_icon: "go-down-symbolic".to_string(),

                        #[name = "secondary_temps_popover"]
                        set_popover = &gtk::Popover {
                            gtk::Label {
                                set_margin_all: 10,
                                set_selectable: false,
                                set_use_markup: true,
                                set_attributes: Some(&AttrList::from_str("0 -1 weight bold").unwrap()),

                                #[watch]
                                set_label: &secondary_temperatures.join("\n"),
                            },
                        },

                        connect_clicked[secondary_temps_popover] => move |_| {
                            secondary_temps_popover.popup();
                        },
                    } -> full_temps_item: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: !secondary_temperatures.is_empty(),
                    },
                },
            },

            PageSection {
                append_child = &gtk::FlowBox {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_column_spacing: 10,
                    set_homogeneous: true,
                    set_selection_mode: gtk::SelectionMode::None,

                    append_child = &InfoRowLevel {
                        #[watch]
                        set_name: {
                            if model.has_stat(StatKind::GpuClock) && model.has_stat(StatKind::GpuTargetClock) {
                                fl!(I18N, "gpu-clock-avg")
                            } else {
                                fl!(I18N, "gpu-clock")
                            }
                        },
                        #[watch]
                        set_value: formatting::fmt_clockspeed(
                            model.read_plain_stat(StatKind::GpuClock),
                            1.0,
                        ),
                        #[watch]
                        set_level_value: {
                            match (model.read_plain_stat(StatKind::GpuClock), model.max_gpu_clock, model.min_gpu_clock) {
                                (Some(cur), Some(max), Some(min)) if max > min => {
                                    (cur - min as f64) / max.saturating_sub(min) as f64
                                }
                                _ => 0.0,
                            }
                        },
                        set_value_size_group: &value_size_group,
                    } -> gpu_clock_item: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: model.has_stat(StatKind::GpuClock),
                    },

                    append_child = &InfoRowLevel {
                        set_name: fl!(I18N, "vram-clock"),
                        set_value_size_group: &value_size_group,
                        #[watch]
                        set_value: formatting::fmt_clockspeed(
                            model.read_plain_stat(StatKind::VramClock),
                            model.vram_clock_ratio,
                        ),
                        #[watch]
                        set_level_value: {
                            match (model.read_plain_stat(StatKind::VramClock), model.max_vram_clock, model.min_vram_clock) {
                                (Some(cur), Some(max), Some(min)) if max > min => {
                                    (cur - min as f64) / (max.saturating_sub(min) as f64)
                                }
                                _ => 0.0,
                            }
                        }
                    } -> vram_clock_item: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: model.has_stat(StatKind::VramClock),
                    },

                    append_child = &InfoRowLevel {
                        set_name: fl!(I18N, "gpu-usage"),
                        set_value_size_group: &value_size_group,
                        #[watch]
                        set_value: format!("{}%", Mono::float(model.read_plain_stat(StatKind::GpuUsage).unwrap_or(0.0), 0)),
                        #[watch]
                        set_level_value: model.read_plain_stat(StatKind::GpuUsage).unwrap_or(0.0) / 100.0,
                    } -> gpu_usage_item: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: model.has_stat(StatKind::GpuUsage),
                    },

                    append_child = &InfoRowLevel {
                        set_name: fl!(I18N, "vram-usage"),
                        set_value_size_group: &value_size_group,
                        #[watch]
                        set_value: formatting::fmt_human_bytes(
                            model.read_plain_stat(StatKind::VramUsed).map(|value| value * 1024.0 * 1024.0).map(|value| value as u64).unwrap_or(0),
                            Some(formatting::ByteUnit::Gibibyte),
                        ),
                        #[watch]
                        set_level_value: model
                            .read_plain_stat(StatKind::VramUsed)
                            .zip(model.read_plain_stat(StatKind::VramSize))
                            .map(|(used, total)| used / total)
                            .unwrap_or(0.0),
                    } -> vram_usage_item: gtk::FlowBoxChild {},

                    append_child = &InfoRowLevel {
                        set_name: fl!(I18N, "gtt-usage"),
                        set_value_size_group: &value_size_group,
                        #[watch]
                        set_value: formatting::fmt_human_bytes(
                            model.read_plain_stat(StatKind::GttUsed).map(|value| value * 1024.0 * 1024.0).map(|value| value as u64).unwrap_or(0),
                            Some(formatting::ByteUnit::Gibibyte),
                        ),
                        #[watch]
                        set_level_value: model
                            .read_plain_stat(StatKind::GttUsed)
                            .zip(model.read_plain_stat(StatKind::GttSize))
                            .map(|(used, total)| used / total)
                            .unwrap_or(0.0),
                    } -> gtt_usage_item: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: model.has_stat(StatKind::GttUsed),
                    },

                    append_child = &InfoRowLevel {
                        set_name: fl!(I18N, "power-usage"),
                        set_value_size_group: &value_size_group,
                        #[watch]
                        set_value: {
                            let power_current = model
                                .read_plain_stat(StatKind::PowerCurrent)
                                .filter(|value| *value != 0.0)
                                .or_else(|| model.read_plain_stat(StatKind::PowerAverage));

                            format!(
                                "{} {}",
                                Mono::float(power_current.unwrap_or(0.0), 1),
                                fl!(I18N, "watt")
                            )
                        },
                        #[watch]
                        set_level_value: {
                            let power_current = model
                                .read_plain_stat(StatKind::PowerCurrent)
                                .filter(|value| *value != 0.0)
                                .or_else(|| model.read_plain_stat(StatKind::PowerAverage));

                            let power_cap = model
                                .read_plain_stat(StatKind::PowerCap)
                                .filter(|value| *value != 0.0);

                            power_current
                                .zip(power_cap)
                                .map(|(current, cap)| current / cap)
                                .unwrap_or(0.0)
                        },
                    } -> power_usage_item: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: model.has_stat(StatKind::PowerCurrent) || model.has_stat(StatKind::PowerAverage),
                    },

                    append_child = &InfoRowLevel {
                        set_name: fl!(I18N, "fan-speed"),
                        set_value_size_group: &value_size_group,
                        #[watch]
                        set_value: formatting::fmt_fan_speed(model.read_plain_stat(StatKind::FanRpm).map(|value| value as u64), model.read_plain_stat(StatKind::FanPercent).map(|value| value as u64), true)
                            .unwrap_or_else(|| fl!(I18N, "missing-stat")),
                        #[watch]
                        set_level_value: model.read_plain_stat(StatKind::FanPercent).map(|value| value / 100.0).unwrap_or(0.0),
                    } -> fan_speed_item: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: model.has_stat(StatKind::FanRpm) || model.has_stat(StatKind::FanPercent),
                    },
                },
            },
        }
    }

    fn init(
        stats_history: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let value_size_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);

        let (primary_temperatures, secondary_temperatures): (Vec<String>, Vec<String>) =
            (Vec::new(), Vec::new());

        let model = Self {
            stats_history,
            latest_stats_snapshot: HashMap::new(),
            stats_raw: Arc::new(DeviceStats::default()),
            vram_clock_ratio: 1.0,
            gpu_model: String::new(),
            max_gpu_clock: None,
            max_vram_clock: None,
            min_gpu_clock: None,
            min_vram_clock: None,
        };

        let widgets = view_output!();

        ComponentParts { widgets, model }
    }

    fn update(&mut self, msg: Self::Input, _sender: relm4::ComponentSender<Self>) {
        match msg {
            GpuStatsSectionMsg::Info(info) => {
                self.vram_clock_ratio = info.vram_clock_ratio();
                if let Some(pci_info) = &info.pci_info {
                    self.gpu_model = info
                        .drm_info
                        .as_ref()
                        .and_then(|drm| drm.device_name.as_deref())
                        .or(pci_info.device_pci_info.model.as_deref())
                        .unwrap_or("Unknown")
                        .to_owned();
                }
            }
            GpuStatsSectionMsg::Stats(stats) => {
                let stats_history = self.stats_history.read().unwrap();

                self.latest_stats_snapshot.clear();

                for (stat, value) in stats_history.iter_latest() {
                    match self.latest_stats_snapshot.entry(stat.kind) {
                        Entry::Vacant(entry) => {
                            let stat_value = match stat.label.clone() {
                                Some(label) => StatValue::Multi(vec![(label, value)]),
                                None => StatValue::Single(value),
                            };
                            entry.insert(stat_value);
                        }
                        Entry::Occupied(mut entry) => match entry.get_mut() {
                            StatValue::Multi(items) => {
                                let label = stat
                                    .label
                                    .clone()
                                    .expect("Mixing labeled and non-labeled stats is illegal");
                                items.push((label, value));
                            }
                            StatValue::Single(_) => {
                                unreachable!("Mixing labeled and non-labeled stats is illegal")
                            }
                        },
                    }
                }

                self.stats_raw = stats;
            }
            GpuStatsSectionMsg::PowerStates(pstates) => {
                self.max_gpu_clock = pstates.max_gpu_clock();
                self.max_vram_clock = pstates.max_vram_clock();
                self.min_gpu_clock = pstates.min_gpu_clock();
                self.min_vram_clock = pstates.min_vram_clock();
            }
        }
    }

    fn pre_view(&self) {
        // stat_history cannot currently be used here, as it does not carry primary/secondary sensor information
        let (primary_temperatures, secondary_temperatures) =
            formatting::fmt_temperature_text(&model.stats_raw);
    }
}

impl GpuStatsSection {
    fn read_plain_stat(&self, stat: StatKind) -> Option<f64> {
        match self.latest_stats_snapshot.get(&stat)? {
            StatValue::Single(value) => Some(*value),
            StatValue::Multi(_) => {
                error!("found unexpected multi stat {stat:?}");
                None
            }
        }
    }

    fn has_stat(&self, stat: StatKind) -> bool {
        self.latest_stats_snapshot.contains_key(&stat)
    }
}

fn format_current_gfxclk(value: Option<f64>) -> String {
    if let Some(v) = value {
        // if the APU/GPU does not actually support current_gfxclk,
        // the value will be `u16::MAX (65535)`
        if v >= u16::MAX as f64 || v == 0.0 {
            fl!(I18N, "missing-stat")
        } else {
            formatting::fmt_clockspeed(Some(v), 1.0)
        }
    } else {
        fl!(I18N, "missing-stat")
    }
}
