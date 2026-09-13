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
        utils::ext::RelmDefaultLauchable,
    },
};
use adw::prelude::*;
use amdgpu_sysfs::gpu_handle::PerformanceLevel;
use i18n_embed_fl::fl;
use lact_schema::PowerStats;
use nvml_wrapper::enums::device::PowerMizerMode;
use relm4::{ComponentController, ComponentParts, ComponentSender, factory::FactoryHashMap};

pub struct PowerFrame {
    power: PowerStats,
    power_row: FactoryHashMap<(), AdjustmentRow<()>>,
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
impl relm4::Component for PowerFrame {
    type Init = ();
    type Input = PowerFrameMsg;
    type Output = OcPageMsg;
    type CommandOutput = ();

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
                set_visible: !model.power_row.is_empty(),
            },
            #[template]
            append_child = &AdjustmentCard {
                #[template_child]
                content {
                    #[local_ref]
                    power_row_widget -> gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        #[watch]
                        set_visible: !model.power_row.is_empty(),
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
            power_row: FactoryHashMap::builder()
                .launch_default()
                .forward(APP_BROKER.sender(), |()| AppMsg::SettingsChanged),
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

        let power_row_widget = model.power_row.widget();
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
            PowerFrameMsg::PowerStats(power) => {
                self.power_row.clear();

                if let Some(value) = power.cap_current {
                    self.power_row.insert(
                        (),
                        AdjustmentRowInit {
                            title: format!("{} ({})", fl!(I18N, "power-cap"), fl!(I18N, "watt")),
                            value,
                            lower: power.cap_min.unwrap_or_default(),
                            upper: power.cap_max.unwrap_or_default(),
                            ..Default::default()
                        },
                    );
                }

                self.power = power;
            }
            PowerFrameMsg::Performance(msg) => {
                self.performance_frame.emit(msg);
            }
            PowerFrameMsg::RefreshVisibility => (),
            PowerFrameMsg::Reset => {
                if !self.power_row.is_empty() {
                    self.power_row.send(
                        &(),
                        AdjustmentRowMsg::SetValue(self.power.cap_default.unwrap_or_default()),
                    );
                }
            }
        }

        self.update_view(widgets, sender);
    }
}

impl PowerFrame {
    fn is_available(&self) -> bool {
        !self.power_row.is_empty() || self.performance_frame.widget().get_visible()
    }

    pub fn get_user_cap(&self) -> Option<f64> {
        self.power_row
            .get(&())?
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
