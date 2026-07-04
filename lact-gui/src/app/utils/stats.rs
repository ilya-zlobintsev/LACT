use lact_schema::DeviceStats;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt};

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Deserialize, Serialize)]
pub enum StatKind {
    GpuClock,
    GpuTargetClock,
    GpuUsage,
    Temperature,
    FanRpm,
    FanPwm,
    PowerCurrent,
    PowerAverage,
    PowerCap,
    Power,
    VramClock,
    VramSize,
    VramUsed,
    GttSize,
    GttUsed,
    GpuVoltage,
    Clockspeed,
    Voltage,
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Deserialize, Serialize)]
pub struct StatIdentifier {
    pub kind: StatKind,
    /// Extra label (e.g. sensor name)
    pub label: Option<String>,
}

impl StatIdentifier {
    pub fn with_label(kind: StatKind, label: String) -> Self {
        Self {
            kind,
            label: Some(label),
        }
    }
}

impl fmt::Display for StatIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.as_str().fmt(f)?;

        if let Some(label) = &self.label {
            write!(f, " ({label})")?;
        }

        Ok(())
    }
}

impl From<StatKind> for StatIdentifier {
    fn from(r#type: StatKind) -> Self {
        Self {
            kind: r#type,
            label: None,
        }
    }
}

impl StatKind {
    pub fn as_str(&self) -> &'static str {
        use StatKind::*;
        match self {
            GpuClock => "Clockspeed (GPU)",
            GpuTargetClock => "Clockspeed (GPU Target)",
            GpuVoltage => "GPU Voltage",
            VramClock => "Clockspeed (VRAM)",
            VramSize => "VRAM Size",
            VramUsed => "VRAM Used",
            GttSize => "GTT Size",
            GttUsed => "GTT Used",
            GpuUsage => "GPU Usage",
            Temperature => "Temp",
            Clockspeed => "Clockspeed",
            Voltage => "Voltage",
            Power => "Power",
            FanRpm => "Fan RPM",
            FanPwm => "Fan",
            PowerCurrent => "Power Draw",
            PowerAverage => "Power Draw (Avg)",
            PowerCap => "Power Cap",
        }
    }

    pub fn metric(&self) -> &'static str {
        use StatKind::*;
        match self {
            GpuClock | GpuTargetClock | VramClock | Clockspeed => "MHz",
            VramSize | VramUsed | GttSize | GttUsed => "MiB",
            GpuVoltage | Voltage => "mV",
            Temperature => "℃",
            FanRpm => "RPM",
            FanPwm => "%",
            GpuUsage => "%",
            PowerCurrent | PowerAverage | PowerCap | Power => "W",
        }
    }

    /// How many digits should be formatted
    pub fn precision(&self) -> usize {
        use StatKind::*;
        match self {
            GpuClock | GpuTargetClock | VramClock | Clockspeed => 0,
            FanPwm => 1,
            FanRpm => 0,
            PowerCurrent | PowerAverage | Power => 1,
            PowerCap => 0,
            Temperature => 1,
            GpuUsage | VramSize | VramUsed | GttSize | GttUsed => 0,
            GpuVoltage | Voltage => 0,
        }
    }

    pub fn show_peak(&self) -> bool {
        use StatKind::*;
        !matches!(self, VramSize | PowerCap)
    }
}

#[derive(Debug)]
pub struct StatsHistory {
    stats: BTreeMap<StatIdentifier, Vec<(i64, f64)>>,
    throttling: Vec<Vec<(i64, Vec<String>)>>,
    vram_clock_ratio: f64,
}

impl StatsHistory {
    pub fn update(&mut self, stats: &DeviceStats) {
        let timestamp = jiff::Timestamp::now().as_millisecond();
        self.update_with_timestamp(stats, timestamp);
    }

