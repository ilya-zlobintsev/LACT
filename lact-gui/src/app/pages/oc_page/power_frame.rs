use crate::{
    APP_BROKER, I18N,
    app::{
        components::{
            adjustment_card::AdjustmentCard,
            adjustment_row::{AdjustmentRow, AdjustmentRowInit, AdjustmentRowMsg},
            page_section::PageSection,
        },
        msg::AppMsg,
        pages::oc_page::{
            OcPageMsg,
            performance_frame::{PerformanceFrame, PerformanceFrameMsg},
        },
        utils::ext::{RelmDefaultLauchable, RelmLaunchable},
    },
};
use adw::prelude::*;
use amdgpu_sysfs::gpu_handle::PerformanceLevel;
use i18n_embed_fl::fl;
use lact_schema::PowerStats;
use nvml_wrapper::enums::device::PowerMizerMode;
use relm4::{ComponentController, ComponentParts, ComponentSender};

pub struct PowerFrame {
    power: PowerStats,
    power_row: relm4::Controller<AdjustmentRow>,
    cap_available: bool,
    performance_frame: relm4::Controller<PerformanceFrame>,
}

#[derive(Debug)]
pub enum PowerFrameMsg {
    PowerStats(PowerStats),
    Performance(PerformanceFrameMsg),
    RefreshVisibility,
    Reset,
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for PowerFrame {
    type Init = ();
    type Input = PowerFrameMsg;
    type Output = OcPageMsg;

    view! {
        #[root]
        PageSection::new(&fl!(I18N, "power-section")) {
            #[watch]
            set_visible: model.is_available(),

            append_header = &gtk::Button {
                set_label: &fl!(I18N, "default-button"),
                connect_clicked => PowerFrameMsg::Reset,

                set_halign: gtk::Align::End,
                set_hexpand: true,
                #[watch]
                set_visible: model.cap_available,
            },
            #[template]
            append_child = &AdjustmentCard {
                #[template_child]
                content {
                    append = &model.power_row.widget().clone() -> gtk::Box {
                        #[watch]
                        set_visible: model.cap_available,
                    },

                    append: model.performance_frame.widget(),
                },
            },
        },
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            power: PowerStats::default(),
            power_row: AdjustmentRow::launch(AdjustmentRowInit {
                title: format!("{} ({})", fl!(I18N, "power-cap"), fl!(I18N, "watt")),
                ..Default::default()
            })
            .connect_receiver(|_, ()| APP_BROKER.send(AppMsg::SettingsChanged)),
            cap_available: false,
            performance_frame: PerformanceFrame::launch_default()
                .forward(sender.output_sender(), |msg| msg),
        };
        let visibility_sender = sender.clone();
        model
            .performance_frame
            .widget()
            .connect_visible_notify(move |_| {
                visibility_sender.input(PowerFrameMsg::RefreshVisibility);
            });

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            PowerFrameMsg::PowerStats(power) => {
                self.power_row.emit(AdjustmentRowMsg::Configure {
                    value: power.cap_current.unwrap_or_default(),
                    lower: power.cap_min.unwrap_or_default(),
                    upper: power.cap_max.unwrap_or_default(),
                });

                self.power = power;
                self.cap_available = self.power.cap_current.is_some();
            }
            PowerFrameMsg::Performance(msg) => {
                self.performance_frame.emit(msg);
            }
            PowerFrameMsg::RefreshVisibility => (),
            PowerFrameMsg::Reset => {
                self.power_row.emit(AdjustmentRowMsg::SetValue(
                    self.power.cap_default.unwrap_or_default(),
                ));
            }
        }
    }
}

impl PowerFrame {
    fn is_available(&self) -> bool {
        self.cap_available || self.performance_frame.widget().get_visible()
    }

    pub fn get_user_cap(&self) -> Option<f64> {
        self.power_row
            .model()
            .get_changed_value()
            .filter(|value| *value != 0.0)
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
