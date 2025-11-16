use lact_schema::DeviceStats;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::BTreeMap};

#[derive(Default, Debug)]
pub struct StatsData {
    stats: BTreeMap<StatType, Vec<(i64, f64)>>,
    throttling: Vec<Vec<(i64, Vec<String>)>>,
}

impl StatsData {
    pub fn update(&mut self, stats: &DeviceStats, vram_clock_ratio: f64) {
        let time = chrono::Local::now().naive_local();
        let timestamp = time.and_utc().timestamp_millis();
        self.update_with_timestamp(stats, vram_clock_ratio, timestamp);
    }

    pub fn update_with_timestamp(
        &mut self,
        stats: &DeviceStats,
        vram_clock_ratio: f64,
        timestamp: i64,
    ) {
        for (name, temperature) in &stats.temps {
            if let Some(value) = temperature.value.current {
                self.stats
                    .entry(StatType::Temperature(name.to_owned()))
                    .or_default()
                    .push((timestamp, value.into()));
            }
        }

        for (name, value) in &stats.voltage.sensors {
            self.stats
                .entry(StatType::Voltage(name.clone()))
                .or_default()
                .push((timestamp, *value as f64));
        }

        let stats_values = [
            (
                StatType::GpuClock,
                stats.clockspeed.gpu_clockspeed.map(|val| val as f64),
            ),
            (
                StatType::GpuTargetClock,
                stats.clockspeed.target_gpu_clockspeed.map(|val| val as f64),
            ),
            (
                StatType::VramClock,
                stats
                    .clockspeed
                    .vram_clockspeed
                    .map(|val| val as f64 * vram_clock_ratio),
            ),
            (
                StatType::GpuVoltage,
                stats.voltage.gpu.map(|val| val as f64),
            ),
            (StatType::PowerAverage, stats.power.average),
            (StatType::PowerCurrent, stats.power.current),
            (StatType::PowerCap, stats.power.cap_current),
            (
                StatType::FanPwm,
                stats
                    .fan
                    .pwm_current
                    .map(|val| (val as f64) / u8::MAX as f64 * 100.0),
            ),
            (
                StatType::FanRpm,
                stats.fan.speed_current.map(|val| val as f64),
            ),
            (StatType::GpuUsage, stats.busy_percent.map(|val| val as f64)),
            (
                StatType::VramSize,
                stats.vram.total.map(|val| (val / 1024 / 1024) as f64),
            ),
            (
                StatType::VramUsed,
                stats.vram.used.map(|val| (val / 1024 / 1024) as f64),
            ),
        ];

        for (stat_type, value) in stats_values {
            if let Some(value) = value {
                self.stats
                    .entry(stat_type)
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

    pub fn list_stats(&self) -> impl Iterator<Item = &StatType> {
        self.stats.keys()
    }

    pub fn throttling_sections(&self) -> &[Vec<(i64, Vec<String>)>] {
        &self.throttling
    }

    pub fn get_stats<'a>(
        &'a self,
        stats: &'a [StatType],
    ) -> impl Iterator<Item = (&'a StatType, &'a [(i64, f64)])> {
        stats
            .iter()
            .filter_map(|stat_type| Some((stat_type, self.stats.get(stat_type)?.as_slice())))
    }

    pub fn all_stats(&self) -> &BTreeMap<StatType, Vec<(i64, f64)>> {
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
        self.stats.clear();
        self.throttling.clear();
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
    VramClock,
    VramSize,
    VramUsed,
    GpuVoltage,
    Voltage(String),
}

impl StatType {
    pub fn display(&self) -> Cow<'static, str> {
        use StatType::*;
        match self {
            GpuClock => "GPU Clock".into(),
            GpuTargetClock => "GPU Clock (Target)".into(),
            GpuVoltage => "GPU Voltage".into(),
            VramClock => "VRAM Clock".into(),
            VramSize => "VRAM Size".into(),
            VramUsed => "VRAM Used".into(),
            GpuUsage => "GPU Usage".into(),
            Temperature(name) => format!("Temp ({name})").into(),
            Voltage(name) => format!("Voltage ({name})").into(),
            FanRpm => "Fan RPM".into(),
            FanPwm => "Fan".into(),
            PowerCurrent => "Power Draw".into(),
            PowerAverage => "Power Draw (Avg)".into(),
            PowerCap => "Power Cap".into(),
        }
    }

    pub fn metric(&self) -> &'static str {
        use StatType::*;
        match self {
            GpuClock | GpuTargetClock | VramClock => "MHz",
            VramSize | VramUsed => "MiB",
            GpuVoltage | Voltage(_) => "mV",
            Temperature(_) => "℃",
            FanRpm => "RPM",
            FanPwm => "%",
            GpuUsage => "%",
            PowerCurrent | PowerAverage | PowerCap => "W",
        }
    }

    pub fn show_peak(&self) -> bool {
        use StatType::*;
        !matches!(self, VramSize | PowerCap)
    }
}
