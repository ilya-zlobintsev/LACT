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
use lact_schema::{PowerStats, config::NvidiaPowerCapMode};
use nvml_wrapper::enums::device::PowerMizerMode;
use relm4::{ComponentController, ComponentParts, ComponentSender, css, factory::FactoryHashMap};

pub struct PowerFrame {
    power: PowerStats,
    power_row: FactoryHashMap<(), AdjustmentRow<()>>,
    performance_frame: relm4::Controller<PerformanceFrame>,
    is_nvidia: bool,
    nvidia_mode: NvidiaPowerCapMode,
}

#[derive(Debug)]
pub enum PowerFrameMsg {
    PowerStats(PowerStats),
    Driver(String),
    NvidiaMode(NvidiaPowerCapMode),
    ToggleNvidiaIoctl(bool),
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

                    #[name = "experimental_controls"]
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 6,
                        #[watch]
                        set_visible: model.nvidia_mode_available(),

                        #[name = "ioctl_toggle"]
                        gtk::CheckButton {
                            set_label: Some(&fl!(I18N, "nvidia-power-cap-ioctl")),
                            #[watch]
                            #[block_signal(ioctl_toggled_handler)]
                            set_active: model.nvidia_mode == NvidiaPowerCapMode::Ioctl,
                            connect_toggled[sender] => move |button| {
                                sender.input(PowerFrameMsg::ToggleNvidiaIoctl(button.is_active()));
                            } @ ioctl_toggled_handler,
                        },
                        gtk::Label {
                            set_label: &fl!(I18N, "nvidia-power-cap-ioctl-warning"),
                            set_wrap: true,
                            set_max_width_chars: 55,
                            set_xalign: 0.0,
                            add_css_class: css::WARNING,
                        },
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
            is_nvidia: false,
            nvidia_mode: NvidiaPowerCapMode::Nvml,
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
                self.power = power;

