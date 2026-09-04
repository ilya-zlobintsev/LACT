use super::adjustment_value::AdjustmentValue;
use crate::app::utils::ext::make_event_controller_no_scroll;
use gtk::prelude::*;
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, css};

pub struct AdjustmentRow {
    adjustment: AdjustmentValue,
    value_ratio: f64,
}

pub struct AdjustmentRowInit {
    pub title: String,
    pub info_text: String,
    pub value: f64,
    pub lower: f64,
    pub upper: f64,
    pub step_increment: f64,
    pub page_increment: f64,
}

impl Default for AdjustmentRowInit {
    fn default() -> Self {
        Self {
            title: String::new(),
            info_text: String::new(),
            value: 0.0,
            lower: 0.0,
            upper: 0.0,
            step_increment: 1.0,
            page_increment: 10.0,
        }
    }
}

#[derive(Debug)]
pub enum AdjustmentRowMsg {
    /// Change display units while preserving the configured value and edit state.
    ValueRatio(f64),
    /// Set a value as an edit, for example when the user presses Reset.
    SetValue(f64),
    SetVisible(bool),
    AddSizeGroup {
        label_group: gtk::SizeGroup,
        input_group: gtk::SizeGroup,
    },
}

#[relm4::component(pub)]
impl relm4::Component for AdjustmentRow {
    type Init = AdjustmentRowInit;
    type Input = AdjustmentRowMsg;
    // Both slider changes and uncommitted spin-button text edits notify the parent.
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 5,

            #[name = "title_box"]
            gtk::Box {
                set_spacing: 5,

                #[name = "label"]
                gtk::Label {
                    set_xalign: 0.0,
                    set_label: &init.title,
                },

                #[name = "info_button"]
                gtk::MenuButton {
                    set_icon_name: "dialog-information-symbolic",
                    set_always_show_arrow: false,
                    add_css_class: css::FLAT,
                    set_visible: !init.info_text.is_empty(),

                    #[wrap(Some)]
                    set_popover = &gtk::Popover {
                        #[name = "info_label"]
                        gtk::Label {
                            set_label: &init.info_text,
                            set_margin_all: 5,
                            set_wrap: true,
                            set_wrap_mode: gtk::pango::WrapMode::Word,
                            set_max_width_chars: 55,
                        },
                    },
                },
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

            #[name = "spinbutton"]
            gtk::SpinButton {
                set_adjustment: &model.adjustment,
                add_controller = make_event_controller_no_scroll(),
                connect_changed[sender] => move |_| {
                    let _ = sender.output(());
                } @ text_change_signal,
            },
        },

        #[local_ref]
        adjustment -> AdjustmentValue {
            connect_value_changed[sender] => move |_| {
                let _ = sender.output(());
            } @ value_change_signal,
        },
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            adjustment: AdjustmentValue::new(
                init.value,
                init.lower,
                init.upper,
                init.step_increment,
                init.page_increment,
            ),
            value_ratio: 1.0,
        };
        let adjustment = &model.adjustment;
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        _sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match msg {
            AdjustmentRowMsg::ValueRatio(ratio) => {
                // Changing display units must not emit an edit notification.
                self.adjustment.block_signal(&widgets.value_change_signal);
                widgets.spinbutton.block_signal(&widgets.text_change_signal);

                let changed = self.get_changed_value().is_some();
                let value = self.get_value();
                let lower = self.adjustment.lower() / self.value_ratio;
                let upper = self.adjustment.upper() / self.value_ratio;
                self.value_ratio = ratio;
                self.adjustment.configure(
                    value * ratio,
                    lower * ratio,
                    upper * ratio,
                    self.adjustment.step_increment(),
                    self.adjustment.page_increment(),
                    0.0,
                );
                if !changed {
                    self.adjustment.set_initial_value(value * ratio);
                }

                widgets
                    .spinbutton
                    .unblock_signal(&widgets.text_change_signal);
                self.adjustment.unblock_signal(&widgets.value_change_signal);
            }
            AdjustmentRowMsg::SetValue(value) => {
                self.adjustment.set_value(value * self.value_ratio);
            }
            AdjustmentRowMsg::SetVisible(visible) => root.set_visible(visible),
            AdjustmentRowMsg::AddSizeGroup {
                label_group,
                input_group,
            } => {
                label_group.add_widget(&widgets.title_box);
                input_group.add_widget(&widgets.spinbutton);
            }
        }
    }
}

impl AdjustmentRow {
    pub fn get_value(&self) -> f64 {
        self.adjustment.value() / self.value_ratio
    }

    pub fn get_changed_value(&self) -> Option<f64> {
        self.adjustment
            .get_changed_value(false)
            .map(|value| value / self.value_ratio)
    }
}
