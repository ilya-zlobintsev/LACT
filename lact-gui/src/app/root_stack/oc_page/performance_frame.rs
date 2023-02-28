use std::str::FromStr;

use crate::app::root_stack::section_box;
use glib::clone;
use gtk::prelude::*;
use gtk::*;
use lact_client::schema::{PerformanceLevel, PowerProfileModesTable};

#[derive(Clone)]
pub struct PerformanceFrame {
    pub container: Box,
    level_drop_down: DropDown,
    mode_drop_down: DropDown,
    description_label: Label,
}

impl PerformanceFrame {
    pub fn new() -> Self {
        let container = section_box("Performance");

        let levels_model: StringList = ["Automatic", "Highest Clocks", "Lower Clocks", "Manual"]
            .into_iter()
            .collect();

        let level_box = Box::new(Orientation::Horizontal, 5);

        let level_drop_down = DropDown::builder()
            .model(&levels_model)
            .sensitive(false)
            .build();
        let description_label = Label::new(None);

        level_box.append(&level_drop_down);
        level_box.append(&description_label);

        let mode_box = Box::new(Orientation::Horizontal, 5);

        let mode_drop_down = DropDown::builder().sensitive(false).build();

        mode_box.append(&mode_drop_down);

        container.append(&level_box);
        container.append(&mode_box);

        let frame = Self {
            container,
            level_drop_down,
            mode_drop_down,
            description_label,
        };

        frame
            .level_drop_down
            .connect_selected_notify(clone!(@strong frame => move |_| {
                frame.set_description();
            }));

        frame
    }

    pub fn set_active_level(&self, level: PerformanceLevel) {
        self.level_drop_down.set_sensitive(true);
        match level {
            PerformanceLevel::Auto => self.level_drop_down.set_selected(0),
            PerformanceLevel::High => self.level_drop_down.set_selected(1),
            PerformanceLevel::Low => self.level_drop_down.set_selected(2),
            PerformanceLevel::Manual => self.level_drop_down.set_selected(3),
        };
        self.set_description();
    }

    pub fn set_power_profile_modes(&self, table: Option<PowerProfileModesTable>) {
        match table {
            Some(table) => {
                let model: StringList = table.modes.into_iter().map(|mode| mode.name).collect();
                self.mode_drop_down.set_model(Some(&model));
                self.mode_drop_down.set_selected(table.active as u32);

                self.mode_drop_down.show();
            }
            None => {
                self.mode_drop_down.hide();
            }
        }
    }

    pub fn connect_power_profile_changed<F: Fn() + 'static>(&self, f: F) {
        self.level_drop_down.connect_selected_notify(move |_| {
            f();
        });
    }

    pub fn get_selected_performance_level(&self) -> PerformanceLevel {
        let selected_item = self
            .level_drop_down
            .selected_item()
            .expect("No selected item");
        let string_object = selected_item.downcast_ref::<StringObject>().unwrap();
        PerformanceLevel::from_str(string_object.string().as_str())
            .expect("Unrecognized selected performance level")
    }

    fn set_description(&self) {
        let text = match self.level_drop_down.selected() {
            0 => "Automatically adjust GPU and VRAM clocks. (Default)",
            1 => "Always use the highest clockspeeds for GPU and VRAM.",
            2 => "Always use the lowest clockspeeds for GPU and VRAM.",
            3 => "Manual performance control.",
            _ => unreachable!(),
        };
        self.description_label.set_text(text);
    }

    pub fn show(&self) {
        self.container.set_visible(true);
    }

    pub fn hide(&self) {
        self.container.set_visible(false);
    }

    pub fn get_visibility(&self) -> bool {
        self.container.get_visible()
    }
}
