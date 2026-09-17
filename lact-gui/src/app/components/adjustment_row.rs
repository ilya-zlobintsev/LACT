use super::adjustment_value::AdjustmentValue;
use crate::I18N;
use crate::app::utils::ext::make_event_controller_no_scroll;
use adw::prelude::*;
use i18n_embed_fl::fl;
use relm4::{FactorySender, RelmWidgetExt, css, factory::FactoryComponent};
use std::marker::PhantomData;

pub struct AdjustmentRow<Key> {
    title: String,
    title_tooltip: String,
    info_text: String,
    unit: Option<String>,
    _key: PhantomData<Key>,
    adjustment: AdjustmentValue,
    value_ratio: f64,
}

pub struct AdjustmentRowInit {
    pub title: String,
    pub title_tooltip: String,
    pub info_text: String,
    pub unit: Option<String>,
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
            title_tooltip: String::new(),
            info_text: String::new(),
            unit: None,
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
    /// Change display units and reset the edit state.
    ValueRatio(f64),
    /// Set a value as an edit, for example when the user presses Reset.
    SetValue(f64),
    SetVisible(bool),
    AddSizeGroup {
        label_group: gtk::SizeGroup,
        input_group: gtk::SizeGroup,
    },
}

#[relm4::factory(pub)]
impl<Key: 'static> FactoryComponent for AdjustmentRow<Key> {
    type ParentWidget = gtk::Box;
    type Index = Key;
    type Init = AdjustmentRowInit;
    type Input = AdjustmentRowMsg;
    // Both slider changes and uncommitted spin-button text edits notify the parent.
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        #[name = "root_box"]
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            add_css_class: "adjustment-row",

            #[name = "title_box"]
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_valign: gtk::Align::Center,
                set_spacing: 2,

                #[name = "label"]
                gtk::Label {
                    set_xalign: 0.0,
                    set_markup: &self.title,
                    set_tooltip_text: (!self.title_tooltip.is_empty()).then_some(self.title_tooltip.as_str()),
                },

                gtk::Box {
                    add_css_class: css::CAPTION,

                    #[name = "subtitle_label"]
                    gtk::Label {
                        set_xalign: 0.0,
                        set_label: &self.subtitle(),
                        add_css_class: css::DIM_LABEL,
                    },

                    gtk::Label {
                        set_label: " · ",
                        set_visible: !self.info_text.is_empty(),
                        add_css_class: css::DIM_LABEL,
                    },

                    #[name = "details_label"]
                    gtk::Label {
                        set_markup: &format!(
                            "<a href=\"details\">{}</a>",
                            gtk::glib::markup_escape_text(&fl!(I18N, "details")),
                        ),
                        set_visible: !self.info_text.is_empty(),
                        connect_activate_link[info_popover] => move |_, _| {
                            info_popover.popup();
                            gtk::glib::Propagation::Stop
                        },
                    },
                },
            },

            #[name = "scale"]
            gtk::Scale {
                set_adjustment: &self.adjustment,
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
                set_adjustment: &self.adjustment,
                set_valign: gtk::Align::Center,
                add_controller = make_event_controller_no_scroll(),
                connect_changed[sender] => move |_| {
                    let _ = sender.output(());
                } @ text_change_signal,
            },
        },

        #[name = "info_popover"]
        gtk::Popover {
            gtk::Label {
                set_label: &self.info_text,
                set_margin_all: 5,
                set_wrap: true,
                set_wrap_mode: gtk::pango::WrapMode::Word,
                set_max_width_chars: 55,
            },
        },

        #[local_ref]
        adjustment -> AdjustmentValue {
            connect_value_changed[sender] => move |_| {
                let _ = sender.output(());
            } @ value_change_signal,
        },
    }

    fn init_model(init: Self::Init, _index: &Self::Index, _sender: FactorySender<Self>) -> Self {
        Self {
            title: init.title,
            title_tooltip: init.title_tooltip,
            info_text: init.info_text,
            unit: init.unit,
            _key: PhantomData,
            adjustment: AdjustmentValue::new(
                init.value,
                init.lower,
                init.upper,
                init.step_increment,
                init.page_increment,
            ),
            value_ratio: 1.0,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &Self::Index,
        root: Self::Root,
        _returned_widget: &gtk::Widget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let adjustment = &self.adjustment;
        let widgets = view_output!();
        widgets.info_popover.set_parent(&widgets.details_label);
        widgets
    }

    fn shutdown(&mut self, widgets: &mut Self::Widgets, _output: relm4::Sender<Self::Output>) {
        widgets.info_popover.unparent();
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        _sender: FactorySender<Self>,
    ) {
        match msg {
            AdjustmentRowMsg::ValueRatio(ratio) => {
                // Changing display units must not emit an edit notification.
                self.adjustment.block_signal(&widgets.value_change_signal);
                widgets.spinbutton.block_signal(&widgets.text_change_signal);

                let raw_current = self.adjustment.value() / self.value_ratio;
                let raw_min = self.adjustment.lower() / self.value_ratio;
                let raw_max = self.adjustment.upper() / self.value_ratio;

                self.adjustment.set_lower(raw_min * ratio);
                self.adjustment.set_upper(raw_max * ratio);
                self.adjustment.set_initial_value(raw_current * ratio);

                self.value_ratio = ratio;
                widgets.subtitle_label.set_label(&self.subtitle());

                widgets
                    .spinbutton
                    .unblock_signal(&widgets.text_change_signal);
                self.adjustment.unblock_signal(&widgets.value_change_signal);
            }
            AdjustmentRowMsg::SetValue(value) => {
                self.adjustment.set_value(value * self.value_ratio);
            }
            AdjustmentRowMsg::SetVisible(visible) => widgets.root_box.set_visible(visible),
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

impl<Key> AdjustmentRow<Key> {
    fn subtitle(&self) -> String {
        let range = format!("{} – {}", self.adjustment.lower(), self.adjustment.upper());
        if let Some(unit) = &self.unit
            && !unit.is_empty()
        {
            format!("{unit} · {range}")
        } else {
            range
        }
    }

    pub fn get_value(&self) -> f64 {
        self.adjustment.value() / self.value_ratio
    }

    pub fn get_changed_value(&self) -> Option<f64> {
        self.adjustment
            .get_changed_value(false)
            .map(|value| value / self.value_ratio)
    }
}