                if let Some(value) = self.power.cap_current {
                    self.power_row.insert(
                        (),
                        AdjustmentRowInit {
                            title: format!("{} ({})", fl!(I18N, "power-cap"), fl!(I18N, "watt")),
                            value,
                            lower: self.cap_min(),
                            upper: self.power.cap_max.unwrap_or_default(),
                            ..Default::default()
                        },
                    );
                }
            }
            PowerFrameMsg::Driver(driver) => {
                // DeviceInfo includes the driver version, e.g. "nvidia 610.57.04".
                self.is_nvidia = driver.split_ascii_whitespace().next() == Some("nvidia");
            }
            // Config arrives before the initial stats rebuild; it is not a user edit.
            PowerFrameMsg::NvidiaMode(mode) => self.nvidia_mode = mode,
            PowerFrameMsg::ToggleNvidiaIoctl(enabled) => {
                let mode = if enabled {
                    NvidiaPowerCapMode::Ioctl
                } else {
                    NvidiaPowerCapMode::Nvml
                };
                if self.nvidia_mode_available() && mode != self.nvidia_mode {
                    self.nvidia_mode = mode;
                    self.power_row.send(
                        &(),
                        AdjustmentRowMsg::SetBounds {
                            lower: self.cap_min(),
                            upper: self.power.cap_max.unwrap_or_default(),
                        },
                    );
                    APP_BROKER.send(AppMsg::SettingsChanged);
                }
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

#[cfg(all(test, feature = "gtk-tests"))]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires a GTK display; run explicitly with --ignored"]
    fn experimental_power_mode_is_explicit_and_restores_native_bounds() {
        adw::init().unwrap();
        let context = gtk::glib::MainContext::default();
        let _guard = context.acquire().unwrap();
        let drain = || {
            while context.pending() {
                context.iteration(false);
            }
        };
        let frame = PowerFrame::detach_default();
        let native_stats = PowerStats {
            cap_current: Some(300.0),
            cap_min: Some(250.0),
            cap_min_native: Some(250.0),
            cap_max: Some(325.0),
            cap_default: Some(300.0),
            ..Default::default()
        };
        frame.emit(PowerFrameMsg::Driver("nvidia 610.57.04".to_owned()));
        frame.emit(PowerFrameMsg::NvidiaMode(NvidiaPowerCapMode::Nvml));
        frame.emit(PowerFrameMsg::PowerStats(native_stats.clone()));
        drain();
        assert!(frame.widgets().experimental_controls.is_visible());
        assert!(!frame.widgets().ioctl_toggle.is_active());
        assert_eq!(frame.model().cap_min(), 250.0);
        assert_eq!(frame.model().get_user_cap(), None);

        // A user opt-in extends the range; opt-out clamps the pending edit and
        // exposes it to AppModel so an old sub-minimum config is not resubmitted.
        frame.widgets().ioctl_toggle.set_active(true);
        drain();
        assert_eq!(frame.model().cap_min(), 30.0);
        frame
            .model()
            .power_row
            .send(&(), AdjustmentRowMsg::SetValue(150.0));
        drain();
        assert_eq!(frame.model().get_user_cap(), Some(150.0));
        frame.widgets().ioctl_toggle.set_active(false);
        drain();
        assert_eq!(frame.model().cap_min(), 250.0);
        assert_eq!(frame.model().get_user_cap(), Some(250.0));

        // Restore an opted-in profile and its lower cap together, without
        // treating programmatic checkbox updates as user mode changes.
        frame.emit(PowerFrameMsg::NvidiaMode(NvidiaPowerCapMode::Ioctl));
        frame.emit(PowerFrameMsg::PowerStats(PowerStats {
            cap_current: Some(150.0),
            cap_min: Some(30.0),
            ..native_stats.clone()
        }));
        drain();
        assert!(frame.widgets().ioctl_toggle.is_active());
        assert_eq!(frame.model().power_row.get(&()).unwrap().get_value(), 150.0);
        assert_eq!(frame.model().get_user_cap(), None);
        frame.emit(PowerFrameMsg::Reset);
        drain();
        assert_eq!(frame.model().get_user_cap(), Some(300.0));
        assert_eq!(
            frame.model().nvidia_power_cap_mode(),
            Some(NvidiaPowerCapMode::Ioctl)
        );

        frame.emit(PowerFrameMsg::NvidiaMode(NvidiaPowerCapMode::Nvml));
        frame.emit(PowerFrameMsg::PowerStats(native_stats));
        drain();
        assert!(!frame.widgets().ioctl_toggle.is_active());
        assert_eq!(frame.model().get_user_cap(), None);
        frame.emit(PowerFrameMsg::Driver("amdgpu".to_owned()));
        drain();
        assert_eq!(frame.model().nvidia_power_cap_mode(), None);
        assert!(!frame.widgets().experimental_controls.is_visible());

        frame.emit(PowerFrameMsg::Driver("nvidia_drm".to_owned()));
        drain();
        assert!(!frame.widgets().experimental_controls.is_visible());

        frame.emit(PowerFrameMsg::Driver("nvidia".to_owned()));
        drain();
        assert!(frame.widgets().experimental_controls.is_visible());
    }
}

impl PowerFrame {
    fn nvidia_mode_available(&self) -> bool {
        self.is_nvidia && self.power.cap_min_native.is_some() && !self.power_row.is_empty()
    }

    fn cap_min(&self) -> f64 {
        let native = self
            .power
            .cap_min_native
            .or(self.power.cap_min)
            .unwrap_or_default();
        if self.is_nvidia && self.nvidia_mode == NvidiaPowerCapMode::Ioctl {
            native.min(30.0)
        } else {
            native
        }
    }

    pub fn nvidia_power_cap_mode(&self) -> Option<NvidiaPowerCapMode> {
        self.nvidia_mode_available().then_some(self.nvidia_mode)
    }

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
