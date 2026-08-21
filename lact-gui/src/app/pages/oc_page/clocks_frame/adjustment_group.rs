use super::adjustment_row::{ClockAdjustmentRow, ClockAdjustmentRowMsg, ClocksData, RowId};
use crate::app::components::oc_adjustment::OcAdjustment;
use gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
use lact_schema::request::ClockspeedType;
use relm4::{css, factory::FactoryHashMap, prelude::FactoryComponent};

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
    adjustments: FactoryHashMap<RowId, ClockAdjustmentRow>,
}

impl AdjustmentGroup {
    pub fn is_empty(&self) -> bool {
        self.adjustments.is_empty()
    }

    pub fn has_secondary(&self) -> bool {
        self.adjustments.values().any(|row| row.is_secondary)
    }

    pub fn set_row(&mut self, id: RowId, data: ClocksData) {
        self.adjustments.insert(id, data);
    }

    /// The adjustment backing a row, so the parent can react to the user moving it.
    ///
    /// The adjustment is owned by the row, so any handler connected to it goes
    /// away together with the row it belongs to.
    pub fn row_adjustment(&self, id: RowId) -> Option<OcAdjustment> {
        self.adjustments.get(&id).map(|row| row.adjustment.clone())
    }

    /// Pushes one offset into every per-domain MSVDD row.
    ///
    /// MSVDD is a single rail shared by all of these domains, so the master row
    /// sets them together; editing one afterwards overrides it for that domain.
    pub fn set_domain_voltage_offsets(&self, offset: i32) {
        for id in self.adjustments.keys() {
            if matches!(id, RowId::Clock(ClockspeedType::ClockDomainVoltageOffset(_))) {
                self.adjustments
                    .send(id, ClockAdjustmentRowMsg::SetValue(offset));
            }
        }
    }

    pub fn add_size_group(&self, label_group: gtk::SizeGroup, input_group: gtk::SizeGroup) {
        for id in self.adjustments.keys() {
            self.adjustments.send(
                id,
                ClockAdjustmentRowMsg::AddSizeGroup {
                    label_group: label_group.clone(),
                    input_group: input_group.clone(),
                },
            );
        }
    }

    pub fn set_value_ratio(&self, ratio: f64) {
        for id in self.adjustments.keys() {
            self.adjustments
                .send(id, ClockAdjustmentRowMsg::ValueRatio(ratio));
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

        for (key, row) in self.adjustments.iter() {
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
                _ => !row.is_secondary || show_secondary,
            };

            any_visible |= show_current;

            self.adjustments
                .send(key, ClockAdjustmentRowMsg::SetVisible(show_current));
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
                RowId::Clock(clock_type) => Some((*clock_type, row.get_configured_value())),
                RowId::MsvddMaster => None,
            })
            .collect()
    }

    pub fn reset_gpu_clock_offsets(&self) {
        for id in self.adjustments.keys() {
            if matches!(id, RowId::Clock(ClockspeedType::GpuClockOffset(_))) {
                self.adjustments
                    .send(id, ClockAdjustmentRowMsg::SetValue(0));
            }
        }
    }

    pub fn get_raw_value(&self, clock_type: ClockspeedType) -> i32 {
        self.adjustments
            .get(&RowId::Clock(clock_type))
            .map(|row| row.get_raw_value())
            .unwrap_or(0)
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
            adjustments: FactoryHashMap::builder().launch_default().detach(),
        }
    }
}
