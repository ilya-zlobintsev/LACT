use std::{collections::HashMap, env};

use gtk::prelude::{ComboBoxExtManual, ObjectExt};
use gtk::*;
use pango::EllipsizeMode;

pub struct Header {
    pub container: HeaderBar,
    gpu_selector: ComboBoxText,
    switcher: StackSwitcher,
    apply_button: Button,
}

impl Header {
    pub fn new() -> Self {
        let container = HeaderBar::new();
        
        container.set_custom_title(Some(&Grid::new())); // Bad workaround to hide the title

        if env::var("XDG_CURRENT_DESKTOP") == Ok("GNOME".to_string()) {
            container.set_show_close_button(true);
        }

        let gpu_selector = ComboBoxText::new();
        container.pack_start(&gpu_selector);

        let switcher = StackSwitcher::new();
        container.pack_start(&switcher);

        let apply_button = Button::new();
        apply_button.set_label("Apply");
        apply_button.set_sensitive(false);
        
        container.pack_start(&apply_button);

        Header {
            container,
            gpu_selector,
            switcher,
            apply_button,
        }
    }

    pub fn set_switcher_stack(&self, stack: &Stack) {
        self.switcher.set_stack(Some(stack));
    }

    pub fn set_gpus(&self, gpus: HashMap<u32, Option<String>>) {
        for (id, name) in &gpus {
            self.gpu_selector
                .append(Some(&id.to_string()), &name.clone().unwrap_or_default());
        }

        //limits the length of gpu names in combobox
        for cell in self.gpu_selector.get_cells() {
            cell.set_property("width-chars", &10).unwrap();
            cell.set_property("ellipsize", &EllipsizeMode::End).unwrap();
        }

        self.gpu_selector.set_active(Some(0));
    }

    pub fn connect_gpu_selection_changed<F: Fn(u32) + 'static>(&self, f: F) {
        self.gpu_selector.connect_changed(move |gpu_selector| {
            let selected_id = gpu_selector.get_active_id().unwrap();
            f(selected_id.parse().unwrap());
        });
    }

    pub fn connect_apply_button_clicked<F: Fn() + 'static>(&self, f: F) {
        self.apply_button.connect_clicked(move |_| {
            f();
        });
    }
    
    pub fn set_apply_button_sensitive(&self) {
        self.apply_button.set_sensitive(true);
    }
}
