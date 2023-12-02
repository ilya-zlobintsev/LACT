use crate::app::{page_section::PageSection, root_stack::action_row};
use glib::clone;
use gtk::prelude::*;
use gtk::*;
use lact_client::schema::amdgpu_sysfs::gpu_handle::{
    power_profile_mode::PowerProfileModesTable, PerformanceLevel,
};
use std::{cell::RefCell, rc::Rc, str::FromStr};

#[cfg(feature = "libadwaita")]
use adw::prelude::{ActionRowExt, ComboRowExt};

#[derive(Debug, Clone)]
pub struct PerformanceFrame {
    pub container: PageSection,

    #[cfg(feature = "libadwaita")]
    level_row: adw::ComboRow,
    #[cfg(feature = "libadwaita")]
    mode_row: adw::ComboRow,

    #[cfg(not(feature = "libadwaita"))]
    level_row: DropDown,
    #[cfg(not(feature = "libadwaita"))]
    level_subtitle: Label,
    #[cfg(not(feature = "libadwaita"))]
    mode_row: DropDown,

    modes_table: Rc<RefCell<Option<PowerProfileModesTable>>>,
}

impl PerformanceFrame {
    pub fn new() -> Self {
        let container = PageSection::new("Performance");

        let listbox = ListBox::builder()
            .css_classes(["boxed-list"])
            .selection_mode(SelectionMode::None)
            .build();

        let levels_model: StringList = ["Automatic", "Highest Clocks", "Lowest Clocks", "Manual"]
            .into_iter()
            .collect();

        #[cfg(feature = "libadwaita")]
        let level_row = {
            let row = adw::ComboRow::builder()
                .model(&levels_model)
                .title("Performance level")
                .subtitle("")
                .subtitle_lines(0)
                .sensitive(false)
                .build();
            listbox.append(&row);
            row
        };

        #[cfg(not(feature = "libadwaita"))]
        let level_subtitle;
        #[cfg(not(feature = "libadwaita"))]
        let level_row = {
            let dropdown = DropDown::builder()
                .model(&levels_model)
                .sensitive(false)
                .valign(Align::Center)
                .build();
            let row = action_row("Performance level", Some(""), &[&dropdown], None);
            level_subtitle = row
                .first_child()
                .unwrap()
                .first_child()
                .unwrap()
                .first_child()
                .unwrap()
                .next_sibling()
                .unwrap()
                .downcast::<Label>()
                .unwrap();
            listbox.append(&row);
            dropdown
        };

        let filler_model: StringList = [""].into_iter().collect();

        #[cfg(feature = "libadwaita")]
        let mode_row = {
            let row = adw::ComboRow::builder()
                .model(&filler_model)
                .title("Power level mode")
                .subtitle("Set \"Performance level\" to \"Manual\" to use power states and modes")
                .subtitle_lines(0)
                .sensitive(false)
                .build();
            listbox.append(&row);
            row
        };

        #[cfg(not(feature = "libadwaita"))]
        let mode_row = {
            let dropdown = DropDown::builder()
                .model(&filler_model)
                .sensitive(false)
                .valign(Align::Center)
                .build();
            let row = action_row(
                "Power level mode",
                Some("Set \"Performance level\" to \"Manual\" to use power states and modes"),
                &[&dropdown],
                None,
            );
            listbox.append(&row);
            dropdown
        };

        container.append(&listbox);

        let frame = Self {
            container,
            level_row,
            #[cfg(not(feature = "libadwaita"))]
            level_subtitle,
            mode_row,
            modes_table: Rc::new(RefCell::new(None)),
        };

        frame
            .level_row
            .connect_selected_notify(clone!(@strong frame => move |_| {
                frame.update_from_selection();
            }));

        frame
            .mode_row
            .connect_selected_notify(clone!(@strong frame => move |_| {
                frame.update_from_selection();
            }));

        frame
    }

    pub fn set_active_level(&self, level: PerformanceLevel) {
        self.level_row.set_sensitive(true);
        match level {
            PerformanceLevel::Auto => self.level_row.set_selected(0),
            PerformanceLevel::High => self.level_row.set_selected(1),
            PerformanceLevel::Low => self.level_row.set_selected(2),
            PerformanceLevel::Manual => self.level_row.set_selected(3),
        };
        self.update_from_selection();
    }

    pub fn set_power_profile_modes(&self, table: Option<PowerProfileModesTable>) {
        self.mode_row.set_visible(table.is_some());

        match &table {
            Some(table) => {
                let model: StringList = table.modes.values().cloned().collect();
                let active_pos = table
                    .modes
                    .keys()
                    .position(|key| *key == table.active)
                    .expect("No active mode") as u32;

                self.mode_row.set_model(Some(&model));
                self.mode_row.set_selected(active_pos);

                // set mode_row sensitivity because it gets reset to sensitive
                // after setting the model
                self.update_from_selection();

                self.mode_row.show();
            }
            None => {
                self.mode_row.hide();
            }
        }
        self.modes_table.replace(table);
    }

    pub fn connect_settings_changed<F: Fn() + 'static + Clone>(&self, f: F) {
        self.level_row
            .connect_selected_notify(clone!(@strong f => move |_| f()));
        self.mode_row.connect_selected_notify(move |_| f());
    }

    pub fn get_selected_performance_level(&self) -> PerformanceLevel {
        let selected_item = self.level_row.selected_item().expect("No selected item");
        let string_object = selected_item.downcast_ref::<StringObject>().unwrap();
        PerformanceLevel::from_str(string_object.string().as_str())
            .expect("Unrecognized selected performance level")
    }

    pub fn get_selected_power_profile_mode(&self) -> Option<u16> {
        if self.mode_row.is_sensitive() {
            self.modes_table.borrow().as_ref().map(|table| {
                let selected_index = table
                    .modes
                    .keys()
                    .nth(self.mode_row.selected() as usize)
                    .expect("Selected mode out of range");
                *selected_index
            })
        } else {
            None
        }
    }

    fn update_from_selection(&self) {
        let mut enable_mode_control = false;

        let subtitle = match self.level_row.selected() {
            0 => "Automatically adjust GPU and VRAM clocks. (Default)",
            1 => "Always use the highest clockspeeds for GPU and VRAM.",
            2 => "Always use the lowest clockspeeds for GPU and VRAM.",
            3 => {
                enable_mode_control = true;
                "Manual performance control."
            }
            _ => unreachable!(),
        };

        #[cfg(feature = "libadwaita")]
        self.level_row.set_subtitle(subtitle);

        #[cfg(not(feature = "libadwaita"))]
        self.level_subtitle.set_text(subtitle);

        self.mode_row.set_sensitive(enable_mode_control);
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
