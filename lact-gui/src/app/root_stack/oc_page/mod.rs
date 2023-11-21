mod clocks_frame;
mod gpu_stats_section;
mod oc_adjustment;
mod performance_frame;
mod power_cap_section;
// mod power_cap_frame;
mod power_states;

use self::power_cap_section::PowerCapSection;
use self::power_states::power_states_frame::PowerStatesFrame;
use clocks_frame::ClocksFrame;
use gpu_stats_section::GpuStatsSection;
use gtk::*;
use gtk::{glib::clone, prelude::*};
use lact_client::schema::amdgpu_sysfs::gpu_handle::PowerLevelKind;
use lact_client::schema::{
    amdgpu_sysfs::gpu_handle::{overdrive::ClocksTableGen, PerformanceLevel},
    DeviceStats, SystemInfo,
};
use performance_frame::PerformanceFrame;
// use power_cap_frame::PowerCapFrame;
use std::collections::HashMap;
use tracing::warn;

const OVERCLOCKING_DISABLED_TEXT: &str = "Overclocking support is not enabled! \
You can still change basic settings, but the more advanced clocks and voltage control will not be available.";

#[derive(Clone)]
pub struct OcPage {
    pub container: ScrolledWindow,
    stats_section: GpuStatsSection,
    pub performance_frame: PerformanceFrame,
    // power_cap_frame: PowerCapFrame,
    power_cap_section: PowerCapSection,
    pub power_states_frame: PowerStatesFrame,
    pub clocks_frame: ClocksFrame,
    pub enable_overclocking_button: Option<Button>,
}

impl OcPage {
    pub fn new(system_info: &SystemInfo) -> Self {
        let container = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .build();

        let vbox = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(15)
            .margin_start(20)
            .margin_end(20)
            .build();

        let mut enable_overclocking_button = None;

        if system_info.amdgpu_overdrive_enabled == Some(false) {
            let (warning_frame, button) = oc_warning_frame();
            enable_overclocking_button = Some(button);
            vbox.append(&warning_frame);
        }

        let stats_section = GpuStatsSection::new();
        vbox.append(&stats_section);

        let power_cap_section = PowerCapSection::new();
        let performance_level_frame = PerformanceFrame::new();
        let clocks_frame = ClocksFrame::new();
        let power_states_frame = PowerStatesFrame::new();

        performance_level_frame.connect_settings_changed(
            clone!(@strong performance_level_frame, @strong power_states_frame => move || {
                let level = performance_level_frame.get_selected_performance_level();
                power_states_frame.set_configurable(level == PerformanceLevel::Manual);
            }),
        );

        vbox.append(&power_cap_section);
        vbox.append(&performance_level_frame.container);
        vbox.append(&power_states_frame);
        vbox.append(&clocks_frame.container);

        container.set_child(Some(&vbox));

        Self {
            container,
            stats_section,
            performance_frame: performance_level_frame,
            clocks_frame,
            power_cap_section,
            enable_overclocking_button,
            power_states_frame,
        }
    }

    pub fn set_stats(&self, stats: &DeviceStats, initial: bool) {
        self.stats_section.set_stats(stats);
        self.power_states_frame.set_stats(stats);
        if initial {
            self.power_cap_section
                .set_max_value(stats.power.cap_max.unwrap_or_default());
            self.power_cap_section
                .set_min_value(stats.power.cap_min.unwrap_or_default());
            self.power_cap_section
                .set_default_value(stats.power.cap_default.unwrap_or_default());

            if let Some(current_cap) = stats.power.cap_current {
                self.power_cap_section.set_initial_value(current_cap);
                self.power_cap_section.set_visible(true);
            } else {
                self.power_cap_section.set_visible(false);
            }

            self.set_performance_level(stats.performance_level);
        }
    }

    pub fn set_clocks_table(&self, table: Option<ClocksTableGen>) {
        match table {
            Some(table) => match self.clocks_frame.set_table(table) {
                Ok(()) => {
                    self.clocks_frame.show();
                }
                Err(err) => {
                    warn!("got invalid clocks table: {err:?}");
                    self.clocks_frame.hide();
                }
            },
            None => {
                self.clocks_frame.hide();
            }
        }
    }

    pub fn connect_settings_changed<F: Fn() + 'static + Clone>(&self, f: F) {
        self.performance_frame.connect_settings_changed(f.clone());
        self.power_cap_section
            .connect_current_value_notify(clone!(@strong f => move |_| f()));
        self.clocks_frame.connect_clocks_changed(f.clone());
        self.power_states_frame.connect_values_changed(f);
    }

    pub fn set_performance_level(&self, profile: Option<PerformanceLevel>) {
        match profile {
            Some(profile) => {
                self.performance_frame.show();
                self.performance_frame.set_active_level(profile);
            }
            None => self.performance_frame.hide(),
        }
    }

    pub fn get_performance_level(&self) -> Option<PerformanceLevel> {
        if self.performance_frame.get_visibility() {
            let level = self.performance_frame.get_selected_performance_level();
            Some(level)
        } else {
            None
        }
    }

    pub fn get_power_cap(&self) -> Option<f64> {
        self.power_cap_section.get_user_cap()
    }

    pub fn get_enabled_power_states(&self) -> HashMap<PowerLevelKind, Vec<u8>> {
        if self.performance_frame.get_selected_performance_level() == PerformanceLevel::Manual {
            self.power_states_frame.get_enabled_power_states()
        } else {
            HashMap::new()
        }
    }
}

fn oc_warning_frame() -> (Frame, Button) {
    let container = Frame::new(Some("Overclocking information"));

    container.set_label_align(0.3);

    let vbox = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(5)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(10)
        .margin_end(10)
        .build();

    let warning_label = Label::builder()
        .use_markup(true)
        .label(OVERCLOCKING_DISABLED_TEXT)
        .wrap(true)
        .wrap_mode(pango::WrapMode::Word)
        .build();

    let enable_button = Button::builder()
        .label("Enable Overclocking")
        .halign(Align::End)
        .build();

    vbox.append(&warning_label);
    vbox.append(&enable_button);

    container.set_child(Some(&vbox));

    (container, enable_button)
}
