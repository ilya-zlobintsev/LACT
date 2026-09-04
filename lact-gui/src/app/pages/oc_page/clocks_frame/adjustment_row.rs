use crate::{
    APP_BROKER, I18N,
    app::{
        components::adjustment_row::{AdjustmentRow, AdjustmentRowInit, AdjustmentRowMsg},
        msg::AppMsg,
        utils::ext::RelmLaunchable,
    },
};
use i18n_embed_fl::fl;
use lact_schema::request::ClockspeedType;
use relm4::{ComponentController, prelude::FactoryComponent};

pub struct ClockAdjustmentRow {
    row: relm4::Controller<AdjustmentRow>,
    pub(super) is_secondary: bool,
}

pub struct ClocksData {
    pub current: i32,
    pub min: i32,
    pub max: i32,
    pub custom_title: Option<String>,
    pub is_secondary: bool,
    pub step: i32,
}

impl Default for ClocksData {
    fn default() -> Self {
        Self {
            current: 0,
            min: 0,
            max: 0,
            custom_title: None,
            is_secondary: false,
            step: 10,
        }
    }
}

impl ClocksData {
    pub fn new(current: i32, min: i32, max: i32) -> Self {
        Self {
            current,
            min,
            max,
            ..Default::default()
        }
    }
}

#[relm4::factory(pub)]
impl FactoryComponent for ClockAdjustmentRow {
    type ParentWidget = gtk::Box;
    type CommandOutput = ();
    type Init = ClocksData;
    type Input = AdjustmentRowMsg;
    type Output = ();
    type Index = ClockspeedType;

    view! {
        self.row.widget().clone() -> gtk::Box {}
    }

    fn init_model(
        data: Self::Init,
        clock_type: &Self::Index,
        _sender: relm4::FactorySender<Self>,
    ) -> Self {
        let title = data
            .custom_title
            .unwrap_or_else(|| clock_title(*clock_type));
        let row = AdjustmentRow::launch(AdjustmentRowInit {
            title,
            info_text: if *clock_type == ClockspeedType::VoltageBoost {
                fl!(I18N, "gpu-voltage-boost-tooltip")
            } else {
                String::new()
            },
            value: f64::from(data.current),
            lower: f64::from(data.min),
            upper: f64::from(data.max),
            step_increment: f64::from(data.step),
            ..Default::default()
        })
        .connect_receiver(|_, ()| APP_BROKER.send(AppMsg::SettingsChanged));

        Self {
            row,
            is_secondary: data.is_secondary,
        }
    }

    fn update(&mut self, msg: Self::Input, _sender: relm4::FactorySender<Self>) {
        self.row.emit(msg);
    }
}

impl ClockAdjustmentRow {
    pub fn get_configured_value(&self) -> Option<i32> {
        self.row
            .model()
            .get_changed_value()
            .map(|value| value as i32)
    }

    pub fn get_raw_value(&self) -> i32 {
        self.row.model().get_value() as i32
    }
}

fn clock_title(clock_type: ClockspeedType) -> String {
    match clock_type {
        ClockspeedType::MaxCoreClock => fl!(I18N, "max-gpu-clock"),
        ClockspeedType::MaxMemoryClock => fl!(I18N, "max-vram-clock"),
        ClockspeedType::MaxVoltage => fl!(I18N, "max-gpu-voltage"),
        ClockspeedType::MinCoreClock => fl!(I18N, "min-gpu-clock"),
        ClockspeedType::MinMemoryClock => fl!(I18N, "min-vram-clock"),
        ClockspeedType::MinVoltage => fl!(I18N, "min-gpu-voltage"),
        ClockspeedType::VoltageOffset => fl!(I18N, "gpu-voltage-offset"),
        ClockspeedType::VoltageBoost => fl!(I18N, "gpu-voltage-boost"),
        ClockspeedType::GpuClockOffset(pstate) => {
            fl!(I18N, "gpu-pstate-clock-offset", pstate = pstate)
        }
        ClockspeedType::MemClockOffset(pstate) => {
            fl!(I18N, "vram-pstate-clock-offset", pstate = pstate)
        }
        ClockspeedType::GpuVfCurveClock(pstate) => fl!(I18N, "gpu-pstate-clock", pstate = pstate),
        ClockspeedType::MemVfCurveClock(pstate) => fl!(I18N, "mem-pstate-clock", pstate = pstate),
        ClockspeedType::GpuVfCurveVoltage(pstate) => {
            fl!(I18N, "gpu-pstate-clock-voltage", pstate = pstate)
        }
        ClockspeedType::MemVfCurveVoltage(pstate) => {
            fl!(I18N, "mem-pstate-clock-voltage", pstate = pstate)
        }
        ClockspeedType::Reset => unreachable!(),
    }
}
