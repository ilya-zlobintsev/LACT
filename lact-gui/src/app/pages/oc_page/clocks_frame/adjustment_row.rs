use crate::{
    APP_BROKER, I18N,
    app::{msg::AppMsg, pages::oc_adjustment::OcAdjustment},
};
use gtk::prelude::BoxExt;
use gtk::{
    glib::{
        SignalHandlerId,
        object::{Cast, ObjectExt},
        types::StaticType,
    },
    prelude::{AdjustmentExt, EventControllerExt, OrientableExt, RangeExt, ScaleExt, WidgetExt},
};
use i18n_embed_fl::fl;
use lact_schema::request::ClockspeedType;
use relm4::{prelude::FactoryComponent};

pub struct ClockAdjustmentRow {
    clock_type: ClockspeedType,
    custom_title: Option<String>,
    value_ratio: f64,
    change_signal: SignalHandlerId,
    adjustment: OcAdjustment,
    pub(super) is_secondary: bool,
}

pub struct ClocksData {
    pub current: i32,
    pub min: i32,
    pub max: i32,
    pub custom_title: Option<String>,
    pub is_secondary: bool,
}

impl ClocksData {
    pub fn new(current: i32, min: i32, max: i32) -> Self {
        Self {
            current,
            min,
            max,
            is_secondary: false,
            custom_title: None,
        }
    }
}

#[derive(Debug)]
pub enum ClockAdjustmentRowMsg {
    ValueRatio(f64),
    SetVisible(bool),
    AddSizeGroup {
        label_group: gtk::SizeGroup,
        input_group: gtk::SizeGroup,
    },
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
        #[name = "root_box"]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 5,

                #[name = "title_label"]
                gtk::Label {
                    set_xalign: 0.0,
                    #[watch]
                    set_markup: &match &self.custom_title {
                        Some(title) => title.clone(),
                        None => {
                            match self.clock_type {
                                ClockspeedType::MaxCoreClock => fl!(I18N, "max-gpu-clock"),
                                ClockspeedType::MaxMemoryClock => fl!(I18N, "max-vram-clock"),
                                ClockspeedType::MaxVoltage => fl!(I18N, "max-gpu-voltage"),
                                ClockspeedType::MinCoreClock => fl!(I18N, "min-gpu-clock"),
                                ClockspeedType::MinMemoryClock => fl!(I18N, "min-vram-clock"),
                                ClockspeedType::MinVoltage => fl!(I18N, "min-gpu-voltage"),
                                ClockspeedType::VoltageOffset => fl!(I18N, "gpu-voltage-offset"),
                                ClockspeedType::GpuClockOffset(pstate) => fl!(I18N, "gpu-pstate-clock-offset", pstate = pstate),
                                ClockspeedType::MemClockOffset(pstate) => fl!(I18N, "vram-pstate-clock-offset", pstate = pstate),
                                ClockspeedType::GpuVfCurveClock(pstate) => fl!(I18N, "gpu-pstate-clock", pstate = pstate),
                                ClockspeedType::MemVfCurveClock(pstate) => fl!(I18N, "mem-pstate-clock", pstate = pstate),
                                ClockspeedType::GpuVfCurveVoltage(pstate) => fl!(I18N, "gpu-pstate-clock-voltage", pstate = pstate),
                                ClockspeedType::MemVfCurveVoltage(pstate) => fl!(I18N, "mem-pstate-clock-voltage", pstate = pstate),
                                ClockspeedType::Reset => unreachable!(),
                            }
                        }
                    }
                },

                gtk::Scale {
                    set_adjustment: &self.adjustment,
                    set_orientation: gtk::Orientation::Horizontal,
                    set_hexpand: true,
                    set_digits: 0,
                    set_round_digits: 0,
                    set_value_pos: gtk::PositionType::Right,
                    set_width_request: 150,
                    add_controller = make_event_controller_no_scroll(),
                },

                #[name = "input_button"]
                gtk::SpinButton {
                    set_adjustment: &self.adjustment,
                    add_controller = make_event_controller_no_scroll(),
                },
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
            10.0,
            10.0,
        );

        let change_signal = adjustment.connect_value_changed(move |_| {
            APP_BROKER.send(AppMsg::SettingsChanged);
        });

        Self {
            clock_type: *clock_type,
            custom_title: data.custom_title,
            adjustment,
            change_signal,
            value_ratio: 1.0,
            is_secondary: data.is_secondary,
        }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: relm4::FactorySender<Self>,
    ) {
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
            ClockAdjustmentRowMsg::AddSizeGroup {
                label_group,
                input_group,
            } => {
                label_group.add_widget(&widgets.title_label);
                input_group.add_widget(&widgets.input_button);
            }
            ClockAdjustmentRowMsg::SetVisible(visible) => {
                widgets.root_box.set_visible(visible);
            }
        }

        self.update_view(widgets, sender);
    }
}

impl ClockAdjustmentRow {
    pub fn get_configured_value(&self) -> Option<i32> {
        self.adjustment
            .get_changed_value(false)
            .map(|value| (value / self.value_ratio) as i32)
    }

    pub fn get_raw_value(&self) -> i32 {
        (self.adjustment.value() / self.value_ratio) as i32
    }
}

fn make_event_controller_no_scroll() -> gtk::EventControllerScroll {
    let controller = gtk::EventControllerScroll::new(
        gtk::EventControllerScrollFlags::VERTICAL | gtk::EventControllerScrollFlags::HORIZONTAL,
    );
    controller.connect_scroll(|controller, dx, dy| {
        if let Some(parent) = controller
            .widget()
            .and_then(|widget| widget.ancestor(gtk::ScrolledWindow::static_type()))
        {
            let scrolled_window = parent.downcast::<gtk::ScrolledWindow>().unwrap();

            if dy != 0.0 {
                let current = scrolled_window.vadjustment().value();
                let step = scrolled_window.vadjustment().step_increment();

                // This is a bit of a hack, fractional values are generally touchpad inputs (in pixels),
                // while whole values are scroll wheel events (which should use the `step` value)
                // With newer GTK this should be changed to getting `unit()` from the scroll controller
                let delta = if dy.fract() == 0.0 { dy * step } else { dy };
                scrolled_window.vadjustment().set_value(current + delta);
            }

            if dx != 0.0 {
                let current = scrolled_window.hadjustment().value();
                let step = scrolled_window.hadjustment().step_increment();
                let delta = if dx.fract() == 0.0 { dy * step } else { dy };
                scrolled_window.hadjustment().set_value(current + delta);
            }
        }

        gtk::glib::Propagation::Stop
    });
    controller
}
