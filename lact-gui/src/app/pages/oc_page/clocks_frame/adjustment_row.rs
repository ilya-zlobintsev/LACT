use crate::{
    app::{msg::AppMsg, pages::oc_adjustment::OcAdjustment},
    APP_BROKER,
};
use gtk::{
    glib::{object::ObjectExt, SignalHandlerId},
    prelude::{AdjustmentExt, OrientableExt, RangeExt, ScaleExt, WidgetExt},
};
use lact_schema::request::ClockspeedType;
use relm4::{prelude::FactoryComponent, RelmWidgetExt};

pub struct ClockAdjustmentRow {
    clock_type: ClockspeedType,
    value_ratio: f64,
    change_signal: SignalHandlerId,
    adjustment: OcAdjustment,
}

pub struct ClocksData {
    pub current: i32,
    pub min: i32,
    pub max: i32,
}

#[derive(Debug)]
pub enum ClockAdjustmentRowMsg {
    ValueRatio(f64),
}

#[relm4::factory(pub)]
impl FactoryComponent for ClockAdjustmentRow {
    type ParentWidget = gtk::Box;
    type CommandOutput = ();
    type Init = ClocksData;
    type Input = ClockAdjustmentRowMsg;
    type Output = ();
    type Index = ClockspeedType;

    view! {
        gtk::Box {
            gtk::Label {
                set_width_request: 185,
                set_xalign: 0.0,
                set_label: match self.clock_type {
                    ClockspeedType::MaxCoreClock => "Maximum GPU Clock (MHz)",
                    ClockspeedType::MaxMemoryClock => "Maximum VRAM Clock (MHz)",
                    ClockspeedType::MaxVoltage => "Maximum GPU voltage (mV)",
                    ClockspeedType::MinCoreClock => "Minimum GPU Clock (MHz)",
                    ClockspeedType::MinMemoryClock => "Minimum VRAM Clock (MHz)",
                    ClockspeedType::MinVoltage => "Minimum GPU voltage (mV)",
                    ClockspeedType::VoltageOffset => "GPU voltage offset (mV)",
                    ClockspeedType::Reset => unreachable!(),
                },
            },

            gtk::Scale {
                set_adjustment: &self.adjustment,
                set_orientation: gtk::Orientation::Horizontal,
                set_hexpand: true,
                set_digits: 0,
                set_round_digits: 0,
                set_value_pos: gtk::PositionType::Right,
                set_margin_horizontal: 5,
            },

            gtk::SpinButton {
                set_adjustment: &self.adjustment,
                set_width_request: 120,
            },
        }
    }

    fn init_model(
        data: Self::Init,
        clock_type: &Self::Index,
        _sender: relm4::FactorySender<Self>,
    ) -> Self {
        let adjustment = OcAdjustment::new(
            data.current as f64,
            data.min as f64,
            data.max as f64,
            1.0,
            10.0,
            1.0,
        );

        let change_signal = adjustment.connect_value_changed(move |_| {
            APP_BROKER.send(AppMsg::SettingsChanged);
        });

        Self {
            clock_type: *clock_type,
            adjustment,
            change_signal,
            value_ratio: 1.0,
        }
    }

    fn update(&mut self, msg: Self::Input, _sender: relm4::FactorySender<Self>) {
        match msg {
            ClockAdjustmentRowMsg::ValueRatio(ratio) => {
                self.adjustment.block_signal(&self.change_signal);

                let raw_current = self.adjustment.value() / self.value_ratio;
                let raw_min = self.adjustment.lower() / self.value_ratio;
                let raw_max = self.adjustment.upper() / self.value_ratio;

                self.adjustment.set_lower(raw_min * ratio);
                self.adjustment.set_upper(raw_max * ratio);
                self.adjustment.set_initial_value(raw_current * ratio);

                self.value_ratio = ratio;

                self.adjustment.unblock_signal(&self.change_signal);
            }
        }
    }
}

impl ClockAdjustmentRow {
    pub fn get_configured_value(&self) -> Option<i32> {
        self.adjustment
            .get_changed_value(false)
            .map(|value| (value / self.value_ratio) as i32)
    }
}
