use crate::app::utils::stat_view::{StatContext, StatMetadata, StatType, build_stat_config_map};
use lact_schema::DeviceStats;
use std::collections::BTreeMap;

#[derive(Default, Debug)]
pub struct StatsData {
    stats: BTreeMap<StatType, Vec<(i64, f64)>>,
    metadata: BTreeMap<StatType, StatMetadata>,
    throttling: Vec<Vec<(i64, Vec<String>)>>,
}

impl StatsData {
    pub fn update(&mut self, stats: &DeviceStats, vram_clock_ratio: f64) {
        let timestamp = jiff::Timestamp::now().as_millisecond();
        self.update_with_timestamp(stats, vram_clock_ratio, timestamp);
    }

    pub fn update_with_timestamp(
        &mut self,
        stats: &DeviceStats,
        vram_clock_ratio: f64,
        timestamp: i64,
    ) {
        let context = StatContext {
            stats,
            gpu_model: "",
            vram_clock_ratio,
            max_gpu_clock: None,
            max_vram_clock: None,
            min_gpu_clock: None,
            min_vram_clock: None,
        };
        let stat_configs = build_stat_config_map(&context);

        for (stat_type, config) in stat_configs {
            let Some(value) = config.sample(&stat_type, &context) else {
                continue;
            };

            self.metadata
                .insert(stat_type.clone(), StatMetadata::from(&config));
            self.stats
                .entry(stat_type)
                .or_default()
                .push((timestamp, value));
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

    pub(crate) fn stat_metadata(&self, stat_type: &StatType) -> Option<&StatMetadata> {
        self.metadata.get(stat_type)
    }

    pub fn throttling_sections(&self) -> &[Vec<(i64, Vec<String>)>] {
        &self.throttling
    }

    pub(crate) fn get_stats<'a>(
        &'a self,
        stats: &'a [StatType],
    ) -> impl Iterator<Item = (&'a StatType, &'a StatMetadata, &'a [(i64, f64)])> {
        stats.iter().filter_map(|stat_type| {
            Some((
                stat_type,
                self.metadata.get(stat_type)?,
                self.stats.get(stat_type)?.as_slice(),
            ))
        })
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
        self.metadata.clear();
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
