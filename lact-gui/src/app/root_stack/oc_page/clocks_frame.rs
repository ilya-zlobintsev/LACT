use crate::app::root_stack::section_box;
use glib::clone;
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
    reset_button: Button,
    clocks_data_unavailable_label: Label,
}

impl ClocksFrame {
    pub fn new() -> Self {
        let container = section_box("Maximum Clocks");

        let tweaking_grid = Grid::builder().row_spacing(5).build();
        let max_sclk_adjustment = oc_adjustment("GPU Clock (MHz)", &tweaking_grid, 0);
        let max_voltage_adjustment = oc_adjustment("GPU voltage (mV)", &tweaking_grid, 1);
        let max_mclk_adjustment = oc_adjustment("VRAM Clock (MHz)", &tweaking_grid, 2);

        let reset_button = Button::builder()
            .label("Defaults")
            .halign(Align::End)
            .build();
        tweaking_grid.attach(&reset_button, 6, 3, 1, 1);

        let clocks_data_unavailable_label = Label::new(Some("No clocks data available"));

        container.append(&tweaking_grid);
        container.append(&clocks_data_unavailable_label);

        Self {
            container,
            tweaking_grid,
            max_sclk_adjustment,
            max_mclk_adjustment,
            max_voltage_adjustment,
            reset_button,
            clocks_data_unavailable_label,
        }
    }

    pub fn set_table(&self, table: ClocksTableGen) -> anyhow::Result<()> {
        if let Some((current_sclk_max, sclk_min, sclk_max)) =
            extract_value_and_range(&table, |table| {
                (table.get_max_sclk(), table.get_max_sclk_range())
            })
        {
            self.max_sclk_adjustment.set_lower(sclk_min.into());
            self.max_sclk_adjustment.set_upper(sclk_max.into());
            self.max_sclk_adjustment.set_value(current_sclk_max.into());
        }

        if let Some((current_mclk_max, mclk_min, mclk_max)) =
            extract_value_and_range(&table, |table| {
                (table.get_max_mclk(), table.get_max_mclk_range())
            })
        {
            self.max_mclk_adjustment.set_lower(mclk_min.into());
            self.max_mclk_adjustment.set_upper(mclk_max.into());
            self.max_mclk_adjustment.set_value(current_mclk_max.into());
        }

        if let Some((current_voltage_max, voltage_min, voltage_max)) =
            extract_value_and_range(&table, |table| {
                (table.get_max_sclk_voltage(), table.get_max_voltage_range())
            })
        {
            self.max_voltage_adjustment.set_lower(voltage_min.into());
            self.max_voltage_adjustment.set_upper(voltage_max.into());
            self.max_voltage_adjustment
                .set_value(current_voltage_max.into());
        }

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

    pub fn connect_clocks_changed<F: Fn() + 'static + Clone>(&self, f: F) {
        let f = clone!(@strong f => move |_: &Adjustment| f());
        self.max_sclk_adjustment.connect_value_changed(f.clone());
        self.max_mclk_adjustment.connect_value_changed(f.clone());
        self.max_voltage_adjustment.connect_value_changed(f);
    }

    pub fn connect_clocks_reset<F: Fn() + 'static + Clone>(&self, f: F) {
        self.reset_button.connect_clicked(move |_| f());
    }

    pub fn get_settings(&self) -> ClocksSettings {
        if self.tweaking_grid.is_visible() {
            let max_core_clock = zero_to_option(self.max_sclk_adjustment.value());
            let max_memory_clock = zero_to_option(self.max_mclk_adjustment.value());
            let max_voltage = zero_to_option(self.max_voltage_adjustment.value());

            ClocksSettings {
                max_core_clock,
                max_memory_clock,
                max_voltage,
            }
        } else {
            ClocksSettings::default()
        }
    }
}

fn extract_value_and_range(
    table: &ClocksTableGen,
    f: fn(&ClocksTableGen) -> (Option<u32>, Option<lact_client::schema::Range>),
) -> Option<(u32, u32, u32)> {
    let (maybe_value, maybe_range) = f(table);
    let (value, range) = maybe_value.zip(maybe_range)?;
    let (min, max) = range.try_into().ok()?;
    Some((value, min, max))
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
        .value_pos(PositionType::Right)
        .margin_start(5)
        .margin_end(5)
        .build();

    let value_selector = SpinButton::new(Some(&adjustment), 1.0, 0);
    let value_label = Label::new(None);

    adjustment.connect_value_changed(clone!(@strong value_label => move |adjustment| {
        let value = adjustment.value();
        value_label.set_text(&value.to_string());
    }));

    let popover = Popover::builder().child(&value_selector).build();
    let value_button = MenuButton::builder()
        .popover(&popover)
        .child(&value_label)
        .build();

    grid.attach(&label, 0, row, 1, 1);
    grid.attach(&scale, 1, row, 4, 1);
    grid.attach(&value_button, 6, row, 4, 1);

    adjustment
}

#[derive(Debug, Default)]
pub struct ClocksSettings {
    pub max_core_clock: Option<u32>,
    pub max_memory_clock: Option<u32>,
    pub max_voltage: Option<u32>,
}

fn zero_to_option(value: f64) -> Option<u32> {
    if value == 0.0 {
        None
    } else {
        Some(value as u32)
    }
}
