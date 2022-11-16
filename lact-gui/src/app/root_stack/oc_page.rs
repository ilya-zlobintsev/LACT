mod clocks_frame;
mod performance_level_frame;
mod power_cap_frame;
mod stats_grid;
mod warning_frame;

use amdgpu_sysfs::gpu_handle::PerformanceLevel;
use clocks_frame::ClocksFrame;
use clocks_frame::ClocksSettings;
use gtk::prelude::*;
use gtk::*;
use lact_schema::{DeviceInfo, DeviceStats};
use performance_level_frame::PowerProfileFrame;
use power_cap_frame::PowerCapFrame;
use stats_grid::StatsGrid;
use warning_frame::WarningFrame;

#[derive(Clone)]
pub struct OcPage {
    pub container: Box,
    stats_grid: StatsGrid,
    performance_level_frame: PowerProfileFrame,
    power_cap_frame: PowerCapFrame,
    clocks_frame: ClocksFrame,
    pub warning_frame: WarningFrame,
}

impl OcPage {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Vertical, 5);

        let warning_frame = WarningFrame::new();

        container.pack_start(&warning_frame.container, false, true, 5);

        let stats_grid = StatsGrid::new();

        container.pack_start(&stats_grid.container, false, true, 5);

        let power_cap_frame = PowerCapFrame::new();

        container.pack_start(&power_cap_frame.container, false, true, 0);

        let power_profile_frame = PowerProfileFrame::new();

        container.pack_start(&power_profile_frame.container, false, true, 0);

        let clocks_frame = ClocksFrame::new();

        container.pack_start(&clocks_frame.container, false, true, 0);

        Self {
            container,
            stats_grid,
            performance_level_frame: power_profile_frame,
            clocks_frame,
            warning_frame,
            power_cap_frame,
        }
    }

    pub fn set_stats(&self, stats: &DeviceStats) {
        self.stats_grid.set_stats(stats);
    }

    pub fn connect_clocks_reset<F: Fn() + 'static + Clone>(&self, f: F) {
        self.clocks_frame.connect_clocks_reset(move || {
            f();
        });
    }

    pub fn connect_settings_changed<F: Fn() + 'static + Clone>(&self, f: F) {
        {
            let f = f.clone();
            self.performance_level_frame
                .connect_power_profile_changed(move || {
                    f();
                });
        }
        {
            let f = f.clone();
            self.clocks_frame.connect_clocks_changed(move || {
                f();
            })
        }
        {
            self.power_cap_frame.connect_cap_changed(move || {
                f();
            })
        }
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

    pub fn get_power_profile(&self) -> Option<PowerProfile> {
        match self.performance_level_frame.get_visibility() {
            true => Some(self.performance_level_frame.get_selected_power_profile()),
            false => None,
        }
    }

    pub fn set_info(&self, info: &DeviceInfo) {
        // TODO
        /*match &info.clocks_table {
            Some(clocks_table) => {
                self.clocks_frame.show();
                self.clocks_frame.set_clocks(clocks_table);
            }
            None => self.clocks_frame.hide(),
        }*/

        /*self.power_cap_frame
        .set_data(info.power_cap, info.power_cap_max);*/
    }

    pub fn get_clocks(&self) -> Option<ClocksSettings> {
        match self.clocks_frame.get_visibility() {
            true => Some(self.clocks_frame.get_settings()),
            false => None,
        }
    }

    pub fn get_power_cap(&self) -> Option<i64> {
        self.power_cap_frame.get_cap()
    }
}