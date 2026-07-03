use i18n_embed_fl::fl;
use lact_schema::DeviceStats;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::LazyLock};

use crate::I18N;

use super::formatting::{self, Mono};

pub(crate) type StatConfigMap = HashMap<StatType, StatConfig>;
type ValueFn = fn(&StatType, &StatContext<'_>) -> Option<f64>;
type SampleFn = fn(&StatType, Option<f64>, &StatContext<'_>) -> Option<f64>;
type FormatDirectFn = fn(f64) -> String;
type FormatFn = fn(&StatType, Option<f64>, &StatContext<'_>) -> String;
type VisibleFn = fn(&StatType, Option<f64>, &StatContext<'_>) -> bool;
type LevelFn = fn(&StatType, Option<f64>, &StatContext<'_>) -> f64;

#[derive(Debug, Clone)]
pub(crate) struct StatConfig {
    pub label: String,
    // mostly used for graph axis
    pub unit_label: &'static str,
    // flag if graph should show peak value
    pub show_peak: bool,
    // flag if the stat should be displayed on graph
    pub graphable: bool,
    format_direct: FormatDirectFn,
    // raw value
    value: ValueFn,
    // value adjusted for graph
    sample: Option<SampleFn>,
    // formatted value
    format: Option<FormatFn>,
    // should the stat be visible
    visible: Option<VisibleFn>,
    // percentage value of the stat
    level: Option<LevelFn>,
}

impl StatConfig {
    pub(crate) fn value(&self, stat_type: &StatType, context: &StatContext<'_>) -> Option<f64> {
        (self.value)(stat_type, context)
    }

    pub(crate) fn sample(&self, stat_type: &StatType, context: &StatContext<'_>) -> Option<f64> {
        let value = self.value(stat_type, context);
        self.sample
            .map_or(value, |sample| sample(stat_type, value, context))
    }

    pub(crate) fn format_direct(&self, value: f64) -> String {
        (self.format_direct)(value)
    }

    pub(crate) fn format(&self, stat_type: &StatType, context: &StatContext<'_>) -> Option<String> {
        let value = self.value(stat_type, context);
        self.format.map(|format| format(stat_type, value, context))
    }

    pub(crate) fn visible(&self, stat_type: &StatType, context: &StatContext<'_>) -> Option<bool> {
        let value = self.value(stat_type, context);
        self.visible
            .map(|visible| visible(stat_type, value, context))
    }

    pub(crate) fn level(&self, stat_type: &StatType, context: &StatContext<'_>) -> Option<f64> {
        let value = self.value(stat_type, context);
        self.level.map(|level| level(stat_type, value, context))
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct StatContext<'a> {
    pub stats: &'a DeviceStats,
    pub vram_clock_ratio: f64,
    pub max_gpu_clock: Option<u64>,
    pub max_vram_clock: Option<u64>,
    pub min_gpu_clock: Option<u64>,
    pub min_vram_clock: Option<u64>,
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Serialize, Deserialize)]
pub enum StatType {
    GpuClock,
    GpuTargetClock,
    GpuUsage,
    Temperature(String),
    FanRpm,
    FanPwm,
    PowerCurrent,
    PowerAverage,
    PowerCap,
    Power(String),
    VramClock,
    VramSize,
    VramUsage,
    GttSize,
    GttUsage,
    GpuVoltage,
    Clockspeed(String),
    Voltage(String),
    PowerUsage,
}

pub(crate) fn build_stat_config_map(context: &StatContext<'_>) -> StatConfigMap {
    let mut configs = static_stat_configs().clone();

    for name in context.stats.temps.keys() {
        let stat_type = StatType::Temperature(name.clone());
        let config = StatConfig {
            label: format!("{} ({name})", fl!(I18N, "gpu-temp")),
            unit_label: "℃",
            show_peak: true,
            graphable: true,
            format_direct: format_direct_1,
            value: |stat_type, context| {
                let StatType::Temperature(name) = stat_type else {
                    return None;
                };

                context
                    .stats
                    .temps
                    .get(name)
                    .and_then(|temperature| temperature.value.current)
                    .map(Into::into)
            },
            sample: None,
            format: None,
            visible: None,
            level: None,
        };
        configs.insert(stat_type, config);
    }

    for name in context.stats.voltage.sensors.keys() {
        let stat_type = StatType::Voltage(name.clone());
        let config = StatConfig {
            label: format!("{} ({name})", fl!(I18N, "voltage")),
            unit_label: "mV",
            show_peak: true,
            graphable: true,
            format_direct: format_direct_0,
            value: |stat_type, context| {
                let StatType::Voltage(name) = stat_type else {
                    return None;
                };

                context
                    .stats
                    .voltage
                    .sensors
                    .get(name)
                    .map(|val| *val as f64)
            },
            sample: None,
            format: None,
            visible: None,
            level: None,
        };
        configs.insert(stat_type, config);
    }

    for name in context.stats.clockspeed.sensors.keys() {
        let stat_type = StatType::Clockspeed(name.clone());
        let config = StatConfig {
            label: format!("{} ({name})", fl!(I18N, "stat-clockspeed")),
            unit_label: "MHz",
            show_peak: true,
            graphable: true,
            format_direct: format_direct_0,
            value: |stat_type, context| {
                let StatType::Clockspeed(name) = stat_type else {
                    return None;
                };

                context
                    .stats
                    .clockspeed
                    .sensors
                    .get(name)
                    .map(|val| *val as f64)
            },
            sample: None,
            format: None,
            visible: None,
            level: None,
        };
        configs.insert(stat_type, config);
    }

    for name in context.stats.power.sensors.keys() {
        let stat_type = StatType::Power(name.clone());
        let config = StatConfig {
            label: format!("{} ({name})", fl!(I18N, "power-usage")),
            unit_label: "W",
            show_peak: true,
            graphable: true,
            format_direct: format_direct_1,
            value: |stat_type, context| {
                let StatType::Power(name) = stat_type else {
                    return None;
                };

                context.stats.power.sensors.get(name).copied()
            },
            sample: None,
            format: None,
            visible: None,
            level: None,
        };
        configs.insert(stat_type, config);
    }

    configs
}

pub(crate) fn static_stat_configs() -> &'static StatConfigMap {
    static CONFIGS: LazyLock<StatConfigMap> = LazyLock::new(|| {
        HashMap::from([
            (
                StatType::GpuClock,
                StatConfig {
                    label: fl!(I18N, "gpu-clock"),
                    unit_label: "MHz",
                    show_peak: true,
                    graphable: true,
                    format_direct: format_direct_0,
                    value: |_, context| {
                        context
                            .stats
                            .clockspeed
                            .gpu_clockspeed
                            .map(|val| val as f64)
                    },
                    sample: None,
                    format: Some(|_, value, _| format_mhz_value(value)),
                    visible: Some(|_, value, _| value.is_some()),
                    level: Some(|_, value, context| {
                        clock_level(
                            value,
                            context.min_gpu_clock.map(|value| value as f64),
                            context.max_gpu_clock.map(|value| value as f64),
                        )
                    }),
                },
            ),
            (
                StatType::GpuTargetClock,
                StatConfig {
                    label: fl!(I18N, "gpu-clock-target"),
                    unit_label: "MHz",
                    show_peak: true,
                    graphable: true,
                    format_direct: format_direct_0,
                    value: |_, context| {
                        context
                            .stats
                            .clockspeed
                            .target_gpu_clockspeed
                            .map(|val| val as f64)
                    },
                    sample: None,
                    format: Some(|_, value, _| format_current_gfxclk(value)),
                    visible: Some(|_, value, _| value.is_some()),
                    level: None,
                },
            ),
            (
                StatType::GpuUsage,
                StatConfig {
                    label: fl!(I18N, "gpu-usage"),
                    unit_label: "%",
                    show_peak: true,
                    graphable: true,
                    format_direct: format_direct_0,
                    value: |_, context| context.stats.busy_percent.map(|val| val as f64),
                    sample: None,
                    format: Some(|_, value, _| {
                        let value = value.unwrap_or(0.0) as u64;
                        format!("{}%", Mono::uint(value))
                    }),
                    visible: Some(|_, value, _| value.is_some()),
                    level: Some(|_, value, _| value.unwrap_or(0.0) / 100.0),
                },
            ),
            (
                StatType::FanRpm,
                StatConfig {
                    label: fl!(I18N, "stat-fan-rpm"),
                    unit_label: "RPM",
                    show_peak: true,
                    graphable: true,
                    format_direct: format_direct_0,
                    value: |_, context| context.stats.fan.speed_current.map(|val| val as f64),
                    sample: None,
                    format: None,
                    visible: None,
                    level: None,
                },
            ),
            (
                StatType::FanPwm,
                StatConfig {
                    label: fl!(I18N, "stat-fan"),
                    unit_label: "%",
                    show_peak: true,
                    graphable: true,
                    format_direct: format_direct_1,
                    value: |_, context| context.stats.fan.pwm_current.map(|val| val as f64),
                    sample: Some(|_, value, _| value.map(|value| value / u8::MAX as f64 * 100.0)),
                    format: None,
                    visible: None,
                    level: None,
                },
            ),
            (
                StatType::PowerCurrent,
                StatConfig {
                    label: fl!(I18N, "stat-power-draw"),
                    unit_label: "W",
                    show_peak: true,
                    graphable: true,
                    format_direct: format_direct_1,
                    value: |_, context| context.stats.power.current,
                    sample: None,
                    format: None,
                    visible: None,
                    level: None,
                },
            ),
            (
                StatType::PowerAverage,
                StatConfig {
                    label: fl!(I18N, "stat-power-draw-avg"),
                    unit_label: "W",
                    show_peak: true,
                    graphable: true,
                    format_direct: format_direct_1,
                    value: |_, context| context.stats.power.average,
                    sample: None,
                    format: None,
                    visible: None,
                    level: None,
                },
            ),
            (
                StatType::PowerCap,
                StatConfig {
                    label: fl!(I18N, "power-cap"),
                    unit_label: "W",
                    show_peak: false,
                    graphable: true,
                    format_direct: format_direct_0,
                    value: |_, context| context.stats.power.cap_current,
                    sample: None,
                    format: None,
                    visible: None,
                    level: None,
                },
            ),
            (
                StatType::VramClock,
                StatConfig {
                    label: fl!(I18N, "vram-clock"),
                    unit_label: "MHz",
                    show_peak: true,
                    graphable: true,
                    format_direct: format_direct_0,
                    value: |_, context| {
                        context
                            .stats
                            .clockspeed
                            .vram_clockspeed
                            .map(|val| val as f64)
                    },
                    sample: Some(|_, value, context| {
                        value.map(|value| value * context.vram_clock_ratio)
                    }),
                    format: Some(|_, value, context| {
                        formatting::fmt_clockspeed(value, context.vram_clock_ratio)
                    }),
                    visible: Some(|_, value, _| value.is_some()),
                    level: Some(|_, value, context| {
                        clock_level(
                            value,
                            context.min_vram_clock.map(|value| value as f64),
                            context.max_vram_clock.map(|value| value as f64),
                        )
                    }),
                },
            ),
            (
                StatType::VramSize,
                StatConfig {
                    label: fl!(I18N, "stat-vram-size"),
                    unit_label: "MiB",
                    show_peak: false,
                    graphable: true,
                    format_direct: format_direct_0,
                    value: |_, context| context.stats.vram.total.map(|val| val as f64),
                    sample: Some(sample_bytes_as_mib),
                    format: None,
                    visible: None,
                    level: None,
                },
            ),
            (
                StatType::VramUsage,
                StatConfig {
                    label: fl!(I18N, "vram-usage"),
                    unit_label: "MiB",
                    show_peak: true,
                    graphable: true,
                    format_direct: format_direct_0,
                    value: |_, context| context.stats.vram.used.map(|val| val as f64),
                    sample: Some(sample_bytes_as_mib),
                    format: Some(|_, value, _| {
                        formatting::fmt_human_bytes(
                            value.unwrap_or(0.0) as u64,
                            Some(formatting::ByteUnit::Gibibyte),
                        )
                    }),
                    visible: Some(|_, _value, _| true),
                    level: Some(|_, value, context| {
                        value
                            .zip(context.stats.vram.total.map(|total| total as f64))
                            .map(|(used, total)| used / total)
                            .unwrap_or(0.0)
                    }),
                },
            ),
            (
                StatType::GttSize,
                StatConfig {
                    label: fl!(I18N, "stat-gtt-size"),
                    unit_label: "MiB",
                    show_peak: false,
                    graphable: true,
                    format_direct: format_direct_0,
                    value: |_, context| context.stats.vram.gtt_total_usable.map(|val| val as f64),
                    sample: Some(sample_bytes_as_mib),
                    format: None,
                    visible: None,
                    level: None,
                },
            ),
            (
                StatType::GttUsage,
                StatConfig {
                    label: fl!(I18N, "gtt-usage"),
                    unit_label: "MiB",
                    show_peak: true,
                    graphable: true,
                    format_direct: format_direct_0,
                    value: |_, context| context.stats.vram.gtt_used.map(|val| val as f64),
                    sample: Some(sample_bytes_as_mib),
                    format: Some(|_, value, _| {
                        formatting::fmt_human_bytes(
                            value.unwrap_or(0.0) as u64,
                            Some(formatting::ByteUnit::Gibibyte),
                        )
                    }),
                    visible: Some(|_, value, context| {
                        value
                            .zip(
                                context
                                    .stats
                                    .vram
                                    .gtt_total_usable
                                    .map(|total| total as f64),
                            )
                            .is_some()
                    }),
                    level: Some(|_, value, context| {
                        value
                            .zip(
                                context
                                    .stats
                                    .vram
                                    .gtt_total_usable
                                    .map(|total| total as f64),
                            )
                            .map(|(used, total)| used / total)
                            .unwrap_or(0.0)
                    }),
                },
            ),
            (
                StatType::GpuVoltage,
                StatConfig {
                    label: fl!(I18N, "gpu-voltage"),
                    unit_label: "mV",
                    show_peak: true,
                    graphable: true,
                    format_direct: format_direct_0,
                    value: |_, context| context.stats.voltage.gpu.map(|val| val as f64),
                    sample: None,
                    format: Some(|_, value, _| {
                        format!("{} V", Mono::float(value.unwrap_or(0.0) / 1000f64, 3))
                    }),
                    visible: Some(|_, value, _| value.is_some()),
                    level: None,
                },
            ),
            (
                StatType::PowerUsage,
                StatConfig {
                    label: fl!(I18N, "power-usage"),
                    unit_label: "W",
                    show_peak: false,
                    graphable: false,
                    format_direct: format_direct_0,
                    value: |_, context| power_usage_value(context.stats),
                    sample: None,
                    format: Some(|_, value, _| {
                        format!(
                            "{} {}",
                            Mono::float(value.unwrap_or(0.0), 1),
                            fl!(I18N, "watt")
                        )
                    }),
                    visible: Some(|_, value, _| value.is_some()),
                    level: Some(|_, value, context| {
                        value
                            .zip(
                                context
                                    .stats
                                    .power
                                    .cap_current
                                    .filter(|value| *value != 0.0),
                            )
                            .map(|(current, cap)| current / cap)
                            .unwrap_or(0.0)
                    }),
                },
            ),
        ])
    });

    &CONFIGS
}
fn power_usage_value(stats: &DeviceStats) -> Option<f64> {
    stats
        .power
        .current
        .filter(|value| *value != 0.0)
        .or(stats.power.average)
}

fn clock_level(current: Option<f64>, min: Option<f64>, max: Option<f64>) -> f64 {
    match (current, max, min) {
        (Some(cur), Some(max), Some(min)) if max > min => (cur - min).max(0.0) / (max - min),
        _ => 0.0,
    }
}

fn format_mhz_value(value: Option<f64>) -> String {
    formatting::fmt_clockspeed(value, 1.0)
}

fn sample_bytes_as_mib(_: &StatType, value: Option<f64>, _: &StatContext<'_>) -> Option<f64> {
    value.map(|value| value / 1024.0 / 1024.0)
}

fn format_direct_0(value: f64) -> String {
    format!("{value:.0}")
}

fn format_direct_1(value: f64) -> String {
    format!("{value:.1}")
}

fn format_current_gfxclk(value: Option<f64>) -> String {
    if let Some(value) = value {
        // If the APU/GPU does not actually support current_gfxclk, the value will be u16::MAX.
        if value >= u16::MAX as f64 || value == 0.0 {
            fl!(I18N, "missing-stat")
        } else {
            format_mhz_value(Some(value))
        }
    } else {
        fl!(I18N, "missing-stat")
    }
}
