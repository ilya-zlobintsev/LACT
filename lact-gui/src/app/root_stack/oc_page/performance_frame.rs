use crate::app::page_section::PageSection;
use amdgpu_sysfs::gpu_handle::{power_profile_mode::PowerProfileModesTable, PerformanceLevel};
use glib::clone;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{
    glib, DropDown, Label, ListBox, MenuButton, Notebook, NotebookPage, Popover, SelectionMode,
    StringObject,
};
use gtk::{prelude::*, Align, Orientation, StringList};
use std::{cell::RefCell, rc::Rc, str::FromStr};

use super::power_profile::power_profile_heuristics_grid::PowerProfileHeuristicsGrid;

type ValuesChangedCallback = Rc<dyn Fn()>;

#[derive(Clone)]
pub struct PerformanceFrame {
    pub container: PageSection,
    level_drop_down: DropDown,
    modes_listbox: ListBox,
    mode_menu_button: MenuButton,
    description_label: Label,
    manual_info_button: MenuButton,
    mode_box: gtk::Box,
    modes_table: Rc<RefCell<Option<PowerProfileModesTable>>>,
    power_mode_info_notebook: Notebook,

    values_changed_callback: Rc<RefCell<Option<ValuesChangedCallback>>>,
}

impl PerformanceFrame {
    pub fn new() -> Self {
        let container = PageSection::new("Performance");

        let levels_model: StringList = ["Automatic", "Highest Clocks", "Lowest Clocks", "Manual"]
            .into_iter()
            .collect();

        let level_box = gtk::Box::new(Orientation::Horizontal, 10);

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

        let mode_box = gtk::Box::new(Orientation::Horizontal, 10);

        let mode_menu_button = MenuButton::builder()
            .sensitive(false)
            .halign(Align::End)
            .always_show_arrow(false)
            .build();

        let modes_listbox = ListBox::builder()
            .selection_mode(SelectionMode::Single)
            .build();

        let modes_popover_content = gtk::Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(10)
            .margin_start(5)
            .margin_end(5)
            .margin_top(5)
            .margin_bottom(5)
            .build();
        modes_popover_content.append(&modes_listbox);

        let power_mode_info_notebook = Notebook::new();
        modes_popover_content.append(&power_mode_info_notebook);

        let modes_popover = Popover::builder().child(&modes_popover_content).build();
        mode_menu_button.set_popover(Some(&modes_popover));

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
        mode_box.append(&mode_menu_button);

        container.append(&mode_box);

        let frame = Self {
            container,
            level_drop_down,
            mode_menu_button,
            description_label,
            manual_info_button,
            modes_listbox,
            mode_box,
            modes_table: Rc::new(RefCell::new(None)),
            power_mode_info_notebook,
            values_changed_callback: Rc::default(),
        };

        frame.level_drop_down.connect_selected_notify(clone!(
            #[strong]
            frame,
            move |_| {
                frame.update_from_selection();
            }
        ));

        frame.modes_listbox.connect_row_selected(clone!(
            #[strong]
            frame,
            move |_, _| frame.update_from_selection()
        ));

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

        while let Some(row) = self.modes_listbox.row_at_index(0) {
            self.modes_listbox.remove(&row);
        }

        match &table {
            Some(table) => {
                for profile in table.modes.values() {
                    let profile_label = Label::builder()
                        .label(&profile.name)
                        .margin_start(5)
                        .margin_end(5)
                        .build();
                    self.modes_listbox.append(&profile_label);
                }

                let active_row = self
                    .modes_listbox
                    .row_at_index(table.active as i32)
                    .unwrap();
                self.modes_listbox.select_row(Some(&active_row));

                self.mode_menu_button.show();
            }
            None => {
                self.mode_menu_button.hide();
            }
        }
        self.modes_table.replace(table);

        self.update_from_selection();
    }

