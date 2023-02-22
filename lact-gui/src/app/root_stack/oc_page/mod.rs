mod clocks_frame;
mod performance_level_frame;
mod power_cap_frame;
mod stats_grid;

use clocks_frame::ClocksFrame;
use gtk::prelude::*;
use gtk::*;
use lact_client::schema::{ClocksTableGen, DeviceStats, PerformanceLevel, SystemInfo};
use performance_level_frame::PerformanceLevelFrame;
use power_cap_frame::PowerCapFrame;
use stats_grid::StatsGrid;
use tracing::warn;

#[derive(Clone)]
pub struct OcPage {
    pub container: Box,
    stats_grid: StatsGrid,
    performance_level_frame: PerformanceLevelFrame,
    power_cap_frame: PowerCapFrame,
    pub clocks_frame: ClocksFrame,
}

impl OcPage {
    pub fn new(system_info: &SystemInfo) -> Self {
        let container = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(5)
            .margin_top(5)
            .margin_bottom(5)
            .build();

        if system_info.amdgpu_overdrive_enabled == Some(false) {
            let warning_frame = oc_warning_frame();
            container.append(&warning_frame);
        }

        let stats_grid = StatsGrid::new();

        container.append(&stats_grid.container);

        let power_cap_frame = PowerCapFrame::new();
        let performance_level_frame = PerformanceLevelFrame::new();
        let clocks_frame = ClocksFrame::new();

        container.append(&power_cap_frame.container);
        container.append(&performance_level_frame.container);
        container.append(&clocks_frame.container);

        Self {
            container,
            stats_grid,
            performance_level_frame,
            clocks_frame,
            power_cap_frame,
        }
    }

    pub fn set_stats(&self, stats: &DeviceStats, initial: bool) {
        self.stats_grid.set_stats(stats);
        if initial {
            self.power_cap_frame.set_data(
                stats.power.cap_current,
                stats.power.cap_max,
                stats.power.cap_default,
            );
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
        self.performance_level_frame
            .connect_power_profile_changed(f.clone());
        self.power_cap_frame.connect_cap_changed(f.clone());
        self.clocks_frame.connect_clocks_changed(f);
    }

    pub fn set_performance_level(&self, profile: Option<PerformanceLevel>) {
        match profile {
            Some(profile) => {
                self.performance_level_frame.show();
                self.performance_level_frame.set_active_profile(profile);
            }
            None => self.performance_level_frame.hide(),
        }
    }

    pub fn get_performance_level(&self) -> Option<PerformanceLevel> {
        if self.performance_level_frame.get_visibility() {
            let level = self
                .performance_level_frame
                .get_selected_performance_level();
            Some(level)
        } else {
            None
        }
    }

    /*pub fn get_clocks(&self) -> Option<ClocksSettings> {
        match self.clocks_frame.get_visibility() {
            true => Some(self.clocks_frame.get_settings()),
            false => None,
        }
    }*/

    pub fn get_power_cap(&self) -> Option<f64> {
        self.power_cap_frame.get_cap()
    }
}

fn section_box(title: &str, spacing: i32, margin: i32) -> Box {
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(spacing)
        .margin_start(margin)
        .margin_end(margin)
        .build();

    let label = Label::builder()
        .use_markup(true)
        .label(format!("<span font_desc='11'><b>{title}</b></span>"))
        .xalign(0.1)
        .build();

    container.append(&label);
    container
}

fn oc_warning_frame() -> Frame {
    let container = Frame::new(Some("Overclocking information"));

    container.set_label_align(0.3);

    let warning_label = Label::new(None);

    warning_label.set_wrap(true);
    warning_label.set_markup("Overclocking support is not enabled! To enable overclocking support, you need to add <b>amdgpu.ppfeaturemask=0xffffffff</b> to your kernel boot options. Look for the documentation of your distro.");
    warning_label.set_selectable(true);

    container.set_child(Some(&warning_label));
    container
}
