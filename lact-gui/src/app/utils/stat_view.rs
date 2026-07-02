use i18n_embed_fl::fl;
use lact_schema::DeviceStats;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::LazyLock};

use crate::I18N;

use super::formatting::{self, Mono};

pub(crate) type StatConfigMap = HashMap<StatType, StatConfig>;
type SampleFn = fn(&StatType, &StatContext<'_>) -> Option<f64>;
type FormatFn = fn(&StatType, &StatContext<'_>) -> String;
type VisibleFn = fn(&StatType, &StatContext<'_>) -> bool;
type LevelFn = fn(&StatType, &StatContext<'_>) -> f64;

#[derive(Debug, Clone)]
pub(crate) struct StatConfig {
    pub label: String,
    pub unit_label: &'static str,
    pub show_peak: bool,
    sample: SampleFn,
    format: Option<FormatFn>,
    visible: Option<VisibleFn>,
    level: Option<LevelFn>,
}

impl StatConfig {
    pub(crate) fn sample(&self, stat_type: &StatType, context: &StatContext<'_>) -> Option<f64> {
        (self.sample)(stat_type, context)
    }

    pub(crate) fn format(&self, stat_type: &StatType, context: &StatContext<'_>) -> Option<String> {
        self.format.map(|format| format(stat_type, context))
    }

    pub(crate) fn visible(&self, stat_type: &StatType, context: &StatContext<'_>) -> Option<bool> {
        self.visible.map(|visible| visible(stat_type, context))
    }

    pub(crate) fn level(&self, stat_type: &StatType, context: &StatContext<'_>) -> Option<f64> {
        self.level.map(|level| level(stat_type, context))
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
    VramUsed,
    GttSize,
    GttUsed,
    GpuVoltage,
    Clockspeed(String),
    Voltage(String),,
    Throttling,
    Temperatures,
    VramUsage,
    GttUsage,
    PowerUsage,
    FanSpeed,
}

impl StatType {
    pub(crate) fn precision(&self) -> usize {
        use StatType::*;
        match self {
            GpuClock | GpuTargetClock | VramClock | Clockspeed(_) => 0,
            FanPwm => 1,
            FanRpm => 0,
            PowerCurrent | PowerAverage | Power(_) => 1,
            PowerCap => 0,
            Temperature(_) => 1,
            GpuUsage | VramSize | VramUsed | GttSize | GttUsed => 0,
            GpuVoltage | Voltage(_) => 0,
            Throttling | Temperatures | VramUsage | GttUsage | PowerUsage
            | FanSpeed => 0,
        }
    }
}

pub(crate) fn build_stat_config_map(context: &StatContext<'_>) -> StatConfigMap {
    let mut configs = static_stat_configs().clone();

    for name in context.stats.temps.keys() {
        let stat_type = StatType::Temperature(name.clone());
        let config = StatConfig {
            label: format!("{} ({name})", fl!(I18N, "gpu-temp")),
            unit_label: "℃",
            show_peak: true,
            sample: |stat_type, context| {
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
            sample: |stat_type, context| {
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
            format: None,
            visible: None,
            level: None,
        };
        configs.insert(stat_type, config);
    }

    for name in context.stats.clockspeed.sensors.keys() {
        let stat_type = StatType::Clockspeed(name.clone());
        let config = StatConfig {
            label: format!("Clockspeed ({name})"),
            unit_label: "MHz",
            show_peak: true,
            sample: |stat_type, context| {
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
            sample: |stat_type, context| {
                let StatType::Power(name) = stat_type else {
                    return None;
                };

                context.stats.power.sensors.get(name).copied()
            },
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
                    sample: |_, context| {
                        context
                            .stats
                            .clockspeed
                            .gpu_clockspeed
                            .map(|val| val as f64)
                    },
                    format: Some(|_, context| {
                        formatting::fmt_clockspeed(context.stats.clockspeed.gpu_clockspeed, 1.0)
                    }),
                    visible: Some(|_, context| context.stats.clockspeed.gpu_clockspeed.is_some()),
                    level: Some(|_, context| {
                        clock_level(
                            context.stats.clockspeed.gpu_clockspeed,
                            context.min_gpu_clock,
                            context.max_gpu_clock,
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
                    sample: |_, context| {
                        context
                            .stats
                            .clockspeed
                            .target_gpu_clockspeed
                            .map(|val| val as f64)
                    },
                    format: Some(|_, context| {
                        format_current_gfxclk(context.stats.clockspeed.target_gpu_clockspeed)
                    }),
                    visible: Some(|_, context| {
                        context.stats.clockspeed.target_gpu_clockspeed.is_some()
                    }),
                    level: None,
                },
            ),
            (
                StatType::GpuUsage,
                StatConfig {
                    label: fl!(I18N, "gpu-usage"),
                    unit_label: "%",
                    show_peak: true,
                    sample: |_, context| context.stats.busy_percent.map(|val| val as f64),
                    format: Some(|_, context| {
                        format!("{}%", Mono::uint(context.stats.busy_percent.unwrap_or(0)))
                    }),
                    visible: Some(|_, context| context.stats.busy_percent.is_some()),
                    level: Some(|_, context| {
                        context.stats.busy_percent.unwrap_or(0) as f64 / 100.0
                    }),
                },
            ),
            (
                StatType::FanRpm,
                StatConfig {
                    label: "Fan RPM".into(),
                    unit_label: "RPM",
                    show_peak: true,
                    sample: |_, context| context.stats.fan.speed_current.map(|val| val as f64),
                    format: None,
                    visible: None,
                    level: None,
                },
            ),
            (
                StatType::FanPwm,
                StatConfig {
                    label: "Fan".into(),
                    unit_label: "%",
                    show_peak: true,
                    sample: |_, context| {
                        context
                            .stats
                            .fan
                            .pwm_current
                            .map(|val| (val as f64) / u8::MAX as f64 * 100.0)
                    },
                    format: None,
                    visible: None,
                    level: None,
                },
            ),
            (
                StatType::PowerCurrent,
                StatConfig {
                    label: "Power Draw".into(),
                    unit_label: "W",
                    show_peak: true,
                    sample: |_, context| context.stats.power.current,
                    format: None,
                    visible: None,
                    level: None,
                },
            ),
            (
                StatType::PowerAverage,
                StatConfig {
                    label: "Power Draw (Avg)".into(),
                    unit_label: "W",
                    show_peak: true,
                    sample: |_, context| context.stats.power.average,
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
                    sample: |_, context| context.stats.power.cap_current,
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
                    sample: |_, context| {
                        context
                            .stats
                            .clockspeed
                            .vram_clockspeed
                            .map(|val| val as f64 * context.vram_clock_ratio)
                    },
                    format: Some(|_, context| {
                        formatting::fmt_clockspeed(
                            context.stats.clockspeed.vram_clockspeed,
                            context.vram_clock_ratio,
                        )
                    }),
                    visible: Some(|_, context| context.stats.clockspeed.vram_clockspeed.is_some()),
                    level: Some(|_, context| {
                        clock_level(
                            context.stats.clockspeed.vram_clockspeed,
                            context.min_vram_clock,
                            context.max_vram_clock,
                        )
                    }),
                },
            ),
            (
                StatType::VramSize,
                StatConfig {
                    label: "VRAM Size".into(),
                    unit_label: "MiB",
                    show_peak: false,
                    sample: |_, context| {
                        context
                            .stats
                            .vram
                            .total
                            .map(|val| (val / 1024 / 1024) as f64)
                    },
                    format: None,
                    visible: None,
                    level: None,
                },
            ),
            (
                StatType::VramUsed,
                StatConfig {
                    label: "VRAM Used".into(),
                    unit_label: "MiB",
                    show_peak: true,
                    sample: |_, context| {
                        context
                            .stats
                            .vram
                            .used
                            .map(|val| (val / 1024 / 1024) as f64)
                    },
                    format: None,
                    visible: None,
                    level: None,
                },
            ),
            (
                StatType::GttSize,
                StatConfig {
                    label: "GTT Size".into(),
                    unit_label: "MiB",
                    show_peak: false,
                    sample: |_, context| {
                        context
                            .stats
                            .vram
                            .gtt_total_usable
                            .map(|val| (val / 1024 / 1024) as f64)
                    },
                    format: None,
                    visible: None,
                    level: None,
                },
            ),
            (
                StatType::GttUsed,
                StatConfig {
                    label: "GTT Used".into(),
                    unit_label: "MiB",
                    show_peak: true,
                    sample: |_, context| {
                        context
                            .stats
                            .vram
                            .gtt_used
                            .map(|val| (val / 1024 / 1024) as f64)
                    },
                    format: None,
                    visible: None,
                    level: None,
                },
            ),
            (
                StatType::GpuVoltage,
                StatConfig {
                    label: fl!(I18N, "gpu-voltage"),
                    unit_label: "mV",
                    show_peak: true,
                    sample: |_, context| context.stats.voltage.gpu.map(|val| val as f64),
                    format: Some(|_, context| {
                        format!(
                            "{} V",
                            Mono::float(context.stats.voltage.gpu.unwrap_or(0) as f64 / 1000f64, 3)
                        )
                    }),
                    visible: Some(|_, context| context.stats.voltage.gpu.is_some()),
                    level: None,
                },
            ),
            (
                StatType::Throttling,
                StatConfig {
                    label: fl!(I18N, "throttling"),
                    unit_label: "",
                    show_peak: false,
                    sample: |_, _| None,
                    format: Some(|_, context| formatting::fmt_throttling_text(context.stats)),
                    visible: Some(|_, _| true),
                    level: None,
                },
            ),
            (
                StatType::Temperatures,
                StatConfig {
                    label: fl!(I18N, "gpu-temp"),
                    unit_label: "℃",
                    show_peak: false,
                    sample: |_, _| None,
                    format: Some(|_, context| {
                        let (primary, _) = formatting::fmt_temperature_text(context.stats);
                        if primary.is_empty() {
                            fl!(I18N, "missing-stat")
                        } else {
                            primary.join(", ")
                        }
                    }),
                    visible: Some(|_, _| true),
                    level: None,
                },
            ),
            (
                StatType::VramUsage,
                StatConfig {
                    label: fl!(I18N, "vram-usage"),
                    unit_label: "",
                    show_peak: false,
                    sample: |_, _| None,
                    format: Some(|_, context| {
                        formatting::fmt_human_bytes(
                            context.stats.vram.used.unwrap_or(0),
                            Some(formatting::ByteUnit::Gibibyte),
                        )
                    }),
                    visible: Some(|_, _| true),
                    level: Some(|_, context| {
                        context
                            .stats
                            .vram
                            .used
                            .zip(context.stats.vram.total)
                            .map(|(used, total)| used as f64 / total as f64)
                            .unwrap_or(0.0)
                    }),
                },
            ),
            (
                StatType::PowerUsage,
                StatConfig {
                    label: fl!(I18N, "power-usage"),
                    unit_label: "W",
                    show_peak: false,
                    sample: |_, _| None,
                    format: Some(|_, context| {
                        format!(
                            "{} {}",
                            Mono::float(power_usage_value(context.stats).unwrap_or(0.0), 1),
                            fl!(I18N, "watt")
                        )
                    }),
                    visible: Some(|_, context| {
                        context.stats.power.average.is_some()
                            || context.stats.power.current.is_some()
                    }),
                    level: Some(|_, context| {
                        power_usage_value(context.stats)
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
            (
                StatType::GttUsage,
                StatConfig {
                    label: fl!(I18N, "gtt-usage"),
                    unit_label: "",
                    show_peak: false,
                    sample: |_, _| None,
                    format: Some(|_, context| {
                        formatting::fmt_human_bytes(
                            context.stats.vram.gtt_used.unwrap_or(0),
                            Some(formatting::ByteUnit::Gibibyte),
                        )
                    }),
                    visible: Some(|_, context| context.stats.vram.gtt_used.is_some()),
                    level: Some(|_, context| {
                        context
                            .stats
                            .vram
                            .gtt_used
                            .zip(context.stats.vram.gtt_total_usable)
                            .map(|(used, total)| used as f64 / total as f64)
                            .unwrap_or(0.0)
                    }),
                },
            ),
            (
                StatType::FanSpeed,
                StatConfig {
                    label: fl!(I18N, "fan-speed"),
                    unit_label: "",
                    show_peak: false,
                    sample: |_, _| None,
                    format: Some(|_, context| {
                        formatting::fmt_fan_speed(context.stats, true)
                            .unwrap_or_else(|| fl!(I18N, "missing-stat"))
                    }),
                    visible: Some(|_, context| {
                        context.stats.fan.pwm_current.is_some()
                            || context.stats.fan.speed_current.is_some()
                    }),
                    level: Some(|_, context| {
                        context
                            .stats
                            .fan
                            .pwm_current
                            .map(|pwm| pwm as f64 / u8::MAX as f64)
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

fn clock_level(current: Option<u64>, min: Option<u64>, max: Option<u64>) -> f64 {
    match (current, max, min) {
        (Some(cur), Some(max), Some(min)) if max > min => {
            cur.saturating_sub(min) as f64 / max.saturating_sub(min) as f64
        }
        _ => 0.0,
    }
}

fn format_current_gfxclk(value: Option<u64>) -> String {
    if let Some(value) = value {
        // If the APU/GPU does not actually support current_gfxclk, the value will be u16::MAX.
        if value >= u16::MAX as u64 || value == 0 {
            fl!(I18N, "missing-stat")
        } else {
            formatting::fmt_clockspeed(Some(value), 1.0)
        }
    } else {
        fl!(I18N, "missing-stat")
    }
}
