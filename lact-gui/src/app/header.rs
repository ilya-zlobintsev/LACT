use gtk::prelude::*;
use gtk::*;
use lact_client::schema::DeviceListEntry;
use pango::EllipsizeMode;

#[derive(Clone)]
pub struct Header {
    pub container: HeaderBar,
    gpu_selector: ComboBoxText,
    switcher: StackSwitcher,
}

impl Header {
    pub fn new() -> Self {
        let container = HeaderBar::new();

        // TODO Check if this is this still needed
        container.set_title_widget(Some(&Grid::new())); // Bad workaround to hide the title

        container.set_show_title_buttons(true);

        let gpu_selector = ComboBoxText::new();
        container.pack_start(&gpu_selector);

        let switcher = StackSwitcher::new();
        container.pack_start(&switcher);

        Header {
            container,
            gpu_selector,
            switcher,
        }
    }

    pub fn set_switcher_stack(&self, stack: &Stack) {
        self.switcher.set_stack(Some(stack));
    }

    pub fn set_devices(&self, gpus: &[DeviceListEntry<'_>]) {
        for entry in gpus {
            self.gpu_selector
                .append(Some(entry.id), entry.name.unwrap_or_default());
        }

        //limits the length of gpu names in combobox
        for cell in self.gpu_selector.cells() {
            cell.set_property("width-chars", &10);
            cell.set_property("ellipsize", &EllipsizeMode::End);
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
