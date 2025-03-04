use lact_schema::DeviceStats;
use std::{borrow::Cow, collections::BTreeMap};
use tracing::debug;

#[derive(Default)]
pub struct StatsData {
    stats: BTreeMap<StatType, Vec<(i64, f64)>>,
    throttling: Vec<(i64, String)>,
}

impl StatsData {
    pub fn update(&mut self, stats: &DeviceStats) {
        let time = chrono::Local::now();
        let timestamp = time.timestamp_millis();

        for (name, temperature) in &stats.temps {
            if let Some(value) = temperature.current {
                self.stats
                    .entry(StatType::Temperature(name.to_owned()))
                    .or_default()
                    .push((timestamp, value.into()));
            }
        }

        let stats_values = [
            (
                StatType::GpuClock,
                stats.clockspeed.gpu_clockspeed.map(|val| val as f64),
            ),
            (
                StatType::GpuTargetClock,
                stats.clockspeed.current_gfxclk.map(|val| val as f64),
            ),
            (
                StatType::VramClock,
                stats.clockspeed.vram_clockspeed.map(|val| val as f64),
            ),
            (StatType::PowerAverage, stats.power.average),
            (StatType::PowerCurrent, stats.power.current),
            (StatType::PowerCap, stats.power.current),
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
        ];

        for (stat_type, value) in stats_values {
            if let Some(value) = value {
                debug!("Pushing stat {} {value}", stat_type.display());
                self.stats
                    .entry(stat_type)
                    .or_default()
                    .push((timestamp, value));
            }
        }

        if let Some(throttle_info) = &stats.throttle_info {
            if !throttle_info.is_empty() {
                let type_text: Vec<String> = throttle_info
                    .iter()
                    .map(|(throttle_type, details)| {
                        format!("{throttle_type} ({})", details.join(", "))
                    })
                    .collect();

                let text = type_text.join(", ");
                self.throttling.push((timestamp, text));
            }
        }
    }

    pub fn list_stats(&self) -> impl Iterator<Item = &StatType> {
        self.stats.keys()
    }

    pub fn get_stats<'a>(
        &'a self,
        stats: &'a [StatType],
    ) -> impl Iterator<Item = (&'a StatType, &'a [(i64, f64)])> {
        stats
            .iter()
            .filter_map(|stat_type| Some((stat_type, self.stats.get(stat_type)?.as_slice())))
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
        let maximum_point = self
            .throttling
            .last()
            .map(|(date_time, _)| *date_time)
            .unwrap_or_default();

        self.throttling
            .retain(|(time_point, _)| ((maximum_point - *time_point) / 1000) < last_seconds);
    }
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone)]
pub enum StatType {
    GpuClock,
    GpuTargetClock,
    VramClock,
    Temperature(String),
    FanRpm,
    FanPwm,
    PowerCurrent,
    PowerAverage,
    PowerCap,
}

impl StatType {
    pub fn display(&self) -> Cow<'static, str> {
        match self {
            StatType::GpuClock => "GPU Clock".into(),
            StatType::GpuTargetClock => "GPU Clock (Target)".into(),
            StatType::VramClock => "VRAM Clock".into(),
            StatType::Temperature(name) => format!("Temp ({name})").into(),
            StatType::FanRpm => "Fan RPM".into(),
            StatType::FanPwm => "Fan %".into(),
            StatType::PowerCurrent => "Power Draw".into(),
            StatType::PowerAverage => "Power Draw (Avg)".into(),
            StatType::PowerCap => "Power Cap".into(),
        }
    }
}