    pub fn update_with_timestamp(&mut self, stats: &DeviceStats, timestamp: i64) {
        for (name, temperature) in &stats.temps {
            if let Some(value) = temperature.value.current {
                self.stats
                    .entry(StatIdentifier {
                        kind: StatKind::Temperature,
                        label: Some(name.clone()),
                    })
                    .or_default()
                    .push((timestamp, value.into()));
            }
        }

        for (name, value) in &stats.voltage.sensors {
            self.stats
                .entry(StatIdentifier {
                    kind: StatKind::Voltage,
                    label: Some(name.clone()),
                })
                .or_default()
                .push((timestamp, *value as f64));
        }

        for (name, value) in &stats.clockspeed.sensors {
            self.stats
                .entry(StatIdentifier {
                    kind: StatKind::Clockspeed,
                    label: Some(name.clone()),
                })
                .or_default()
                .push((timestamp, *value as f64));
        }

        for (name, value) in &stats.power.sensors {
            self.stats
                .entry(StatIdentifier {
                    kind: StatKind::Power,
                    label: Some(name.clone()),
                })
                .or_default()
                .push((timestamp, *value));
        }

        let stats_values = [
            (
                StatKind::GpuClock,
                stats.clockspeed.gpu_clockspeed.map(|val| val as f64),
            ),
            (
                StatKind::GpuTargetClock,
                stats.clockspeed.target_gpu_clockspeed.map(|val| val as f64),
            ),
            (
                StatKind::VramClock,
                stats
                    .clockspeed
                    .vram_clockspeed
                    .map(|val| val as f64 * self.vram_clock_ratio),
            ),
            (
                StatKind::GpuVoltage,
                stats.voltage.gpu.map(|val| val as f64),
            ),
            (StatKind::PowerAverage, stats.power.average),
            (StatKind::PowerCurrent, stats.power.current),
            (StatKind::PowerCap, stats.power.cap_current),
            (
                StatKind::FanPwm,
                stats
                    .fan
                    .pwm_current
                    .map(|val| (val as f64) / u8::MAX as f64 * 100.0),
            ),
            (
                StatKind::FanRpm,
                stats.fan.speed_current.map(|val| val as f64),
            ),
            (StatKind::GpuUsage, stats.busy_percent.map(|val| val as f64)),
            (
                StatKind::VramSize,
                stats.vram.total.map(|val| (val / 1024 / 1024) as f64),
            ),
            (
                StatKind::VramUsed,
                stats.vram.used.map(|val| (val / 1024 / 1024) as f64),
            ),
            (
                StatKind::GttSize,
                stats
                    .vram
                    .gtt_total_usable
                    .map(|val| (val / 1024 / 1024) as f64),
            ),
            (
                StatKind::GttUsed,
                stats.vram.gtt_used.map(|val| (val / 1024 / 1024) as f64),
            ),
        ];

        for (stat_type, value) in stats_values {
            if let Some(value) = value {
                self.stats
                    .entry(stat_type.into())
                    .or_default()
                    .push((timestamp, value));
            }
        }

        let is_throttling = stats
            .throttle_info
            .as_ref()
            .is_some_and(|info| !info.is_empty());

        if is_throttling {
            let text: Vec<String> = stats
                .throttle_info
                .iter()
                .flatten()
                .map(|(throttle_type, details)| {
                    if details.is_empty() {
                        throttle_type.clone()
                    } else {
                        format!("{throttle_type} ({})", details.join(","))
                    }
                })
                .collect();

            if let Some(last_section) = self.throttling.last_mut() {
                last_section.push((timestamp, text));
            } else {
                self.throttling.push(vec![(timestamp, text)]);
            }
        } else if self
            .throttling
            .last()
            .is_none_or(|last_section| !last_section.is_empty())
        {
            self.throttling.push(vec![]);
        };
    }

    pub fn set_vram_clock_ratio(&mut self, ratio: f64) {
        self.vram_clock_ratio = ratio;
    }

    pub fn list_stats(&self) -> impl Iterator<Item = &StatIdentifier> {
        self.stats.keys()
    }

    pub fn throttling_sections(&self) -> &[Vec<(i64, Vec<String>)>] {
        &self.throttling
    }

    pub fn get_stats<'a>(
        &'a self,
        stats: &'a [StatIdentifier],
    ) -> impl Iterator<Item = (&'a StatIdentifier, &'a [(i64, f64)])> {
        stats
            .iter()
            .filter_map(|stat_type| Some((stat_type, self.stats.get(stat_type)?.as_slice())))
    }

    pub fn all_stats(&self) -> &BTreeMap<StatIdentifier, Vec<(i64, f64)>> {
        &self.stats
    }

    pub fn first_timestamp(&self) -> Option<i64> {
        self.stats
            .values()
            .filter_map(|points| points.first())
            .map(|(timestamp, _)| *timestamp)
            .min()
    }

    pub fn last_timestamp(&self) -> Option<i64> {
        self.stats
            .values()
            .filter_map(|points| points.last())
            .map(|(timestamp, _)| *timestamp)
            .max()
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn trim(&mut self, last_seconds: i64) {
        // Limit data to N seconds
        for data in self.stats.values_mut() {
            let maximum_point = data
                .last()
                .map(|(date_time, _)| *date_time)
                .unwrap_or_default();

            data.retain(|(time_point, _)| ((maximum_point - *time_point) / 1000) < last_seconds);
        }

        self.stats.retain(|_, data| !data.is_empty());

        // Limit data to N seconds
        let last_timestamp = self
            .stats
            .iter()
            .flat_map(|(_, stats)| stats)
            .map(|(date_time, _)| *date_time)
            .next_back()
            .unwrap_or_default();

        self.throttling.retain(|section| {
            if let Some((last_time_point, _)) = section.last() {
                ((last_timestamp - last_time_point) / 1000) < last_seconds
            } else {
                true
            }
        });
    }
}

impl Default for StatsHistory {
    fn default() -> Self {
        Self {
            stats: BTreeMap::new(),
            throttling: Vec::new(),
            vram_clock_ratio: 1.0,
        }
    }
}
