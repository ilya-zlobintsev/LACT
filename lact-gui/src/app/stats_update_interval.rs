use crate::{
    CONFIG, I18N,
    config::{MAX_STATS_POLL_INTERVAL_MS, MIN_STATS_POLL_INTERVAL_MS},
};
use gtk::prelude::{BoxExt, EditableExt, OrientableExt, WidgetExt};
use i18n_embed_fl::fl;
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent, css};

pub struct StatsUpdateInterval;

#[relm4::component(pub)]
impl SimpleComponent for StatsUpdateInterval {
    type Init = ();
    type Input = ();
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 4,
            add_css_class: "sidebar-section",

            gtk::Label {
                set_label: &fl!(I18N, "stats-update-interval"),
                set_halign: gtk::Align::Start,
                set_margin_horizontal: 4,
                add_css_class: css::DIM_LABEL,
                add_css_class: css::CAPTION,
            },

            gtk::SpinButton {
                set_range: (MIN_STATS_POLL_INTERVAL_MS as f64, MAX_STATS_POLL_INTERVAL_MS as f64),
                set_increments: (250.0, 500.0),
                set_digits: 0,
                set_alignment: 0.5,
                set_value: CONFIG.read().stats_poll_interval_ms as f64,
                connect_value_changed => move |btn| {
                    CONFIG.write().edit(|config| {
                        config.stats_poll_interval_ms = btn.value() as i64;
                    })
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self;
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }
}
