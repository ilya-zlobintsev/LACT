mod stats_grid;

use daemon::gpu_controller::GpuStats;
use gtk::*;
use stats_grid::StatsGrid;

#[derive(Clone)]
pub struct OcPage {
    pub container: Box,
    stats_grid: StatsGrid,
}

impl OcPage {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Vertical, 5);

        let stats_grid = StatsGrid::new();

        container.pack_start(&stats_grid.container, false, true, 10);

        Self { container, stats_grid }
    }
    
    pub fn set_stats(&self, stats: &GpuStats) {
        self.stats_grid.set_stats(stats);
    }
}
