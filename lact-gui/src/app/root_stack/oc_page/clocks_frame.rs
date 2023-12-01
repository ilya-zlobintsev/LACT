use crate::app::page_section::PageSection;
use crate::app::root_stack::action_row;
use glib::clone;
use gtk::prelude::*;
use gtk::*;
use lact_client::schema::amdgpu_sysfs;
use lact_client::schema::amdgpu_sysfs::gpu_handle::overdrive::{ClocksTable, ClocksTableGen};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::debug;

#[cfg(feature = "libadwaita")]
use libadwaita::prelude::ActionRowExt;

const VOLTAGE_OFFSET_RANGE: f64 = 250.0;

// The AtomicBool stores if the value was changed
#[derive(Debug, Clone)]
pub struct ClocksFrame {
    pub container: PageSection,
    max_values_box: Box,
    heading_listbox: ListBox,

    #[cfg(feature = "libadwaita")]
    advanced_switch_row: libadwaita::SwitchRow,

    #[cfg(not(feature = "libadwaita"))]
    advanced_switch_row: Switch,

    min_values_box: Box,
    min_sclk_adjustment: (Adjustment, Rc<AtomicBool>),
    min_mclk_adjustment: (Adjustment, Rc<AtomicBool>),
    min_voltage_adjustment: (Adjustment, Rc<AtomicBool>),
    max_sclk_adjustment: (Adjustment, Rc<AtomicBool>),
    max_mclk_adjustment: (Adjustment, Rc<AtomicBool>),
    max_voltage_adjustment: (Adjustment, Rc<AtomicBool>),
    voltage_offset_adjustment: (Adjustment, Rc<AtomicBool>),
    reset_button: Button,
    clocks_data_unavailable_label: Label,
}

impl ClocksFrame {
    pub fn new() -> Self {
        let container = PageSection::new("Clockspeed and voltage");

        let heading_listbox = ListBox::builder()
            .css_classes(["boxed-list"])
            .selection_mode(SelectionMode::None)
            .build();

        let warning_row = action_row(
            "Warning!",
            Some("Changing these values may lead to system instability and potentially damage your hardware!"),
            &Vec::<&Widget>::new(),
            Some(&vec!["warning"]));

        heading_listbox.append(&warning_row);

        #[cfg(feature = "libadwaita")]
        let advanced_switch_row = {
            let row = libadwaita::SwitchRow::builder()
                .title("Advanced mode")
                .active(false)
                .build();
            heading_listbox.append(&row);
            row
        };

        #[cfg(not(feature = "libadwaita"))]
        let advanced_switch_row = {
            let switch = Switch::builder().active(false).build();
            let row = action_row("Advanced mode", None, &[&switch], None);

            heading_listbox.append(&row);
            switch
        };

        container.append(&heading_listbox);

        let min_values_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(12)
            .build();
        min_values_box.append(
            &Label::builder()
                .label("Minimum Values")
                .xalign(0.0)
                .css_classes(["title-4"])
                .build(),
        );
        let min_values_listbox = ListBox::builder()
            .css_classes(["boxed-list"])
            .selection_mode(SelectionMode::None)
            .build();
        min_values_box.append(&min_values_listbox);

        let min_sclk_adjustment = oc_adjustment("Minimum GPU Clock (MHz)", &min_values_listbox);
        let min_mclk_adjustment = oc_adjustment("Minimum VRAM Clock (MHz)", &min_values_listbox);
        let min_voltage_adjustment = oc_adjustment("Minimum GPU voltage (mV)", &min_values_listbox);

        container.append(&min_values_box);

        let max_values_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(12)
            .build();
        max_values_box.append(
            &Label::builder()
                .label("Maximum Values")
                .xalign(0.0)
                .css_classes(["title-4"])
                .build(),
        );
        let max_values_listbox = ListBox::builder()
            .css_classes(["boxed-list"])
            .selection_mode(SelectionMode::None)
            .build();
        max_values_box.append(&max_values_listbox);

        let max_sclk_adjustment = oc_adjustment("Maximum GPU Clock (MHz)", &max_values_listbox);
        let max_voltage_adjustment = oc_adjustment("Maximum GPU voltage (mV)", &max_values_listbox);
        let max_mclk_adjustment = oc_adjustment("Maximum VRAM Clock (MHz)", &max_values_listbox);
        let voltage_offset_adjustment =
            oc_adjustment("GPU voltage offset (mV)", &max_values_listbox);

        let reset_button = Button::builder()
            .label("Reset")
            .child(
                &Label::builder()
                    .label("Reset")
                    .margin_start(12)
                    .margin_end(12)
                    .build(),
            )
            .valign(Align::Center)
            .halign(Align::Center)
            .css_classes(["destructive-action", "circular"])
            .build();

        let reset_row = action_row(
            "Reset values",
            Some("Warning: this will reset all clock and voltage settings to their default values"),
            &[&reset_button],
            None,
        );

        max_values_listbox.append(&reset_row);

        let clocks_data_unavailable_label = Label::builder()
            .label("No clocks data available")
            .css_classes(["error"])
            .halign(Align::Start)
            .build();

        container.append(&max_values_box);
        container.append(&clocks_data_unavailable_label);

        let frame = Self {
            container,
            max_values_box,
            heading_listbox,
            advanced_switch_row,
            min_sclk_adjustment,
            min_mclk_adjustment,
            min_voltage_adjustment,
            max_sclk_adjustment,
            max_mclk_adjustment,
            max_voltage_adjustment,
            reset_button,
            clocks_data_unavailable_label,
            voltage_offset_adjustment,
            min_values_box,
        };

        frame.set_configuration_mode(false);

        frame
            .advanced_switch_row
            .connect_active_notify(clone!(@strong frame => move |row| {
                frame.set_configuration_mode(row.is_active());
            }));

        frame
    }

    pub fn set_table(&self, table: ClocksTableGen) -> anyhow::Result<()> {
        debug!("using clocks table {table:?}");

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
                self.voltage_offset_adjustment
                    .0
                    .set_lower(VOLTAGE_OFFSET_RANGE * -1.0);
                self.voltage_offset_adjustment
                    .0
                    .set_upper(VOLTAGE_OFFSET_RANGE);
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
        self.max_values_box.show();
        self.heading_listbox.show();
        self.clocks_data_unavailable_label.hide();
    }

    pub fn hide(&self) {
        self.max_values_box.hide();
        self.heading_listbox.hide();
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
        if self.max_values_box.is_visible() {
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
        self.min_values_box.set_visible(advanced);
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

fn oc_adjustment(title: &'static str, listbox: &ListBox) -> (Adjustment, Rc<AtomicBool>) {
    let adjustment = Adjustment::new(0.0, 0.0, 0.0, 1.0, 10.0, 0.0);

    #[cfg(feature = "libadwaita")]
    let value_selector = libadwaita::SpinRow::builder()
        .title(title)
        .adjustment(&adjustment)
        .build();

    #[cfg(not(feature = "libadwaita"))]
    let value_selector = {
        let spin_btn = SpinButton::builder()
            .adjustment(&adjustment)
            .valign(Align::Center)
            .build();
        let row = action_row(title, None, &[&spin_btn], None);
        row.set_child(Some(&spin_btn));
        spin_btn
    };

    let changed = Rc::new(AtomicBool::new(false));

    adjustment.connect_value_changed(clone!(@strong changed => move |_| {
        changed.store(true, Ordering::SeqCst);
    }));

    adjustment.connect_changed(clone!(@strong value_selector => move |adjustment| {
            value_selector.set_sensitive(adjustment.upper() == 0.0);
        }
    ));

    listbox.append(&value_selector);

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
