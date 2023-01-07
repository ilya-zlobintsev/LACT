use gtk::prelude::*;
use gtk::*;
use lact_client::schema::PerformanceLevel;

use super::section_box;

#[derive(Clone)]
pub struct PerformanceLevelFrame {
    pub container: Box,
    combo_box: ComboBoxText,
}

impl PerformanceLevelFrame {
    pub fn new() -> Self {
        let container = section_box("Performance level");

        let root_box = Box::new(Orientation::Horizontal, 5);

        let combo_box = ComboBoxText::new();

        combo_box.append(None, "Automatic");
        combo_box.append(None, "Highest clocks");
        combo_box.append(None, "Lowest clocks");

        root_box.append(&combo_box);

        let description_label = Label::new(Some("A description is supposed to be here"));

        root_box.append(&description_label);

        {
            let description_label = description_label.clone();
            combo_box.connect_changed(move |combobox| match combobox.active().unwrap() {
                0 => description_label
                    .set_text("Automatically adjust GPU and VRAM clocks. (Default)"),
                1 => description_label
                    .set_text("Always use the highest clockspeeds for GPU and VRAM."),
                2 => description_label
                    .set_text("Always use the lowest clockspeeds for GPU and VRAM."),
                _ => unreachable!(),
            });
        }

        container.append(&root_box);
        Self {
            container,
            combo_box,
        }
    }

    pub fn set_active_profile(&self, level: PerformanceLevel) {
        match level {
            PerformanceLevel::Auto => self.combo_box.set_active(Some(0)),
            PerformanceLevel::High => self.combo_box.set_active(Some(1)),
            PerformanceLevel::Low => self.combo_box.set_active(Some(2)),
            PerformanceLevel::Manual => todo!(),
        };
    }

    pub fn connect_power_profile_changed<F: Fn() + 'static>(&self, f: F) {
        self.combo_box.connect_changed(move |_| {
            f();
        });
    }

    pub fn get_selected_performance_level(&self) -> PerformanceLevel {
        match self.combo_box.active().unwrap() {
            0 => PerformanceLevel::Auto,
            1 => PerformanceLevel::High,
            2 => PerformanceLevel::Low,
            _ => unreachable!(),
        }
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
