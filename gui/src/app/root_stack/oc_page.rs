mod power_profile_frame;
mod stats_grid;

use daemon::gpu_controller::{GpuStats, PowerProfile};
use gtk::*;

use power_profile_frame::PowerProfileFrame;
use stats_grid::StatsGrid;

#[derive(Clone)]
pub struct OcPage {
    pub container: Box,
    stats_grid: StatsGrid,
    power_profile_frame: PowerProfileFrame,
}

impl OcPage {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Vertical, 5);

        let stats_grid = StatsGrid::new();

        container.pack_start(&stats_grid.container, false, true, 10);

        let power_profile_frame = PowerProfileFrame::new();

        container.pack_start(&power_profile_frame.container, false, true, 10);

        Self {
            container,
            stats_grid,
            power_profile_frame,
        }
    }

    pub fn set_stats(&self, stats: &GpuStats) {
        self.stats_grid.set_stats(stats);
    }

    pub fn connect_settings_changed<F: Fn() + 'static>(&self, f: F) {
        self.power_profile_frame
            .connect_power_profile_changed(move || {
                f();
            });
    }

    pub fn set_power_profile(&self, profile: &Option<PowerProfile>) {
        match profile {
            Some(profile) => {
                self.power_profile_frame.show();
                self.power_profile_frame.set_active_profile(profile);
            }
            None => self.power_profile_frame.hide(),
        }
    }

    pub fn get_power_profile(&self) -> Option<PowerProfile> {
        match self.power_profile_frame.get_visibility() {
            true => Some(self.power_profile_frame.get_selected_power_profile()),
            false => None,
        }
    }
}
