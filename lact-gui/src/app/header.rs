use super::{AppMsg, DebugSnapshot, DisableOverdrive, DumpVBios, ResetConfig, ShowGraphsWindow};
use gtk::prelude::*;
use gtk::*;
use lact_client::schema::DeviceListEntry;
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, SimpleComponent,
};
use relm4_components::simple_combo_box::SimpleComboBox;

pub struct Header {
    gpu_selector: Controller<SimpleComboBox<DeviceListEntry>>,
}

#[relm4::component(pub)]
impl SimpleComponent for Header {
    type Init = (Vec<DeviceListEntry>, gtk::Stack);
    type Input = ();
    type Output = AppMsg;

    view! {
        gtk::HeaderBar {
            set_show_title_buttons: true,

            #[wrap(Some)]
            set_title_widget = &StackSwitcher {
                set_stack: Some(&stack),
            },

            #[local_ref]
            pack_start = gpu_selector -> ComboBoxText,

            pack_end = &gtk::MenuButton {
                set_icon_name: "open-menu-symbolic",
                set_menu_model: Some(&app_menu),
            }
        }
    }

    menu! {
        app_menu: {
            section! {
                "Show historical charts" => ShowGraphsWindow,
            },
            section! {
                "Generate debug snapshot" => DebugSnapshot,
                "Dump VBIOS" => DumpVBios,
            } ,
            section! {
                "Disable overclocking support" => DisableOverdrive,
                "Reset all configuration" => ResetConfig,
            }
        }
    }

    fn init(
        (variants, stack): Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let gpu_selector = SimpleComboBox::builder()
            .launch(SimpleComboBox {
                variants,
                active_index: Some(0),
            })
            .forward(sender.output_sender(), |_| AppMsg::ReloadData);

        // limits the length of gpu names in combobox
        for cell in gpu_selector.widget().cells() {
            cell.set_property("width-chars", 10);
            cell.set_property("ellipsize", pango::EllipsizeMode::End);
        }

        let model = Self { gpu_selector };

        let gpu_selector = model.gpu_selector.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

impl Header {
    pub fn selected_gpu_id(&self) -> Option<String> {
        self.gpu_selector
            .model()
            .get_active_elem()
            .map(|model| model.id.clone())
    }
}

/*#[derive(Clone)]
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
        menu.append(Some("Dump VBIOS"), Some("app.dump-vbios"));
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

        menu.append(Some("Reset all configuration"), Some("app.reset-config"));

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
}*/
