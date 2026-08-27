use crate::{
    APP_BROKER, I18N,
    app::{
        components::{oc_adjustment::OcAdjustment, page_section::PageSection},
        msg::AppMsg,
        pages::oc_page::{
            OcPageMsg,
            performance_frame::{PerformanceFrame, PerformanceFrameMsg},
        },
        utils::ext::RelmDefaultLauchable,
    },
};
use adw::prelude::*;
use amdgpu_sysfs::gpu_handle::PerformanceLevel;
use i18n_embed_fl::fl;
use lact_schema::PowerStats;
use nvml_wrapper::enums::device::PowerMizerMode;
use relm4::{ComponentController, ComponentParts, ComponentSender, RelmWidgetExt};
use std::fmt::Write;

pub struct PowerSection {
    power: PowerStats,
    adjustment: OcAdjustment,
    value_text: String,
    cap_available: bool,
    performance_level_available: bool,
    power_profile_available: bool,
    power_mizer_available: bool,
    performance_frame: relm4::Controller<PerformanceFrame>,
}

#[derive(Debug)]
pub enum PowerSectionMsg {
    PowerStats(PowerStats),
    Performance(PerformanceFrameMsg),
    RefreshText,
    Reset,
}

#[relm4::component(pub)]
impl relm4::Component for PowerSection {
    type Init = ();
    type Input = PowerSectionMsg;
    type Output = OcPageMsg;
    type CommandOutput = ();

    view! {
        #[root]
        PageSection::new(&fl!(I18N, "power-section")) {
            #[watch]
            set_visible: model.is_available(),

            append_header = &gtk::Button {
                set_label: &fl!(I18N, "default-button"),
                connect_clicked => PowerSectionMsg::Reset,

                set_halign: gtk::Align::End,
                set_hexpand: true,
                #[watch]
                set_visible: model.cap_available,
            },

            append_child = &gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 10,
                #[watch]
                set_visible: model.cap_available,

                gtk::Label {
                    set_label: &fl!(I18N, "power-cap"),
                },

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
            },

            append_child = model.performance_frame.widget(),
        },

        #[local_ref]
        adjustment -> OcAdjustment {
            connect_value_notify => move |_| {
                APP_BROKER.send(AppMsg::SettingsChanged);
            } @ value_notify,
            connect_value_notify => PowerSectionMsg::RefreshText,
            connect_upper_notify => PowerSectionMsg::RefreshText,
        },
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            power: PowerStats::default(),
            adjustment: OcAdjustment::default(),
            value_text: String::new(),
            cap_available: false,
            performance_level_available: false,
            power_profile_available: false,
            power_mizer_available: false,
            performance_frame: PerformanceFrame::launch_default()
                .forward(sender.output_sender(), |msg| msg),
        };
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
            PowerSectionMsg::PowerStats(power) => {
                // The signal blocking has to be manual,
                // because relm's signal block macro feature doesn't seem to work with non-widget objects
                self.adjustment.block_signal(&widgets.value_notify);

                self.adjustment.set_upper(power.cap_max.unwrap_or_default());
                self.adjustment.set_lower(power.cap_min.unwrap_or_default());
                self.adjustment
                    .set_initial_value(power.cap_current.unwrap_or_default());

                self.adjustment.unblock_signal(&widgets.value_notify);

                self.power = power;
                self.cap_available = self.power.cap_current.is_some();
            }
            PowerSectionMsg::Performance(msg) => {
                match &msg {
                    PerformanceFrameMsg::PerformanceLevel(level) => {
                        self.performance_level_available = level.is_some();
                    }
                    PerformanceFrameMsg::PowerProfileModes(table) => {
                        self.power_profile_available = table.is_some();
                    }
                    PerformanceFrameMsg::PowerMizerInfo { active, .. } => {
                        self.power_mizer_available = active.is_some();
                    }
                    PerformanceFrameMsg::PowerProfileSelected(_)
                    | PerformanceFrameMsg::PowerMizerSelected(_) => {}
                }
                self.performance_frame.emit(msg);
            }
            PowerSectionMsg::RefreshText => {
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
            PowerSectionMsg::Reset => {
                self.adjustment
                    .set_value(self.power.cap_default.unwrap_or_default());
            }
        }

        self.update_view(widgets, sender);
    }
}

impl PowerSection {
    fn is_available(&self) -> bool {
        self.cap_available
            || self.performance_level_available
            || self.power_profile_available
            || self.power_mizer_available
    }

    pub fn get_user_cap(&self) -> Option<f64> {
        self.adjustment.get_changed_value(true)
    }

    pub fn performance_level(&self) -> Option<PerformanceLevel> {
        self.performance_frame.model().performance_level()
    }

    pub fn active_power_mizer_mode(&self) -> Option<PowerMizerMode> {
        self.performance_frame.model().active_power_mizer_mode()
    }

    pub fn power_profile_mode(&self) -> Option<u16> {
        self.performance_frame.model().power_profile_mode()
    }

    pub fn power_profile_mode_custom_heuristics(&self) -> Vec<Vec<Option<i32>>> {
        self.performance_frame
            .model()
            .power_profile_mode_custom_heuristics()
    }
}
