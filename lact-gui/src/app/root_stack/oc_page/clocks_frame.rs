use super::{oc_adjustment, section_box};
use anyhow::{anyhow, Context};
use gtk::prelude::*;
use gtk::*;
use lact_client::schema::{ClocksTable, ClocksTableGen};

#[derive(Clone)]
pub struct ClocksFrame {
    pub container: Box,
    max_sclk_adjustment: Adjustment,
    max_mclk_adjustment: Adjustment,
}

impl ClocksFrame {
    pub fn new() -> Self {
        let container = section_box("Maximum Clocks", 0, 5);
        container.hide();

        let (sclk_box, max_sclk_adjustment) = oc_adjustment(None, "MHz");
        let (mclk_box, max_mclk_adjustment) = oc_adjustment(None, "MHz");

        container.append(&sclk_box);
        container.append(&mclk_box);

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

/*fn scale(adjustment: &Adjustment) -> Scale {
    Scale::builder()
        .orientation(Orientation::Horizontal)
        .adjustment(adjustment)
        .draw_value(true)
        .hexpand(true)
        .build()
}

fn range_label() -> Label {
    Label::builder()
        .yalign(0.7)
        .margin_start(10)
        .margin_end(10)
        .build()
}*/