    pub fn connect_settings_changed<F: Fn() + 'static + Clone>(&self, f: F) {
        self.level_drop_down.connect_selected_notify(clone!(
            #[strong]
            f,
            move |_| f()
        ));
        self.modes_listbox.connect_row_selected(clone!(
            #[strong(rename_to = modes_table)]
            self.modes_table,
            #[strong]
            f,
            move |_, row| {
                let modes_table = modes_table.borrow();

                if let Some(row) = row {
                    if let Some(table) = modes_table.as_ref() {
                        if row.index() != table.active as i32 {
                            f();
                        }
                    }
                }
            }
        ));

        *self.values_changed_callback.borrow_mut() = Some(Rc::new(f));
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
        if self.mode_menu_button.is_sensitive() {
            self.modes_listbox
                .selected_row()
                .map(|row| row.index() as u16)
        } else {
            None
        }
    }

    pub fn get_power_profile_mode_custom_heuristics(&self) -> Vec<Vec<Option<i32>>> {
        let modes_table = self.modes_table.borrow();
        if let Some(table) = modes_table.as_ref() {
            if let Some(row) = self.modes_listbox.selected_row() {
                let active_index = row.index() as u16;
                if let Some(active_profile) = table.modes.get(&active_index) {
                    if active_profile.is_custom() {
                        let mut components = vec![];

                        for page in self
                            .power_mode_info_notebook
                            .pages()
                            .iter::<NotebookPage>()
                            .flatten()
                        {
                            let values_grid = page
                                .child()
                                .downcast::<PowerProfileHeuristicsGrid>()
                                .unwrap();
                            components.push(values_grid.imp().component.borrow().values.clone());
                        }

                        return components;
                    }
                }
            }
        }

        vec![]
    }

    fn update_from_selection(&self) {
        self.power_mode_info_notebook.set_visible(false);

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

        self.manual_info_button.set_visible(!enable_mode_control);

        self.mode_menu_button.set_sensitive(enable_mode_control);
        self.mode_menu_button.set_hexpand(enable_mode_control);

        let values_changed_callback = self.values_changed_callback.borrow();

        let modes_table = self.modes_table.borrow();
        if let Some(table) = modes_table.as_ref() {
            if let Some(row) = self.modes_listbox.selected_row() {
                let active_index = row.index() as u16;
                if let Some(active_profile) = table.modes.get(&active_index) {
                    self.mode_menu_button.set_label(&active_profile.name);

                    self.power_mode_info_notebook.set_visible(true);

                    // Save current page to be restored after being refilled
                    let current_page = self.power_mode_info_notebook.current_page();
                    // Remove pages
                    while self.power_mode_info_notebook.n_pages() != 0 {
                        self.power_mode_info_notebook.remove_page(None);
                    }

                    for (i, component) in active_profile.components.iter().enumerate() {
                        let values_grid = PowerProfileHeuristicsGrid::new();
                        values_grid.set_component(component, table);

                        let title = component.clock_type.as_deref().unwrap_or("All");
                        let title_label = Label::builder()
                            .label(title)
                            .margin_start(5)
                            .margin_end(5)
                            .build();
                        self.power_mode_info_notebook
                            .append_page(&values_grid, Some(&title_label));

                        if let Some(f) = &*values_changed_callback {
                            values_grid.connect_component_values_changed(clone!(
                                #[strong]
                                f,
                                #[strong(rename_to = modes_table)]
                                self.modes_table,
                                #[strong]
                                values_grid,
                                move || {
                                    let mut modes_table = modes_table.borrow_mut();
                                    if let Some(current_table) = &mut *modes_table {
                                        let changed_component =
                                            values_grid.imp().component.borrow().clone();
                                        current_table
                                            .modes
                                            .get_mut(&active_index)
                                            .unwrap()
                                            .components[i] = changed_component;
                                    }

                                    f();
                                }
                            ));
                        }
                    }

                    self.power_mode_info_notebook
                        .set_show_tabs(active_profile.components.len() > 1);

                    let is_custom = active_profile.is_custom();
                    for page in self
                        .power_mode_info_notebook
                        .pages()
                        .iter::<NotebookPage>()
                        .flatten()
                    {
                        page.child().set_sensitive(is_custom);
                    }

                    // Restore selected page
                    if current_page.is_some() {
                        self.power_mode_info_notebook.set_current_page(current_page);
                    }
                }
            }
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
