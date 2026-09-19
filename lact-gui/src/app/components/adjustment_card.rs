use crate::I18N;
use gtk::prelude::*;
use i18n_embed_fl::fl;
use relm4::css;

#[relm4::widget_template(pub)]
impl relm4::WidgetTemplate for AdjustmentCard {
    view! {
        #[name = "content"]
        gtk::ListBox {
            set_selection_mode: gtk::SelectionMode::None,
            add_css_class: css::BOXED_LIST,
            add_css_class: "adjustment-list",
            set_hexpand: true,
            set_valign: gtk::Align::Start,

            #[name = "advanced_features"]
            gtk::ListBoxRow {
                set_activatable: false,
                set_selectable: false,
                set_visible: false,
                add_css_class: "adjustment-card-advanced-features",

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 5,

                    gtk::Label {
                        set_label: &fl!(I18N, "advanced-features"),
                        set_halign: gtk::Align::Start,
                        add_css_class: css::DIM_LABEL,
                        add_css_class: css::CAPTION,
                    },

                    #[name = "controls"]
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 10,
                        set_halign: gtk::Align::Start,
                    },
                },
            },
        }
    }
}
