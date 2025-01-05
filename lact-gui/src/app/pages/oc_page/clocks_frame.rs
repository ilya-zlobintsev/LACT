mod adjustment_row;

use crate::app::page_section::PageSection;
use crate::app::pages::oc_adjustment::OcAdjustment;
use adjustment_row::AdjustmentRow;
use amdgpu_sysfs::gpu_handle::overdrive::{ClocksTable as _, ClocksTableGen as AmdClocksTable};
use glib::clone;
use gtk::prelude::*;
use gtk::*;
use lact_schema::{request::SetClocksCommand, ClocksTable, NvidiaClockInfo, NvidiaClocksTable};
use subclass::prelude::ObjectSubclassIsExt;
use tracing::debug;

const DEFAULT_VOLTAGE_OFFSET_RANGE: i32 = 250;
const WARNING_TEXT: &str = "Warning: changing these values may lead to system instability and potentially damage your hardware!";

// The AtomicBool stores if the value was changed
#[derive(Clone)]
pub struct ClocksFrame {
    pub container: PageSection,
    tweaking_grid: Grid,
    modes_switcher_box: Box,
    basic_togglebutton: ToggleButton,
    advanced_togglebutton: ToggleButton,
    min_values_grid: Grid,
    min_sclk_adjustment: AdjustmentRow,
    min_mclk_adjustment: AdjustmentRow,
    min_voltage_adjustment: AdjustmentRow,
    max_sclk_adjustment: AdjustmentRow,
    max_mclk_adjustment: AdjustmentRow,
    max_voltage_adjustment: AdjustmentRow,
    voltage_offset_adjustment: AdjustmentRow,
    reset_button: Button,
    warning_label: Label,
    clocks_data_unavailable_label: Label,
}

impl ClocksFrame {
    pub fn new() -> Self {
        let container = PageSection::new("Clockspeed and voltage");

        let warning_label = Label::builder()
            .label(WARNING_TEXT)
            .wrap_mode(pango::WrapMode::Word)
            .halign(Align::Start)
            .margin_top(5)
            .margin_bottom(5)
            .build();
        container.append(&warning_label);

        let modes_switcher_box = Box::new(Orientation::Horizontal, 0);

        let modes_switcher_label = Label::builder()
            .label("Configuration mode:")
            .hexpand(true)
            .halign(Align::Start)
            .build();
        let basic_togglebutton = ToggleButton::builder().label("Basic").build();
        let advanced_togglebutton = ToggleButton::builder().label("Advanced").build();

        modes_switcher_box.append(&modes_switcher_label);
        modes_switcher_box.append(&basic_togglebutton);
        modes_switcher_box.append(&advanced_togglebutton);

        container.append(&modes_switcher_box);

        let min_values_grid = Grid::builder().row_spacing(5).build();

        let min_sclk_adjustment =
            AdjustmentRow::new_and_attach("Minimum GPU Clock (MHz)", &min_values_grid, 0);
        let min_mclk_adjustment =
            AdjustmentRow::new_and_attach("Minimum VRAM Clock (MHz)", &min_values_grid, 1);
        let min_voltage_adjustment =
            AdjustmentRow::new_and_attach("Minimum GPU voltage (mV)", &min_values_grid, 2);

        container.append(&min_values_grid);

        let tweaking_grid = Grid::builder().row_spacing(5).build();

        let max_sclk_adjustment =
            AdjustmentRow::new_and_attach("Maximum GPU Clock (MHz)", &tweaking_grid, 1);
        let max_voltage_adjustment =
            AdjustmentRow::new_and_attach("Maximum GPU voltage (mV)", &tweaking_grid, 2);
        let max_mclk_adjustment =
            AdjustmentRow::new_and_attach("Maximum VRAM Clock (MHz)", &tweaking_grid, 3);
        let voltage_offset_adjustment =
            AdjustmentRow::new_and_attach("GPU voltage offset (mV)", &tweaking_grid, 4);

        let reset_button = Button::builder()
            .label("Reset")
            .halign(Align::Fill)
            .margin_top(5)
            .margin_bottom(5)
            .tooltip_text("Warning: this resets all clock settings to defaults!")
            .css_classes(["destructive-action"])
            .build();
        tweaking_grid.attach(&reset_button, 6, 5, 1, 1);

        let clocks_data_unavailable_label = Label::builder()
            .label("No clocks data available")
            .margin_start(10)
            .margin_end(10)
            .halign(Align::Start)
            .build();

        container.append(&tweaking_grid);
        container.append(&clocks_data_unavailable_label);

        let frame = Self {
            container,
            tweaking_grid,
            min_sclk_adjustment,
            min_mclk_adjustment,
            min_voltage_adjustment,
            max_sclk_adjustment,
            max_mclk_adjustment,
            max_voltage_adjustment,
            reset_button,
            clocks_data_unavailable_label,
            voltage_offset_adjustment,
            advanced_togglebutton,
            basic_togglebutton,
            min_values_grid,
            warning_label,
            modes_switcher_box,
        };

        frame.set_configuration_mode(false);

        frame.basic_togglebutton.connect_clicked(clone!(
            #[strong]
            frame,
            move |button| {
                frame.set_configuration_mode(!button.is_active());
            }
        ));
        frame.advanced_togglebutton.connect_clicked(clone!(
            #[strong]
            frame,
            move |button| {
                frame.set_configuration_mode(button.is_active());
            }
        ));

        frame
    }

