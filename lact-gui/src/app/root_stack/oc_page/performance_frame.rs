use crate::app::page_section::PageSection;
use amdgpu_sysfs::gpu_handle::{power_profile_mode::PowerProfileModesTable, PerformanceLevel};
use glib::clone;
use gtk::prelude::*;
use gtk::*;
use std::{cell::RefCell, rc::Rc, str::FromStr};

#[derive(Clone)]
pub struct PerformanceFrame {
    pub container: PageSection,
    level_drop_down: DropDown,
    mode_drop_down: DropDown,
    description_label: Label,
    manual_info_button: MenuButton,
    mode_box: Box,
    modes_table: Rc<RefCell<Option<PowerProfileModesTable>>>,
}

impl PerformanceFrame {
    pub fn new() -> Self {
        let container = PageSection::new("Performance");

        let levels_model: StringList = ["Automatic", "Highest Clocks", "Lowest Clocks", "Manual"]
            .into_iter()
            .collect();

        let level_box = Box::new(Orientation::Horizontal, 10);

        let level_drop_down = DropDown::builder()
            .model(&levels_model)
            .sensitive(false)
            .build();
        let description_label = Label::builder().halign(Align::End).hexpand(true).build();
        let perfromance_title_label = Label::builder().label("Performance level:").build();

        level_box.append(&perfromance_title_label);
        level_box.append(&description_label);
        level_box.append(&level_drop_down);

        container.append(&level_box);

        let mode_box = Box::new(Orientation::Horizontal, 10);

        let mode_drop_down = DropDown::builder()
            .sensitive(false)
            .halign(Align::End)
            .build();

        let unavailable_label = Label::new(Some(
            "Performance level has to be set to \"manual\" to use power states and modes",
        ));
        let mode_info_popover = Popover::builder().child(&unavailable_label).build();
        let manual_info_button = MenuButton::builder()
            .icon_name("dialog-information-symbolic")
            .hexpand(true)
            .halign(Align::End)
            .popover(&mode_info_popover)
            .build();

        let mode_title_label = Label::new(Some("Power level mode:"));
        mode_box.append(&mode_title_label);
        mode_box.append(&manual_info_button);
        mode_box.append(&mode_drop_down);

        container.append(&mode_box);

        let frame = Self {
            container,
            level_drop_down,
            mode_drop_down,
            description_label,
            manual_info_button,
            mode_box,
            modes_table: Rc::new(RefCell::new(None)),
        };

        frame
            .level_drop_down
            .connect_selected_notify(clone!(@strong frame => move |_| {
                frame.update_from_selection();
            }));

        frame
            .mode_drop_down
            .connect_selected_notify(clone!(@strong frame => move |_| {
                frame.update_from_selection();
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
        self.update_from_selection();
    }

    pub fn set_power_profile_modes(&self, table: Option<PowerProfileModesTable>) {
        self.mode_box.set_visible(table.is_some());

        match &table {
            Some(table) => {
                let model: StringList = table.modes.values().cloned().collect();
                let active_pos = table
                    .modes
                    .keys()
                    .position(|key| *key == table.active)
                    .expect("No active mode") as u32;

                self.mode_drop_down.set_model(Some(&model));
                self.mode_drop_down.set_selected(active_pos);

                self.mode_drop_down.show();
            }
            None => {
                self.mode_drop_down.hide();
            }
        }
        self.modes_table.replace(table);
    }

    pub fn connect_settings_changed<F: Fn() + 'static + Clone>(&self, f: F) {
        self.level_drop_down
            .connect_selected_notify(clone!(@strong f => move |_| f()));
        self.mode_drop_down.connect_selected_notify(move |_| f());
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

    pub fn get_selected_power_profile_mode(&self) -> Option<u16> {
        if self.mode_drop_down.is_sensitive() {
            self.modes_table.borrow().as_ref().map(|table| {
                let selected_index = table
                    .modes
                    .keys()
                    .nth(self.mode_drop_down.selected() as usize)
                    .expect("Selected mode out of range");
                *selected_index
            })
        } else {
            None
        }
    }

    fn update_from_selection(&self) {
        let mut enable_mode_control = false;

        let text = match self.level_drop_down.selected() {
            0 => "Automatically adjust GPU and VRAM clocks. (Default)",
            1 => "Always use the highest clockspeeds for GPU and VRAM.",
            2 => "Always use the lowest clockspeeds for GPU and VRAM.",
            3 => {
                enable_mode_control = true;
                "Manual performance control."
            }
            _ => unreachable!(),
        };
        self.description_label.set_text(text);
        self.mode_drop_down.set_sensitive(enable_mode_control);

        self.manual_info_button.set_visible(!enable_mode_control);
        self.mode_drop_down.set_hexpand(enable_mode_control);
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
