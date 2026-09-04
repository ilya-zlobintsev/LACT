use super::adjustment_value::AdjustmentValue;
use crate::app::utils::ext::make_event_controller_no_scroll;
use gtk::prelude::*;
use relm4::{RelmWidgetExt, css};

#[relm4::widget_template(pub)]
impl relm4::WidgetTemplate for AdjustmentRow {
    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 5,

            #[name = "title_box"]
            gtk::Box {
                set_spacing: 5,

                #[name = "label"]
                gtk::Label {
                    set_xalign: 0.0,
                },

                #[name = "info_button"]
                gtk::MenuButton {
                    set_icon_name: "dialog-information-symbolic",
                    set_always_show_arrow: false,
                    add_css_class: css::FLAT,
                    set_visible: false,

                    #[wrap(Some)]
                    set_popover = &gtk::Popover {
                        #[name = "info_label"]
                        gtk::Label {
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
                add_controller = make_event_controller_no_scroll(),
            },
        }
    }
}

impl AdjustmentRow {
    pub fn set_adjustment(&self, adjustment: &AdjustmentValue) {
        self.scale.set_adjustment(adjustment);
        self.spinbutton.set_adjustment(adjustment);
    }

    pub fn set_info_text(&self, text: &str) {
        self.info_label.set_label(text);
        self.info_button.set_visible(!text.is_empty());
    }
}

impl AsRef<gtk::Widget> for AdjustmentRow {
    fn as_ref(&self) -> &gtk::Widget {
        self.upcast_ref()
    }
}
