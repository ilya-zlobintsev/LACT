use super::adjustment_row::{ClockAdjustmentRow, ClockAdjustmentRowMsg, ClocksData};
use gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
use lact_schema::request::ClockspeedType;
use relm4::factory::FactoryHashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClockCategory {
    CoreClock,
    CoreVoltage,
    VramClock,
    CoreCurveClock,
    VramCurveClock,
    CoreCurveVoltage,
    VramCurveVoltage,
}

pub const ALL_CATEGORIES: [ClockCategory; 7] = [
    ClockCategory::CoreClock,
    ClockCategory::CoreVoltage,
    ClockCategory::VramClock,
    ClockCategory::CoreCurveClock,
    ClockCategory::VramCurveClock,
    ClockCategory::CoreCurveVoltage,
    ClockCategory::VramCurveVoltage,
];

pub const CORE_CATEGORIES: [ClockCategory; 4] = [
    ClockCategory::CoreClock,
    ClockCategory::CoreVoltage,
    ClockCategory::CoreCurveClock,
    ClockCategory::CoreCurveVoltage,
];

pub const VRAM_CATEGORIES: [ClockCategory; 3] = [
    ClockCategory::VramClock,
    ClockCategory::VramCurveClock,
    ClockCategory::VramCurveVoltage,
];

pub fn clock_category(clock_type: ClockspeedType) -> ClockCategory {
    match clock_type {
        ClockspeedType::MaxCoreClock
        | ClockspeedType::MinCoreClock
        | ClockspeedType::GpuClockOffset(_) => ClockCategory::CoreClock,
        ClockspeedType::MinVoltage | ClockspeedType::MaxVoltage | ClockspeedType::VoltageOffset => {
            ClockCategory::CoreVoltage
        }
        ClockspeedType::MaxMemoryClock
        | ClockspeedType::MinMemoryClock
        | ClockspeedType::MemClockOffset(_) => ClockCategory::VramClock,
        ClockspeedType::GpuVfCurveClock(_) => ClockCategory::CoreCurveClock,
        ClockspeedType::MemVfCurveClock(_) => ClockCategory::VramCurveClock,
        ClockspeedType::GpuVfCurveVoltage(_) => ClockCategory::CoreCurveVoltage,
        ClockspeedType::MemVfCurveVoltage(_) => ClockCategory::VramCurveVoltage,
        ClockspeedType::Reset => unreachable!(),
    }
}

pub struct AdjustmentGroup {
    adjustments: FactoryHashMap<ClockspeedType, ClockAdjustmentRow>,
}

impl AdjustmentGroup {
    pub fn new(_category: ClockCategory) -> Self {
        let adjustments = FactoryHashMap::builder().launch_default().detach();

        let container: &gtk::Box = adjustments.widget();
        container.set_orientation(gtk::Orientation::Vertical);
        container.set_spacing(5);
        container.set_valign(gtk::Align::Start);

        Self { adjustments }
    }

    pub fn widget(&self) -> &gtk::Box {
        self.adjustments.widget()
    }

    pub fn is_empty(&self) -> bool {
        self.adjustments.is_empty()
    }

    pub fn has_secondary(&self) -> bool {
        self.adjustments.values().any(|row| row.is_secondary)
    }

    pub fn set_clock(&mut self, clock_type: ClockspeedType, data: ClocksData) {
        self.adjustments.insert(clock_type, data);
    }

    pub fn clear(&mut self) {
        self.adjustments.clear();
    }

    pub fn add_size_group(&self, label_group: gtk::SizeGroup, input_group: gtk::SizeGroup) {
        for clock_type in self.adjustments.keys() {
            self.adjustments.send(
                clock_type,
                ClockAdjustmentRowMsg::AddSizeGroup {
                    label_group: label_group.clone(),
                    input_group: input_group.clone(),
                },
            );
        }
    }

    pub fn set_value_ratio(&self, ratio: f64) {
        for clock_type in self.adjustments.keys() {
            self.adjustments
                .send(clock_type, ClockAdjustmentRowMsg::ValueRatio(ratio));
        }
    }

    pub fn toggle_secondary_visibility(
        &self,
        show_secondary: bool,
        show_nvidia_options: bool,
        enable_gpu_locked: bool,
        enable_vram_locked: bool,
    ) {
        for (key, row) in self.adjustments.iter() {
            let show_current = match key {
                ClockspeedType::MaxCoreClock | ClockspeedType::MinCoreClock
                    if show_nvidia_options =>
                {
                    enable_gpu_locked
                }
                ClockspeedType::MaxMemoryClock | ClockspeedType::MinMemoryClock
                    if show_nvidia_options =>
                {
                    enable_vram_locked
                }
                _ => !row.is_secondary || show_secondary,
            };

            self.adjustments
                .send(key, ClockAdjustmentRowMsg::SetVisible(show_current));
        }
    }

    pub fn get_commands(&self) -> Vec<(ClockspeedType, Option<i32>)> {
        self.adjustments
            .iter()
            .map(|(clock_type, row)| (*clock_type, row.get_configured_value()))
            .collect()
    }

    pub fn get_raw_value(&self, clock_type: ClockspeedType) -> i32 {
        self.adjustments
            .get(&clock_type)
            .map(|row| row.get_raw_value())
            .unwrap_or(0)
    }
}
