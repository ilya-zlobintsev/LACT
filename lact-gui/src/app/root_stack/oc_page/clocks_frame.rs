use super::section_box;
use anyhow::{anyhow, Context};
use gtk::prelude::*;
use gtk::*;
use lact_client::schema::{ClocksTable, ClocksTableGen};

#[derive(Clone)]
pub struct ClocksFrame {
    pub container: Box,
    tweaking_grid: Grid,
    max_sclk_adjustment: Adjustment,
    max_mclk_adjustment: Adjustment,
    max_voltage_adjustment: Adjustment,
    clocks_data_unavailable_label: Label,
}

impl ClocksFrame {
    pub fn new() -> Self {
        let container = section_box("Maximum Clocks", 0, 5);

        let tweaking_grid = Grid::new();
        let max_sclk_adjustment = oc_adjustment("GPU Clock (MHz)", &tweaking_grid, 0);
        let max_voltage_adjustment = oc_adjustment("GPU voltage (mV)", &tweaking_grid, 1);
        let max_mclk_adjustment = oc_adjustment("VRAM Clock (MHz)", &tweaking_grid, 2);

        let clocks_data_unavailable_label = Label::new(Some("No clocks data available"));

        container.append(&tweaking_grid);
        container.append(&clocks_data_unavailable_label);

        Self {
            container,
            tweaking_grid,
            max_sclk_adjustment,
            max_mclk_adjustment,
            max_voltage_adjustment,
            clocks_data_unavailable_label,
        }
    }

    pub fn set_table(&self, table: ClocksTableGen) -> anyhow::Result<()> {
        let current_sclk_max = table.get_max_sclk().context("No max sclk clockspeed")?;
        let current_mclk_max = table.get_max_mclk().context("No max mclk clockspeed")?;
        let current_voltage_max = table
            .get_max_sclk_voltage()
            .context("No max sclk voltage")?;

        let ranges = table.get_allowed_ranges();
        let (sclk_min, sclk_max) = ranges
            .sclk
            .try_into()
            .map_err(|_| anyhow!("No sclk range"))?;
        let (mclk_min, mclk_max) = ranges
            .mclk
            .and_then(|range| range.try_into().ok())
            .context("No mclk range")?;
        // TODO
        // let (voltage_min, voltage_max) = ranges.vddc.

        self.max_sclk_adjustment.set_lower(sclk_min.into());
        self.max_sclk_adjustment.set_upper(sclk_max.into());
        self.max_sclk_adjustment.set_value(current_sclk_max.into());

        self.max_mclk_adjustment.set_lower(mclk_min.into());
        self.max_mclk_adjustment.set_upper(mclk_max.into());
        self.max_mclk_adjustment.set_value(current_mclk_max.into());

        Ok(())
    }

    pub fn show(&self) {
        self.tweaking_grid.show();
        self.clocks_data_unavailable_label.hide();
    }

    pub fn hide(&self) {
        self.tweaking_grid.hide();
        self.clocks_data_unavailable_label.show();
    }
}

fn oc_adjustment(title: &'static str, grid: &Grid, row: i32) -> Adjustment {
    let label = Label::new(Some(title));

    let adjustment = Adjustment::new(0.0, 0.0, 0.0, 1.0, 10.0, 0.0);
    let scale = Scale::builder()
        .orientation(Orientation::Horizontal)
        .adjustment(&adjustment)
        .hexpand(true)
        .round_digits(0)
        .digits(0)
        .draw_value(true)
        .value_pos(PositionType::Right)
        .build();

    grid.attach(&label, 0, row, 1, 1);
    grid.attach(&scale, 1, row, 4, 1);

    adjustment
}
