use super::{ClocksData, clock_title};
use crate::{
    APP_BROKER, I18N,
    app::{
        components::adjustment_row::{AdjustmentRow, AdjustmentRowInit, AdjustmentRowMsg},
        msg::AppMsg,
    },
};
use adw::prelude::*;
use i18n_embed_fl::fl;
use lact_schema::request::ClockspeedType;
use relm4::{css, factory::FactoryHashMap, prelude::FactoryComponent};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClockCategory {
    CoreClock,
    CoreVoltage,
    VramClock,
    CoreCurveClock,
    VramCurveClock,
    CoreCurveVoltage,
    VramCurveVoltage,
    AdvancedClock,
    AdvancedVoltage,
}

impl ClockCategory {
    pub fn from_type(clock_type: ClockspeedType) -> Self {
        match clock_type {
            ClockspeedType::MaxCoreClock
            | ClockspeedType::MinCoreClock
            | ClockspeedType::GpuClockOffset(_) => ClockCategory::CoreClock,
            ClockspeedType::MinVoltage
            | ClockspeedType::MaxVoltage
            | ClockspeedType::VoltageOffset
            | ClockspeedType::VoltageBoost => ClockCategory::CoreVoltage,
            ClockspeedType::MaxMemoryClock
            | ClockspeedType::MinMemoryClock
            | ClockspeedType::MemClockOffset(_) => ClockCategory::VramClock,
            ClockspeedType::GpuVfCurveClock(_) => ClockCategory::CoreCurveClock,
            ClockspeedType::MemVfCurveClock(_) => ClockCategory::VramCurveClock,
            ClockspeedType::GpuVfCurveVoltage(_) => ClockCategory::CoreCurveVoltage,
            ClockspeedType::MemVfCurveVoltage(_) => ClockCategory::VramCurveVoltage,
            ClockspeedType::ClockDomainOffset(_) => ClockCategory::AdvancedClock,
            ClockspeedType::ClockDomainVoltageOffset(_) => ClockCategory::AdvancedVoltage,
            ClockspeedType::Reset => unreachable!(),
        }
    }

    pub fn is_core(&self) -> bool {
        Self::CORE.contains(self)
    }

    pub fn is_vram(&self) -> bool {
        Self::VRAM.contains(self)
    }

    pub fn is_advanced(&self) -> bool {
        Self::ADVANCED.contains(self)
    }

    pub const CORE: [ClockCategory; 4] = [
        ClockCategory::CoreClock,
        ClockCategory::CoreVoltage,
        ClockCategory::CoreCurveClock,
        ClockCategory::CoreCurveVoltage,
    ];

    /// Controls that are not part of the core or VRAM clock story, and which live
    /// in their own section rather than mixed into either column.
    pub const ADVANCED: [ClockCategory; 2] =
        [ClockCategory::AdvancedClock, ClockCategory::AdvancedVoltage];

    pub const VRAM: [ClockCategory; 3] = [
        ClockCategory::VramClock,
        ClockCategory::VramCurveClock,
        ClockCategory::VramCurveVoltage,
    ];
}

pub struct AdjustmentGroup {
    adjustments: FactoryHashMap<ClockspeedType, AdjustmentRow<ClockspeedType>>,
    secondary_clocks: HashSet<ClockspeedType>,
}

impl AdjustmentGroup {
    pub fn is_empty(&self) -> bool {
        self.adjustments.is_empty()
    }

    pub fn has_secondary(&self) -> bool {
        !self.secondary_clocks.is_empty()
    }

    pub fn set_clock(&mut self, clock_type: ClockspeedType, data: ClocksData) {
        self.adjustments.insert(
            clock_type,
            AdjustmentRowInit {
                title: data.custom_title.unwrap_or_else(|| clock_title(clock_type)),
                info_text: if clock_type == ClockspeedType::VoltageBoost {
                    fl!(I18N, "gpu-voltage-boost-tooltip")
                } else {
                    String::new()
                },
                value: f64::from(data.current),
                lower: f64::from(data.min),
                upper: f64::from(data.max),
                step_increment: get_row_step(clock_type),
                ..Default::default()
            },
        );
        if data.is_secondary {
            self.secondary_clocks.insert(clock_type);
        } else {
            self.secondary_clocks.remove(&clock_type);
        }
    }

    pub fn add_size_group(&self, label_group: gtk::SizeGroup, input_group: gtk::SizeGroup) {
        for clock_type in self.adjustments.keys() {
            self.adjustments.send(
                clock_type,
                AdjustmentRowMsg::AddSizeGroup {
                    label_group: label_group.clone(),
                    input_group: input_group.clone(),
                },
            );
        }
    }

    pub fn set_value_ratio(&self, ratio: f64) {
        for clock_type in self.adjustments.keys() {
            self.adjustments
                .send(clock_type, AdjustmentRowMsg::ValueRatio(ratio));
        }
    }

    pub fn toggle_secondary_visibility(
        &self,
        show_secondary: bool,
        show_nvidia_options: bool,
        enable_gpu_locked: bool,
        enable_vram_locked: bool,
        vf_curve_editing: bool,
    ) {
        let mut any_visible = false;

        for key in self.adjustments.keys() {
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
                ClockspeedType::GpuClockOffset(_) if show_nvidia_options && vf_curve_editing => {
                    false
                }
                _ => !self.secondary_clocks.contains(key) || show_secondary,
            };

            any_visible |= show_current;

            self.adjustments
                .send(key, AdjustmentRowMsg::SetVisible(show_current));
        }

        // removes empty card
        self.adjustments.widget().set_visible(any_visible);
    }

    pub fn get_commands(&self) -> Vec<(ClockspeedType, Option<i32>)> {
        self.adjustments
            .iter()
            .map(|(clock_type, row)| {
                (
                    *clock_type,
                    row.get_changed_value().map(|value| value as i32),
                )
            })
            .collect()
    }

    pub fn reset_gpu_clock_offsets(&self) {
        for clock_type in self.adjustments.keys() {
            if matches!(clock_type, ClockspeedType::GpuClockOffset(_)) {
                self.adjustments
                    .send(clock_type, AdjustmentRowMsg::SetValue(0.0));
            }
        }
    }

    pub fn get_raw_value(&self, clock_type: ClockspeedType) -> i32 {
        self.adjustments
            .get(&clock_type)
            .map(|row| row.get_value() as i32)
            .unwrap_or(0)
    }
}

fn get_row_step(clock_type: ClockspeedType) -> f64 {
    match ClockCategory::from_type(clock_type) {
        ClockCategory::CoreClock
        | ClockCategory::VramClock
        | ClockCategory::CoreCurveClock
        | ClockCategory::VramCurveClock => 5.0,
        ClockCategory::CoreVoltage
        | ClockCategory::CoreCurveVoltage
        | ClockCategory::VramCurveVoltage
        | ClockCategory::AdvancedVoltage => 1.0,
        ClockCategory::AdvancedClock => 5.0,
    }
}

#[relm4::factory(pub)]
impl FactoryComponent for AdjustmentGroup {
    type Init = ();
    type Input = ();
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::Box;
    type Index = ClockCategory;

    view! {
        self.adjustments.widget().clone() -> gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 5,
            set_valign: gtk::Align::Start,
            add_css_class: css::CARD,
        }
    }

    fn init_model(_: Self::Init, _: &Self::Index, _: relm4::FactorySender<Self>) -> Self {
        Self {
            adjustments: FactoryHashMap::builder()
                .launch_default()
                .forward(APP_BROKER.sender(), |()| AppMsg::SettingsChanged),
            secondary_clocks: HashSet::new(),
        }
    }
}
