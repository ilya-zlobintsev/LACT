use crate::{
    APP_BROKER, I18N,
    app::{
        components::{oc_adjustment::OcAdjustment, page_section::PageSection},
        msg::AppMsg,
        pages::oc_page::{
            OcPageMsg,
            performance_frame::{PerformanceFrame, PerformanceFrameMsg},
        },
        utils::ext::{RelmDefaultLauchable, make_event_controller_no_scroll},
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
    adjustment: OcAdjustment,
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
                set_visible: model.cap_available,
            },
            // todo: refactor to adjustment-row
            append_child = &gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 10,
                #[watch]
                set_visible: model.cap_available,

                gtk::Label {
                    set_label: &format!("{} ({})", fl!(I18N, "power-cap"), fl!(I18N, "watt")),
                    set_xalign: 0.0,
                },

                gtk::Scale {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_hexpand: true,
                    set_digits: 0,
                    set_round_digits: 0,
                    set_value_pos: gtk::PositionType::Right,
                    set_width_request: 100,
                    set_adjustment: adjustment,
                    add_controller = make_event_controller_no_scroll(),
                },

                #[name = "input_button"]
                gtk::SpinButton {
                    set_adjustment: adjustment,
                    add_controller = make_event_controller_no_scroll(),
                    connect_changed => move |_| {
                        APP_BROKER.send(AppMsg::SettingsChanged);
                    } @ text_change_signal,
                },
            },

            append_child = model.performance_frame.widget(),
        },

        #[local_ref]
        adjustment -> OcAdjustment {
            connect_value_notify => move |_| {
                APP_BROKER.send(AppMsg::SettingsChanged);
            } @ value_notify,
        },
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            power: PowerStats::default(),
            adjustment: OcAdjustment::new(0.0, 0.0, 0.0, 1.0, 10.0),
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
            PowerFrameMsg::PowerStats(power) => {
                // The signal blocking has to be manual,
                // because relm's signal block macro feature doesn't seem to work with non-widget objects
                self.adjustment.block_signal(&widgets.value_notify);
                widgets
                    .input_button
                    .block_signal(&widgets.text_change_signal);

                self.adjustment.set_upper(power.cap_max.unwrap_or_default());
                self.adjustment.set_lower(power.cap_min.unwrap_or_default());
                self.adjustment
                    .set_initial_value(power.cap_current.unwrap_or_default());

                widgets
                    .input_button
                    .unblock_signal(&widgets.text_change_signal);
                self.adjustment.unblock_signal(&widgets.value_notify);

                self.power = power;
                self.cap_available = self.power.cap_current.is_some();
            }
            PowerFrameMsg::Performance(msg) => {
                self.performance_frame.emit(msg);
            }
            PowerFrameMsg::RefreshVisibility => (),
            PowerFrameMsg::Reset => {
                self.adjustment
                    .set_value(self.power.cap_default.unwrap_or_default());
            }
        }

        self.update_view(widgets, sender);
    }
}

impl PowerFrame {
    fn is_available(&self) -> bool {
        self.cap_available || self.performance_frame.widget().get_visible()
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
