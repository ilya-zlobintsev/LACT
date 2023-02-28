use std::str::FromStr;

use crate::app::root_stack::section_box;
use glib::clone;
use gtk::prelude::*;
use gtk::*;
use lact_client::schema::PerformanceLevel;

#[derive(Clone)]
pub struct PerformanceFrame {
    pub container: Box,
    drop_down: DropDown,
    description_label: Label,
}

impl PerformanceFrame {
    pub fn new() -> Self {
        let container = section_box("Performance");

        let model: StringList = ["Automatic", "Highest Clocks", "Lower Clocks", "Manual"]
            .into_iter()
            .collect();

        let root_box = Box::new(Orientation::Horizontal, 5);

        let drop_down = DropDown::builder().model(&model).sensitive(false).build();
        let description_label = Label::new(None);

        root_box.append(&drop_down);
        root_box.append(&description_label);

        container.append(&root_box);

        let frame = Self {
            container,
            drop_down,
            description_label,
        };

        frame
            .drop_down
            .connect_selected_notify(clone!(@strong frame => move |_| {
                frame.set_description();
            }));

        frame
    }

    pub fn set_active_profile(&self, level: PerformanceLevel) {
        self.drop_down.set_sensitive(true);
        match level {
            PerformanceLevel::Auto => self.drop_down.set_selected(0),
            PerformanceLevel::High => self.drop_down.set_selected(1),
            PerformanceLevel::Low => self.drop_down.set_selected(2),
            PerformanceLevel::Manual => self.drop_down.set_selected(3),
        };
        self.set_description();
    }

    pub fn connect_power_profile_changed<F: Fn() + 'static>(&self, f: F) {
        self.drop_down.connect_selected_notify(move |_| {
            f();
        });
    }

    pub fn get_selected_performance_level(&self) -> PerformanceLevel {
        let selected_item = self.drop_down.selected_item().expect("No selected item");
        let string_object = selected_item.downcast_ref::<StringObject>().unwrap();
        PerformanceLevel::from_str(string_object.string().as_str())
            .expect("Unrecognized selected performance level")
    }

    fn set_description(&self) {
        let text = match self.drop_down.selected() {
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
