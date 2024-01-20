use crate::app::page_section::PageSection;
use glib::clone;
use gtk::prelude::*;
use gtk::*;
use lact_client::schema::amdgpu_sysfs;
use lact_client::schema::amdgpu_sysfs::gpu_handle::overdrive::{ClocksTable, ClocksTableGen};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
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
    min_sclk_adjustment: (Adjustment, Rc<AtomicBool>),
    min_mclk_adjustment: (Adjustment, Rc<AtomicBool>),
    min_voltage_adjustment: (Adjustment, Rc<AtomicBool>),
    max_sclk_adjustment: (Adjustment, Rc<AtomicBool>),
    max_mclk_adjustment: (Adjustment, Rc<AtomicBool>),
    max_voltage_adjustment: (Adjustment, Rc<AtomicBool>),
    voltage_offset_adjustment: (Adjustment, Rc<AtomicBool>),
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

        let min_sclk_adjustment = oc_adjustment("Minimum GPU Clock (MHz)", &min_values_grid, 0);
        let min_mclk_adjustment = oc_adjustment("Minimum VRAM Clock (MHz)", &min_values_grid, 1);
        let min_voltage_adjustment = oc_adjustment("Minimum GPU voltage (mV)", &min_values_grid, 2);

        container.append(&min_values_grid);

        let tweaking_grid = Grid::builder().row_spacing(5).build();

        let max_sclk_adjustment = oc_adjustment("Maximum GPU Clock (MHz)", &tweaking_grid, 1);
        let max_voltage_adjustment = oc_adjustment("Maximum GPU voltage (mV)", &tweaking_grid, 2);
        let max_mclk_adjustment = oc_adjustment("Maximum VRAM Clock (MHz)", &tweaking_grid, 3);
        let voltage_offset_adjustment = oc_adjustment("GPU voltage offset (mV)", &tweaking_grid, 4);

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

        frame
            .basic_togglebutton
            .connect_clicked(clone!(@strong frame => move |button| {
                frame.set_configuration_mode(!button.is_active());
            }));
        frame
            .advanced_togglebutton
            .connect_clicked(clone!(@strong frame => move |button| {
                frame.set_configuration_mode(button.is_active());
            }));

        frame
    }

    pub fn set_table(&self, table: ClocksTableGen) -> anyhow::Result<()> {
        println!("using clocks table {table:?}");

        // The upper value "0.0" is used to hide the adjustment when info is not available

        if let Some((current_sclk_min, sclk_min, sclk_max)) =
            extract_value_and_range(&table, |table| {
                (
                    table.get_current_sclk_range().min,
                    table.get_min_sclk_range(),
                )
            })
        {
            self.min_sclk_adjustment.0.set_lower(sclk_min.into());
            self.min_sclk_adjustment.0.set_upper(sclk_max.into());
            self.min_sclk_adjustment
                .0
                .set_value(current_sclk_min.into());
        } else {
            self.min_sclk_adjustment.0.set_upper(0.0);
        }

        if let Some((current_mclk_min, mclk_min, mclk_max)) =
            extract_value_and_range(&table, |table| {
                (
                    table.get_current_mclk_range().min,
                    table.get_min_mclk_range(),
                )
            })
        {
            self.min_mclk_adjustment.0.set_lower(mclk_min.into());
            self.min_mclk_adjustment.0.set_upper(mclk_max.into());
            self.min_mclk_adjustment
                .0
                .set_value(current_mclk_min.into());
        } else {
            self.min_mclk_adjustment.0.set_upper(0.0);
        }

        if let Some((current_min_voltage, voltage_min, voltage_max)) =
            extract_value_and_range(&table, |table| {
                (
                    table
                        .get_current_voltage_range()
                        .and_then(|range| range.min),
                    table.get_min_voltage_range(),
                )
            })
        {
            self.min_voltage_adjustment.0.set_lower(voltage_min.into());
            self.min_voltage_adjustment.0.set_upper(voltage_max.into());
            self.min_voltage_adjustment
                .0
                .set_value(current_min_voltage.into());
        } else {
            self.min_voltage_adjustment.0.set_upper(0.0);
        }

        if let Some((current_sclk_max, sclk_min, sclk_max)) =
            extract_value_and_range(&table, |table| {
                (table.get_max_sclk(), table.get_max_sclk_range())
            })
        {
            self.max_sclk_adjustment.0.set_lower(sclk_min.into());
            self.max_sclk_adjustment.0.set_upper(sclk_max.into());
            self.max_sclk_adjustment
                .0
                .set_value(current_sclk_max.into());
        } else {
            self.max_sclk_adjustment.0.set_upper(0.0);
        }

        if let Some((current_mclk_max, mclk_min, mclk_max)) =
            extract_value_and_range(&table, |table| {
                (table.get_max_mclk(), table.get_max_mclk_range())
            })
        {
            self.max_mclk_adjustment.0.set_lower(mclk_min.into());
            self.max_mclk_adjustment.0.set_upper(mclk_max.into());
            self.max_mclk_adjustment
                .0
                .set_value(current_mclk_max.into());
        } else {
            self.max_mclk_adjustment.0.set_upper(0.0);
        }

        if let Some((current_voltage_max, voltage_min, voltage_max)) =
            extract_value_and_range(&table, |table| {
                (table.get_max_sclk_voltage(), table.get_max_voltage_range())
            })
        {
            self.max_voltage_adjustment.0.set_lower(voltage_min.into());
            self.max_voltage_adjustment.0.set_upper(voltage_max.into());
            self.max_voltage_adjustment
                .0
                .set_value(current_voltage_max.into());
        } else {
            self.max_voltage_adjustment.0.set_upper(0.0);
        }

        if let ClocksTableGen::Vega20(table) = table {
            if let Some(offset) = table.voltage_offset {
                let (min_offset, max_offset) = table
                    .od_range
                    .voltage_offset
                    .and_then(|range| range.into_full())
                    .unwrap_or((-DEFAULT_VOLTAGE_OFFSET_RANGE, DEFAULT_VOLTAGE_OFFSET_RANGE));

                self.voltage_offset_adjustment
                    .0
                    .set_lower(min_offset as f64);
                self.voltage_offset_adjustment
                    .0
                    .set_upper(max_offset as f64);
                self.voltage_offset_adjustment.0.set_value(offset.into());
            } else {
                self.voltage_offset_adjustment.0.set_upper(0.0);
            }
        } else {
            self.voltage_offset_adjustment.0.set_upper(0.0);
        }

        emit_changed(&self.min_sclk_adjustment);
        emit_changed(&self.min_mclk_adjustment);
        emit_changed(&self.min_voltage_adjustment);
        emit_changed(&self.max_sclk_adjustment);
        emit_changed(&self.max_mclk_adjustment);
        emit_changed(&self.max_voltage_adjustment);
        emit_changed(&self.voltage_offset_adjustment);

        Ok(())
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

    pub fn connect_clocks_changed<F: Fn() + 'static + Clone>(&self, f: F) {
        let f = clone!(@strong f => move |_: &Adjustment| f());
        self.min_sclk_adjustment.0.connect_value_changed(f.clone());
        self.min_mclk_adjustment.0.connect_value_changed(f.clone());
        self.min_voltage_adjustment
            .0
            .connect_value_changed(f.clone());
        self.max_sclk_adjustment.0.connect_value_changed(f.clone());
        self.max_mclk_adjustment.0.connect_value_changed(f.clone());
        self.max_voltage_adjustment
            .0
            .connect_value_changed(f.clone());
        self.voltage_offset_adjustment.0.connect_value_changed(f);
    }

    pub fn connect_clocks_reset<F: Fn() + 'static + Clone>(&self, f: F) {
        self.reset_button.connect_clicked(move |_| f());
    }

    pub fn get_settings(&self) -> ClocksSettings {
        if self.tweaking_grid.is_visible() {
            let min_core_clock = get_adjustment_value(&self.min_sclk_adjustment);
            let min_memory_clock = get_adjustment_value(&self.min_mclk_adjustment);
            let min_voltage = get_adjustment_value(&self.min_voltage_adjustment);
            let max_core_clock = get_adjustment_value(&self.max_sclk_adjustment);
            let max_memory_clock = get_adjustment_value(&self.max_mclk_adjustment);
            let max_voltage = get_adjustment_value(&self.max_voltage_adjustment);

            let voltage_offset = if self.voltage_offset_adjustment.0.upper() == 0.0 {
                None
            } else {
                Some(self.voltage_offset_adjustment.0.value() as i32)
            };

            ClocksSettings {
                min_core_clock,
                min_memory_clock,
                min_voltage,
                max_core_clock,
                max_memory_clock,
                max_voltage,
                voltage_offset,
            }
        } else {
            ClocksSettings::default()
        }
    }

    fn set_configuration_mode(&self, advanced: bool) {
        self.advanced_togglebutton.set_active(advanced);
        self.basic_togglebutton.set_active(!advanced);

        self.min_values_grid.set_visible(advanced);
    }
}

