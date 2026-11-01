use super::ControlInit;
use crate::app::utils::{ext::make_event_controller_no_scroll, formatting::fmt_value_with_unit};
use adw::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, css};

pub struct AdjustmentInit {
    pub unit: String,
    pub value: f64,
    pub default_value: Option<f64>,
    pub lower: f64,
    pub upper: f64,
    pub step_increment: f64,
    pub page_increment: f64,
    pub lower_label_group: Option<gtk::SizeGroup>,
    pub upper_label_group: Option<gtk::SizeGroup>,
}

impl Default for AdjustmentInit {
    fn default() -> Self {
        Self {
            unit: String::new(),
            value: 0.0,
            default_value: None,
            lower: 0.0,
            upper: 0.0,
            step_increment: 1.0,
            page_increment: 10.0,
            lower_label_group: None,
            upper_label_group: None,
        }
    }
}

pub struct AdjustmentControl {
    adjustment: gtk::Adjustment,
    default_value: Option<f64>,
    value_ratio: f64,
    title: gtk::Box,
    unit: String,
    pub(super) size_group_widgets: Vec<(gtk::SizeGroup, gtk::Widget)>,
}

#[derive(Debug)]
pub enum AdjustmentControlMsg {
    Changed,
    #[expect(dead_code, reason = "setting-row-migration")]
    SetValueRatio(f64),
}

#[relm4::component(pub)]
impl Component for AdjustmentControl {
    type Init = ControlInit<AdjustmentInit>;
    type Input = AdjustmentControlMsg;
    type Output = f64;
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            gtk::Box {
                set_spacing: 12,
                append: &model.title,

                #[name = "spinbutton"]
                gtk::SpinButton {
                    set_adjustment: &model.adjustment,
                    set_valign: gtk::Align::Center,
                    add_controller = make_event_controller_no_scroll(),
                },
            },

            gtk::Box {
                set_spacing: 12,

                #[name = "lower_label"]
                gtk::Label {
                    set_label: &fmt_value_with_unit(model.adjustment.lower(), &model.unit),
                    add_css_class: css::CAPTION,
                    add_css_class: css::DIM_LABEL,
                },
                #[name = "scale"]
                gtk::Scale {
                    set_adjustment: &model.adjustment,
                    set_orientation: gtk::Orientation::Horizontal,
                    set_hexpand: true,
                    set_digits: 0,
                    set_round_digits: 0,
                    set_value_pos: gtk::PositionType::Right,
                    set_width_request: 100,
                    add_controller = make_event_controller_no_scroll(),
                },
                #[name = "upper_label"]
                gtk::Label {
                    set_label: &fmt_value_with_unit(model.adjustment.upper(), &model.unit),
                    add_css_class: css::CAPTION,
                    add_css_class: css::DIM_LABEL,
                },
            },
        },

        #[local_ref]
        adjustment -> gtk::Adjustment {
            connect_value_changed => AdjustmentControlMsg::Changed @ value_change_signal,
        },
    }

    fn init(
        ControlInit {
            init,
            title,
            label_group,
            input_group,
        }: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = Self {
            adjustment: gtk::Adjustment::new(
                init.value,
                init.lower,
                init.upper,
                init.step_increment,
                init.page_increment,
                0.0,
            ),
            default_value: init.default_value,
            value_ratio: 1.0,
            title,
            unit: init.unit,
            size_group_widgets: Vec::new(),
        };
        let adjustment = &model.adjustment;
        let widgets = view_output!();
        for (group, widget) in [
            (label_group, model.title.clone().upcast::<gtk::Widget>()),
            (input_group, widgets.spinbutton.clone().upcast()),
            (init.lower_label_group, widgets.lower_label.clone().upcast()),
            (init.upper_label_group, widgets.upper_label.clone().upcast()),
        ] {
            if let Some(group) = group {
                group.add_widget(&widget);
                model.size_group_widgets.push((group, widget));
            }
        }
        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            AdjustmentControlMsg::Changed => {
                let _ = sender.output(self.get_value());
            }
            AdjustmentControlMsg::SetValueRatio(ratio) => {
                self.adjustment.block_signal(&widgets.value_change_signal);
                self.set_value_ratio(ratio);
                widgets
                    .lower_label
                    .set_label(&fmt_value_with_unit(self.adjustment.lower(), &self.unit));
                widgets
                    .upper_label
                    .set_label(&fmt_value_with_unit(self.adjustment.upper(), &self.unit));
                self.adjustment.unblock_signal(&widgets.value_change_signal);
            }
        }
    }
}

impl AdjustmentControl {
    pub(super) fn get_value(&self) -> f64 {
        self.adjustment.value() / self.value_ratio
    }

    pub(super) fn set_value(&self, value: f64) {
        self.adjustment.set_value(value * self.value_ratio);
    }

    pub(super) fn reset(&self) {
        if let Some(value) = self.default_value {
            self.set_value(value);
        }
    }

    pub(super) fn serialize(&self) -> Option<f64> {
        let value = self.get_value();
        (Some(value) != self.default_value).then_some(value)
    }

    /// Changes display units, retaining the raw value.
    fn set_value_ratio(&mut self, ratio: f64) {
        let value = self.get_value();
        let lower = self.adjustment.lower() / self.value_ratio;
        let upper = self.adjustment.upper() / self.value_ratio;
        self.value_ratio = ratio;
        self.adjustment.set_lower(lower * ratio);
        self.adjustment.set_upper(upper * ratio);
        self.set_value(value);
    }
}
