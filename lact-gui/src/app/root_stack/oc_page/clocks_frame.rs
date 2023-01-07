use anyhow::{anyhow, Context};
use gtk::prelude::*;
use gtk::*;
use lact_client::schema::{ClocksTable, ClocksTableGen};

use super::section_box;

#[derive(Clone)]
pub struct ClocksFrame {
    pub container: Box,
    pub max_sclk_adjustment: Adjustment,
    pub max_mclk_adjustment: Adjustment,
}

impl ClocksFrame {
    pub fn new() -> Self {
        let container = section_box("Maximum Clocks");
        container.hide();

        let max_sclk_adjustment = Adjustment::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let max_mclk_adjustment = Adjustment::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);

        let max_sclk_scale = scale(&max_sclk_adjustment);
        let max_mclk_scale = scale(&max_mclk_adjustment);

        container.append(&max_sclk_scale);
        container.append(&max_mclk_scale);

        Self {
            container,
            max_sclk_adjustment,
            max_mclk_adjustment,
        }
    }

    pub fn set_table(&self, table: ClocksTableGen) -> anyhow::Result<()> {
        let current_sclk_max = table.get_max_sclk().context("No max sclk clockspeed")?;
        let current_mclk_max = table.get_max_mclk().context("No max mclk clockspeed")?;

        let ranges = table.get_allowed_ranges();
        let (sclk_min, sclk_max) = ranges
            .sclk
            .try_into()
            .map_err(|_| anyhow!("No sclk range"))?;
        let (mclk_min, mclk_max) = ranges
            .mclk
            .and_then(|range| range.try_into().ok())
            .context("No mclk range")?;

        self.max_sclk_adjustment.set_lower(sclk_min.into());
        self.max_sclk_adjustment.set_upper(sclk_max.into());
        self.max_sclk_adjustment.set_value(current_sclk_max.into());

        self.max_mclk_adjustment.set_lower(mclk_min.into());
        self.max_mclk_adjustment.set_upper(mclk_max.into());
        self.max_mclk_adjustment.set_value(current_mclk_max.into());

        Ok(())
    }
}

fn scale(adjustment: &Adjustment) -> Scale {
    Scale::builder()
        .orientation(Orientation::Horizontal)
        .adjustment(adjustment)
        .draw_value(true)
        .build()
}
