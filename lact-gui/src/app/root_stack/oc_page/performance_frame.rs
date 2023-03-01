use crate::app::root_stack::section_box;
use glib::clone;
use gtk::prelude::*;
use gtk::*;
use lact_client::schema::{PerformanceLevel, PowerProfileModesTable};
use std::{cell::RefCell, rc::Rc, str::FromStr};

#[derive(Clone)]
pub struct PerformanceFrame {
    pub container: Box,
    level_drop_down: DropDown,
    mode_drop_down: DropDown,
    description_label: Label,
    mode_info_popover: Popover,
    modes_table: Rc<RefCell<Option<PowerProfileModesTable>>>,
}

impl PerformanceFrame {
    pub fn new() -> Self {
        let container = section_box("Performance");

        let grid = Grid::builder().row_spacing(5).column_spacing(10).build();

        let levels_model: StringList = ["Automatic", "Highest Clocks", "Lower Clocks", "Manual"]
            .into_iter()
            .collect();

        let level_drop_down = DropDown::builder()
            .model(&levels_model)
            .sensitive(false)
            .build();
        let description_label = Label::builder().halign(Align::End).hexpand(true).build();
        let perfromance_title_label = Label::builder().label("Performance level:").build();

        grid.attach(&perfromance_title_label, 0, 0, 1, 1);
        grid.attach(&description_label, 1, 0, 1, 1);
        grid.attach(&level_drop_down, 2, 0, 1, 1);

        let mode_drop_down = DropDown::builder().sensitive(false).build();
        let mode_info_popover = Popover::new();
        let mode_info_button = MenuButton::builder()
            .icon_name("info-symbolic")
            .hexpand(true)
            .halign(Align::End)
            .popover(&mode_info_popover)
            .build();

        let mode_title_label = Label::new(Some("Power level mode:"));
        grid.attach(&mode_title_label, 0, 1, 1, 1);
        grid.attach(&mode_info_button, 1, 1, 1, 1);
        grid.attach(&mode_drop_down, 2, 1, 1, 1);

        container.append(&grid);

        let frame = Self {
            container,
            level_drop_down,
            mode_drop_down,
            description_label,
            mode_info_popover,
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
        match &table {
            Some(table) => {
                let model: StringList = table.modes.iter().map(|mode| mode.name.clone()).collect();
                self.mode_drop_down.set_model(Some(&model));
                self.mode_drop_down.set_selected(table.active as u32);

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

    pub fn get_selected_power_profile_mode(&self) -> Option<usize> {
        if self.mode_drop_down.is_sensitive() {
            Some(self.mode_drop_down.selected() as usize)
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

        if enable_mode_control {
            let active_mode = self.mode_drop_down.selected();
            let table = self.modes_table.borrow();

            if let Some(table) = table.as_ref() {
                let vbox = Box::new(Orientation::Vertical, 5);
                let active_mode = &table.modes[active_mode as usize];

                for heuristic in &table.available_heuristics {
                    let value = active_mode
                        .heuristics
                        .get(heuristic)
                        .and_then(|value| value.as_deref())
                        .unwrap_or("-");

                    let label = Label::new(Some(&format!("{heuristic}: {value}")));
                    vbox.append(&label);
                }

                self.mode_info_popover.set_child(Some(&vbox));
            } else {
                let label = Label::new(Some("(No description)"));
                self.mode_info_popover.set_child(Some(&label));
            }
        } else {
            let label = Label::new(Some(
                "Performance level has to be set to \"manual\" to use power profile modes",
            ));
            self.mode_info_popover.set_child(Some(&label));
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
