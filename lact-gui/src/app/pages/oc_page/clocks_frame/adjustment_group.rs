use super::{ClocksData, clock_title};
use crate::{
    APP_BROKER, I18N,
    app::{
        components::{
            adjustment_row::{AdjustmentRow, AdjustmentRowInit, AdjustmentRowMsg},
            adjustment_value::AdjustmentValue,
        },
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

/// Identifies a row within a group of adjustments.
///
/// Most rows map one to one onto a clockspeed the daemon can set. The MSVDD
/// master is the exception: it only exists in the GUI, where it drives the
/// per-domain offset rows, so it has no clockspeed type of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RowId {
    Clock(ClockspeedType),
    MsvddMaster,
}

impl ClockCategory {
    pub fn from_row(id: RowId) -> Self {
        match id {
            RowId::MsvddMaster => ClockCategory::AdvancedVoltage,
            RowId::Clock(clock_type) => Self::from_type(clock_type),
        }
    }

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
            ClockspeedType::XbarRatio => ClockCategory::AdvancedClock,
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
    adjustments: FactoryHashMap<RowId, AdjustmentRow<RowId>>,
    secondary_clocks: HashSet<RowId>,
}

impl AdjustmentGroup {
    pub fn is_empty(&self) -> bool {
        self.adjustments.is_empty()
    }

    pub fn has_secondary(&self) -> bool {
        !self.secondary_clocks.is_empty()
    }

    pub fn set_row(&mut self, id: RowId, data: ClocksData) {
        self.adjustments.insert(
            id,
            AdjustmentRowInit {
                title: data.custom_title.unwrap_or_else(|| row_title(id)),
                info_text: row_info_text(id),
                value: f64::from(data.current),
                lower: f64::from(data.min),
                upper: f64::from(data.max),
                step_increment: get_row_step(id),
                ..Default::default()
            },
        );
        if data.is_secondary {
            self.secondary_clocks.insert(id);
        } else {
            self.secondary_clocks.remove(&id);
        }
    }

    /// The adjustment backing a row, so the parent can react to the user moving it.
    ///
    /// The adjustment is owned by the row, so any handler connected to it goes
    /// away together with the row it belongs to.
    pub fn row_adjustment(&self, id: RowId) -> Option<AdjustmentValue> {
        self.adjustments.get(&id).map(AdjustmentRow::adjustment)
    }

    /// Pushes one offset into every per-domain MSVDD row.
    ///
    /// MSVDD is a single rail shared by all of these domains, so the master row
    /// sets them together; editing one afterwards overrides it for that domain.
    pub fn set_domain_voltage_offsets(&self, offset: i32) {
        for id in self.adjustments.keys() {
            if matches!(id, RowId::Clock(ClockspeedType::ClockDomainVoltageOffset(_))) {
                self.adjustments
                    .send(id, AdjustmentRowMsg::SetValue(f64::from(offset)));
            }
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
                RowId::Clock(ClockspeedType::MaxCoreClock | ClockspeedType::MinCoreClock)
                    if show_nvidia_options =>
                {
                    enable_gpu_locked
                }
                RowId::Clock(ClockspeedType::MaxMemoryClock | ClockspeedType::MinMemoryClock)
                    if show_nvidia_options =>
                {
                    enable_vram_locked
                }
                RowId::Clock(ClockspeedType::GpuClockOffset(_))
                    if show_nvidia_options && vf_curve_editing =>
                {
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

    /// Rows that carry a value the daemon can set, paired with their new value.
    ///
    /// Rows that only exist in the GUI are skipped; they act through the rows
    /// they drive rather than being applied themselves.
    pub fn get_commands(&self) -> Vec<(ClockspeedType, Option<i32>)> {
        self.adjustments
            .iter()
            .filter_map(|(id, row)| match id {
                RowId::Clock(clock_type) => Some((
                    *clock_type,
                    row.get_changed_value().map(|value| value as i32),
                )),
                RowId::MsvddMaster => None,
            })
            .collect()
    }

    pub fn reset_gpu_clock_offsets(&self) {
        for id in self.adjustments.keys() {
            if matches!(id, RowId::Clock(ClockspeedType::GpuClockOffset(_))) {
                self.adjustments.send(id, AdjustmentRowMsg::SetValue(0.0));
            }
        }
    }

    pub fn get_raw_value(&self, id: RowId) -> i32 {
        self.adjustments
            .get(&id)
            .map(|row| row.get_value() as i32)
            .unwrap_or(0)
    }
}

fn row_title(id: RowId) -> String {
    match id {
        RowId::Clock(clock_type) => clock_title(clock_type),
        RowId::MsvddMaster => fl!(I18N, "msvdd-offset"),
    }
}

fn row_info_text(id: RowId) -> String {
    match id {
        RowId::Clock(ClockspeedType::VoltageBoost) => fl!(I18N, "gpu-voltage-boost-tooltip"),
        RowId::Clock(ClockspeedType::XbarRatio) => fl!(I18N, "xbar-ratio-tooltip"),
        RowId::MsvddMaster => fl!(I18N, "msvdd-offset-tooltip"),
        _ => String::new(),
    }
}

fn get_row_step(id: RowId) -> f64 {
    // A percentage, so it does not step like the clocks it sits next to
    if id == RowId::Clock(ClockspeedType::XbarRatio) {
        return 1.0;
    }

    match ClockCategory::from_row(id) {
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