fn extract_value_and_range(
    table: &ClocksTableGen,
    f: fn(
        &ClocksTableGen,
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

fn oc_adjustment(title: &'static str, grid: &Grid, row: i32) -> (Adjustment, Rc<AtomicBool>) {
    let label = Label::builder().label(title).halign(Align::Start).build();

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

    let popover = Popover::builder().child(&value_selector).build();
    let value_button = MenuButton::builder()
        .popover(&popover)
        .child(&value_label)
        .build();

    let changed = Rc::new(AtomicBool::new(false));

    adjustment.connect_value_changed(
        clone!(@strong value_label, @strong changed => move |adjustment| {
            changed.store(true, Ordering::SeqCst);

            let value = adjustment.value();
            value_label.set_text(&value.to_string());
        }),
    );

    adjustment.connect_changed(
        clone!(@strong label, @strong value_label, @strong scale, @strong value_button => move |adjustment| {
            let value = adjustment.value();
            value_label.set_text(&value.to_string());

            if adjustment.upper() == 0.0 {
                label.hide();
                value_label.hide();
                scale.hide();
                value_button.hide();
            } else {
                label.show();
                value_label.show();
                scale.show();
                value_button.show();
            }
        }
    ));

    grid.attach(&label, 0, row, 1, 1);
    grid.attach(&scale, 1, row, 4, 1);
    grid.attach(&value_button, 6, row, 4, 1);

    (adjustment, changed)
}

#[derive(Debug, Default)]
pub struct ClocksSettings {
    pub min_core_clock: Option<i32>,
    pub min_memory_clock: Option<i32>,
    pub min_voltage: Option<i32>,
    pub max_core_clock: Option<i32>,
    pub max_memory_clock: Option<i32>,
    pub max_voltage: Option<i32>,
    pub voltage_offset: Option<i32>,
}

fn get_adjustment_value((adjustment, changed): &(Adjustment, Rc<AtomicBool>)) -> Option<i32> {
    let changed = changed.load(Ordering::SeqCst);

    if changed {
        let value = adjustment.value();
        if value == 0.0 {
            None
        } else {
            Some(value as i32)
        }
    } else {
        None
    }
}

fn emit_changed(adjustment: &(Adjustment, Rc<AtomicBool>)) {
    adjustment.0.emit_by_name::<()>("changed", &[]);
    adjustment.1.store(false, Ordering::SeqCst);
}