    pub fn set_table(&self, table: ClocksTable) -> anyhow::Result<()> {
        debug!("using clocks table {table:?}");

        let adjustments = [
            &self.min_sclk_adjustment,
            &self.min_mclk_adjustment,
            &self.min_voltage_adjustment,
            &self.max_sclk_adjustment,
            &self.max_mclk_adjustment,
            &self.max_voltage_adjustment,
            &self.voltage_offset_adjustment,
        ];

        for adjustment in adjustments {
            adjustment.set_visible(false);
        }

        match table {
            ClocksTable::Amd(table) => self.set_amd_table(table),
            ClocksTable::Nvidia(table) => self.set_nvidia_table(table),
        }

        for adjustment in adjustments {
            adjustment.refresh();
        }

        Ok(())
    }

    fn set_amd_table(&self, table: AmdClocksTable) {
        if let Some((current_sclk_min, sclk_min, sclk_max)) =
            extract_value_and_range_amd(&table, |table| {
                (
                    table.get_current_sclk_range().min,
                    table.get_min_sclk_range(),
                )
            })
        {
            let min_sclk_adjustment = &self.min_sclk_adjustment.imp().adjustment;
            min_sclk_adjustment.set_lower(sclk_min.into());
            min_sclk_adjustment.set_upper(sclk_max.into());
            min_sclk_adjustment.set_initial_value(current_sclk_min.into());

            self.min_sclk_adjustment.set_visible(true);
        }

        if let Some((current_mclk_min, mclk_min, mclk_max)) =
            extract_value_and_range_amd(&table, |table| {
                (
                    table.get_current_mclk_range().min,
                    table.get_min_mclk_range(),
                )
            })
        {
            let min_mclk_adjustment = &self.min_mclk_adjustment.imp().adjustment;
            min_mclk_adjustment.set_lower(mclk_min.into());
            min_mclk_adjustment.set_upper(mclk_max.into());
            min_mclk_adjustment.set_initial_value(current_mclk_min.into());

            self.min_mclk_adjustment.set_visible(true);
        }

        if let Some((current_min_voltage, voltage_min, voltage_max)) =
            extract_value_and_range_amd(&table, |table| {
                (
                    table
                        .get_current_voltage_range()
                        .and_then(|range| range.min),
                    table.get_min_voltage_range(),
                )
            })
        {
            let min_voltage_adjustment = &self.min_voltage_adjustment.imp().adjustment;

            min_voltage_adjustment.set_lower(voltage_min.into());
            min_voltage_adjustment.set_upper(voltage_max.into());
            min_voltage_adjustment.set_value(current_min_voltage.into());

            self.min_voltage_adjustment.set_visible(true);
        }

        if let Some((current_sclk_max, sclk_min, sclk_max)) =
            extract_value_and_range_amd(&table, |table| {
                (table.get_max_sclk(), table.get_max_sclk_range())
            })
        {
            let max_sclk_adjustment = &self.max_sclk_adjustment.imp().adjustment;

            max_sclk_adjustment.set_lower(sclk_min.into());
            max_sclk_adjustment.set_upper(sclk_max.into());
            max_sclk_adjustment.set_value(current_sclk_max.into());

            self.max_sclk_adjustment.set_visible(true);
        }

        if let Some((current_mclk_max, mclk_min, mclk_max)) =
            extract_value_and_range_amd(&table, |table| {
                (table.get_max_mclk(), table.get_max_mclk_range())
            })
        {
            let max_mclk_adjustment = &self.max_mclk_adjustment.imp().adjustment;
            max_mclk_adjustment.set_lower(mclk_min.into());
            max_mclk_adjustment.set_upper(mclk_max.into());
            max_mclk_adjustment.set_value(current_mclk_max.into());

            self.max_mclk_adjustment.set_visible(true);
        }

        if let Some((current_voltage_max, voltage_min, voltage_max)) =
            extract_value_and_range_amd(&table, |table| {
                (table.get_max_sclk_voltage(), table.get_max_voltage_range())
            })
        {
            let max_voltage_adjustment = &self.max_voltage_adjustment.imp().adjustment;
            max_voltage_adjustment.set_lower(voltage_min.into());
            max_voltage_adjustment.set_upper(voltage_max.into());
            max_voltage_adjustment.set_value(current_voltage_max.into());

            self.max_voltage_adjustment.set_visible(true);
        }

        if let AmdClocksTable::Vega20(table) = table {
            if let Some(offset) = table.voltage_offset {
                let (min_offset, max_offset) = table
                    .od_range
                    .voltage_offset
                    .and_then(|range| range.into_full())
                    .unwrap_or((-DEFAULT_VOLTAGE_OFFSET_RANGE, DEFAULT_VOLTAGE_OFFSET_RANGE));

                let voltage_offset_adjustment = &self.voltage_offset_adjustment.imp().adjustment;
                voltage_offset_adjustment.set_lower(min_offset as f64);
                voltage_offset_adjustment.set_upper(max_offset as f64);
                voltage_offset_adjustment.set_value(offset.into());

                self.voltage_offset_adjustment.set_visible(true);
            }
        }
    }

