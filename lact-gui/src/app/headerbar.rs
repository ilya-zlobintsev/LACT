use super::{apply_box::ApplyBox, gpu_selector::GpuSelector};
use gtk::{
    gio::ActionEntry,
    glib::{self, clone, IsA},
    prelude::{ActionMapExtManual, GtkWindowExt},
};

#[derive(Debug, Clone)]
pub struct Headerbar {
    #[cfg(feature = "adw")]
    pub container: adw::HeaderBar,

    #[cfg(not(feature = "adw"))]
    pub container: gtk::HeaderBar,

    pub gpu_selector: GpuSelector,
    pub apply_box: ApplyBox,
}

impl Headerbar {
    pub fn new(app: &impl IsA<gtk::Application>, root_win: &gtk::ApplicationWindow) -> Self {
        #[cfg(feature = "adw")]
        let container = adw::HeaderBar::builder().show_title(false).build();

        #[cfg(not(feature = "adw"))]
        let container = gtk::HeaderBar::builder()
            .title_widget(&gtk::Label::new(None))
            .build();

        let gpu_selector = GpuSelector::new();
        let apply_box = ApplyBox::new();

        #[cfg(not(feature = "adw"))]
        let about_dialog = gtk::AboutDialog::builder()
            .hide_on_close(true)
            .modal(true)
            .transient_for(root_win)
            .program_name("LACT")
            .icon_name("io.github.lact-linux")
            .version(std::env!("CARGO_PKG_VERSION"))
            .license_type(gtk::License::MitX11)
            .copyright("The LACT Developers")
            .authors(
                std::env!("CARGO_PKG_AUTHORS")
                    .split(':')
                    .collect::<Vec<&str>>(),
            )
            .build();

        #[cfg(feature = "adw")]
        let about_dialog = adw::AboutWindow::builder()
            .hide_on_close(true)
            .modal(true)
            .transient_for(root_win)
            .application_name("LACT")
            .application_icon("io.github.lact-linux")
            .version(std::env!("CARGO_PKG_VERSION"))
            .license_type(gtk::License::MitX11)
            .copyright("The LACT Developers")
            .developers(
                std::env!("CARGO_PKG_AUTHORS")
                    .split(':')
                    .collect::<Vec<&str>>(),
            )
            .website("https://github.com/ilya-zlobintsev/LACT")
            .issue_url("https://github.com/ilya-zlobintsev/LACT/issues")
            .build();

        let menu = gtk::gio::Menu::new();
        menu.append_item(&gtk::gio::MenuItem::new(
            Some("About LACT"),
            Some("win.about"),
        ));

        root_win.add_action_entries([ActionEntry::builder("about")
            .activate(clone!(@weak about_dialog => move |_, _, _| {
                about_dialog.present();
            }))
            .build()]);

        container.pack_start(&gpu_selector.dropdown);
        container.pack_end(
            &gtk::MenuButton::builder()
                .icon_name("open-menu-symbolic")
                .menu_model(&menu)
                .build(),
        );
        container.pack_end(&apply_box.container);

        Self {
            container,
            gpu_selector,
            apply_box,
        }
    }
}
