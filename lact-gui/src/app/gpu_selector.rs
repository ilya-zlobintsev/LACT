use gtk::prelude::*;
use gtk::*;
use lact_client::schema::DeviceListEntry;
use pango::EllipsizeMode;

#[derive(Debug, Clone)]
pub struct GpuSelector {
    pub dropdown: ComboBoxText,
}

impl GpuSelector {
    pub fn new() -> Self {
        // WARN: gtk::ComboBoxText is deprecated, use gtk::DropDown instead
        let dropdown = ComboBoxText::new();

        dropdown
            .first_child()
            .unwrap()
            .first_child()
            .unwrap()
            .add_css_class("flat");

        Self { dropdown }
    }

    pub fn set_devices(&self, gpus: &[DeviceListEntry<'_>]) {
        for (i, entry) in gpus.iter().enumerate() {
            let name = format!("{i}: {}", entry.name.unwrap_or_default());
            self.dropdown.append(Some(entry.id), &name);
        }

        // limits the length of gpu names in combobox
        for cell in self.dropdown.cells() {
            cell.set_property("width-chars", 10);
            cell.set_property("ellipsize", EllipsizeMode::End);
        }

        self.dropdown.set_active(Some(0));
    }

    pub fn connect_gpu_selection_changed<F: Fn(String) + 'static>(&self, f: F) {
        self.dropdown.connect_changed(move |gpu_selector| {
            if let Some(selected_id) = gpu_selector.active_id() {
                f(selected_id.to_string());
            }
        });
    }
}