    fn set_nvidia_table(&self, table: NvidiaClocksTable) {
        if let Some(gpc_info) = &table.gpc {
            set_nvidia_clock_offset(gpc_info, &self.max_sclk_adjustment);
        }
        if let Some(mem_info) = &table.mem {
            set_nvidia_clock_offset(mem_info, &self.max_mclk_adjustment);
        }
    }

    pub fn show(&self) {
        self.tweaking_grid.show();
        self.modes_switcher_box.show();
        self.warning_label.show();
        self.clocks_data_unavailable_label.hide();
    }

    pub fn hide(&self) {
        self.tweaking_grid.hide();
        self.modes_switcher_box.hide();
        self.warning_label.hide();
        self.clocks_data_unavailable_label.show();
    }

    pub fn set_vram_clock_ratio(&self, ratio: f64) {
        self.min_mclk_adjustment.set_value_ratio(ratio);
        self.max_mclk_adjustment.set_value_ratio(ratio);
    }

    pub fn connect_clocks_changed<F: Fn() + 'static + Clone>(&self, f: F) {
        let f = clone!(
            #[strong]
            f,
            move |_: &OcAdjustment| f()
        );

        self.min_sclk_adjustment
            .imp()
            .adjustment
            .connect_value_changed(f.clone());
        self.min_mclk_adjustment
            .imp()
            .adjustment
            .connect_value_changed(f.clone());
        self.min_voltage_adjustment
            .imp()
            .adjustment
            .connect_value_changed(f.clone());
        self.max_sclk_adjustment
            .imp()
            .adjustment
            .connect_value_changed(f.clone());
        self.max_mclk_adjustment
            .imp()
            .adjustment
            .connect_value_changed(f.clone());
        self.max_voltage_adjustment
            .imp()
            .adjustment
            .connect_value_changed(f.clone());
        self.voltage_offset_adjustment
            .imp()
            .adjustment
            .connect_value_changed(f);
    }

    pub fn connect_clocks_reset<F: Fn() + 'static + Clone>(&self, f: F) {
        self.reset_button.connect_clicked(move |_| f());
    }

    pub fn get_commands(&self) -> Vec<SetClocksCommand> {
        if self.tweaking_grid.get_visible() {
            type ClocksCommandFn = fn(i32) -> SetClocksCommand;

            let adjustments: &[(&AdjustmentRow, ClocksCommandFn)] = &[
                (&self.min_sclk_adjustment, SetClocksCommand::MinCoreClock),
                (&self.min_mclk_adjustment, SetClocksCommand::MinMemoryClock),
                (&self.min_voltage_adjustment, SetClocksCommand::MinVoltage),
                (&self.max_sclk_adjustment, SetClocksCommand::MaxCoreClock),
                (&self.max_mclk_adjustment, SetClocksCommand::MaxMemoryClock),
                (&self.max_voltage_adjustment, SetClocksCommand::MaxVoltage),
            ];
            let mut commands: Vec<SetClocksCommand> = adjustments
                .iter()
                .filter_map(|(row, f)| {
                    let value = row.get_value()?;
                    Some(f(value))
                })
                .collect();

            if self.voltage_offset_adjustment.get_visible() {
                if let Some(offset) = self.voltage_offset_adjustment.get_value() {
                    commands.push(SetClocksCommand::VoltageOffset(offset));
                }
            }

            commands
        } else {
            vec![]
        }
    }

    fn set_configuration_mode(&self, advanced: bool) {
        self.advanced_togglebutton.set_active(advanced);
        self.basic_togglebutton.set_active(!advanced);

        self.min_values_grid.set_visible(advanced);
    }
}

fn extract_value_and_range_amd(
    table: &AmdClocksTable,
    f: fn(
        &AmdClocksTable,
    ) -> (
        Option<i32>,
        Option<amdgpu_sysfs::gpu_handle::overdrive::Range>,
    ),
) -> Option<(i32, i32, i32)> {
    let (maybe_value, maybe_range) = f(table);
    let (value, range) = maybe_value.zip(maybe_range)?;
    let (min, max) = range.try_into().ok()?;
    Some((value, min, max))
}

fn set_nvidia_clock_offset(clock_info: &NvidiaClockInfo, adjustment_row: &AdjustmentRow) {
    let oc_adjustment = &adjustment_row.imp().adjustment;
    oc_adjustment.set_lower((clock_info.max + clock_info.offset_range.0) as f64);
    oc_adjustment.set_upper((clock_info.max + clock_info.offset_range.1) as f64);
    oc_adjustment
        .set_value((clock_info.max + (clock_info.offset / clock_info.offset_ratio)) as f64);

    adjustment_row.set_visible(true);
}
