use gtk::prelude::*;
use gtk::*;
use lact_client::schema::{DeviceListEntry, SystemInfo};
use pango::EllipsizeMode;

#[derive(Clone)]
pub struct Header {
    pub container: HeaderBar,
    gpu_selector: ComboBoxText,
    switcher: StackSwitcher,
}

impl Header {
    pub fn new(system_info: &SystemInfo) -> Self {
        let container = HeaderBar::new();
        container.set_show_title_buttons(true);

        let switcher = StackSwitcher::new();
        container.set_title_widget(Some(&switcher));

        let gpu_selector = ComboBoxText::new();
        container.pack_start(&gpu_selector);

        let menu = gio::Menu::new();
        menu.append(
            Some("Show historical charts"),
            Some("app.show-graphs-window"),
        );
        menu.append(
            Some("Generate debug snapshot"),
            Some("app.generate-debug-snapshot"),
        );

        if system_info.amdgpu_overdrive_enabled == Some(true) {
            menu.append(
                Some("Disable overclocking support"),
                Some("app.disable-overdrive"),
            )
        }

        let menu_button = MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .menu_model(&menu)
            .build();
        container.pack_end(&menu_button);

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
        for (i, entry) in gpus.iter().enumerate() {
            let name = format!("{i}: {}", entry.name.unwrap_or_default());
            self.gpu_selector.append(Some(entry.id), &name);
        }

        //limits the length of gpu names in combobox
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
