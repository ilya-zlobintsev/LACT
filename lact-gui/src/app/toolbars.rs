use gtk::prelude::*;
use gtk::*;
use lact_client::schema::DeviceListEntry;
use pango::EllipsizeMode;

#[derive(Clone)]
pub struct Toolbars {
    pub headerbar: libadwaita::HeaderBar,
    pub gpu_selector: ComboBoxText,
    pub title: libadwaita::WindowTitle,
}

impl Toolbars {
    pub fn new() -> Self {
        let title = libadwaita::WindowTitle::builder()
            .title("Information")
            .build();

        let headerbar = libadwaita::HeaderBar::builder()
            .title_widget(&title)
            .show_title(true)
            .build();

        // WARN: gtk::ComboBoxText is deprecated, use gtk::DropDown instead
        let gpu_selector = ComboBoxText::builder()
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        Self {
            headerbar,
            gpu_selector,
            title,
        }
    }

    pub fn set_devices(&self, gpus: &[DeviceListEntry<'_>]) {
        for (i, entry) in gpus.iter().enumerate() {
            let name = format!("{i}: {}", entry.name.unwrap_or_default());
            self.gpu_selector.append(Some(entry.id), &name);
        }

        // limits the length of gpu names in combobox
        for cell in self.gpu_selector.cells() {
            cell.set_property("width-chars", 10);
            cell.set_property("ellipsize", EllipsizeMode::End);
        }

        self.gpu_selector.set_active(Some(0));
    }

    pub fn connect_gpu_selection_changed<F: Fn(String) + 'static>(&self, f: F) {
        self.gpu_selector.connect_changed(move |gpu_selector| {
            if let Some(selected_id) = gpu_selector.active_id() {
                f(selected_id.to_string());
            }
        });
    }
}
