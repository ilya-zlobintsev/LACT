use crate::{
    app::{
        msg::AppMsg,
        page_section::PageSection,
        pages::{oc_adjustment::OcAdjustment, PageUpdate},
    },
    APP_BROKER, I18N,
};
use gtk::{
    glib::object::ObjectExt,
    prelude::{AdjustmentExt, BoxExt, ButtonExt, OrientableExt, RangeExt, ScaleExt, WidgetExt},
};
use i18n_embed_fl::fl;
use lact_schema::PowerStats;
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt};
use std::fmt::Write;

#[derive(Default)]
pub struct PowerCapSection {
    power: PowerStats,
    adjustment: OcAdjustment,
    value_text: String,
}

#[derive(Debug)]
pub enum PowerCapMsg {
    Update(PageUpdate),
    RefreshText,
    Reset,
}

#[relm4::component(pub)]
impl relm4::Component for PowerCapSection {
    type Init = ();
    type Input = PowerCapMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        PageSection::new(&fl!(I18N, "power-cap")) {
            append = &gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,

                gtk::Label {
                    #[watch]
                    set_label: &model.value_text,
                },

                gtk::Scale {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_hexpand: true,
                    set_round_digits: 0,
                    set_margin_horizontal: 5,
                    set_draw_value: false,
                    set_adjustment: adjustment,
                },

                gtk::Button {
                    set_label: &fl!(I18N, "reset-button"),
                    connect_clicked => PowerCapMsg::Reset,
                },
            }
        },

        #[local_ref]
        adjustment -> OcAdjustment {
            connect_value_notify => move |_| {
                APP_BROKER.send(AppMsg::SettingsChanged);
            } @ value_notify,
            connect_value_notify => PowerCapMsg::RefreshText,
            connect_upper_notify => PowerCapMsg::RefreshText,
        },
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self::default();
        let adjustment = &model.adjustment;

        let widgets = view_output!();

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
            PowerCapMsg::Update(PageUpdate::Stats(stats)) => {
                // The signal blocking has to be manual,
                // because relm's signal block macro feature doesn't seem to work with non-widget objects
                self.adjustment.block_signal(&widgets.value_notify);
                let power = stats.power;

                self.adjustment.set_upper(power.cap_max.unwrap_or_default());
                self.adjustment.set_lower(power.cap_min.unwrap_or_default());
                self.adjustment
                    .set_initial_value(power.cap_current.unwrap_or_default());

                self.adjustment.unblock_signal(&widgets.value_notify);

                self.power = power;
            }
            PowerCapMsg::Update(PageUpdate::Info(_)) => (),
            PowerCapMsg::RefreshText => {
                self.value_text.clear();
                write!(
                    self.value_text,
                    "{}/{} {}",
                    self.adjustment.value(),
                    self.adjustment.upper(),
                    fl!(I18N, "watt")
                )
                .unwrap();
            }
            PowerCapMsg::Reset => {
                self.adjustment
                    .set_value(self.power.cap_default.unwrap_or_default());
            }
        }

        self.update_view(widgets, sender);
    }
}

impl PowerCapSection {
    pub fn get_user_cap(&self) -> Option<f64> {
        self.adjustment.get_changed_value(true)
    }
}
