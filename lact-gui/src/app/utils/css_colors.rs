use gtk::prelude::*;
use relm4::view;
use std::cell::RefCell;

thread_local! {
    static COLOR_PROBES: RefCell<Option<CssColorProbes>> = const { RefCell::new(None) };
}

struct CssColorProbes {
    theme_base_color: gtk::Label,
    theme_text_color: gtk::Label,
    borders: gtk::Label,
    unfocused_borders: gtk::Label,
    theme_unfocused_fg_color: gtk::Label,
    success_color: gtk::Label,
    accent_bg_color: gtk::Label,
    error_color: gtk::Label,
    warning_color: gtk::Label,
}

pub struct CssColors {
    pub theme_base_color: gtk::gdk::RGBA,
    pub theme_text_color: gtk::gdk::RGBA,
    pub borders: gtk::gdk::RGBA,
    pub unfocused_borders: gtk::gdk::RGBA,
    pub theme_unfocused_fg_color: gtk::gdk::RGBA,
    pub success_color: gtk::gdk::RGBA,
    pub accent_bg_color: gtk::gdk::RGBA,
    pub error_color: gtk::gdk::RGBA,
    pub warning_color: gtk::gdk::RGBA,
}

pub fn init_probes() -> gtk::Box {
    view! {
        probes = gtk::Box::builder()
            .accessible_role(gtk::AccessibleRole::Presentation)
            .build() {
            set_opacity: 0.0,
            set_can_target: false,
            set_focusable: false,

            #[name = "theme_base_color"]
            gtk::Label {
                add_css_class: "color-probe-theme-base-color",
            },

            #[name = "theme_text_color"]
            gtk::Label {
                add_css_class: "color-probe-theme-text-color",
            },

            #[name = "borders"]
            gtk::Label {
                add_css_class: "color-probe-borders",
            },

            #[name = "unfocused_borders"]
            gtk::Label {
                add_css_class: "color-probe-unfocused-borders",
            },

            #[name = "theme_unfocused_fg_color"]
            gtk::Label {
                add_css_class: "color-probe-theme-unfocused-fg-color",
            },

            #[name = "success_color"]
            gtk::Label {
                add_css_class: "color-probe-success-color",
            },

            #[name = "accent_bg_color"]
            gtk::Label {
                add_css_class: "color-probe-accent-bg-color",
            },

            #[name = "error_color"]
            gtk::Label {
                add_css_class: "color-probe-error-color",
            },

            #[name = "warning_color"]
            gtk::Label {
                add_css_class: "color-probe-warning-color",
            },
        }
    }

    COLOR_PROBES.with(|probes| {
        probes.replace(Some(CssColorProbes {
            theme_base_color,
            theme_text_color,
            borders,
            unfocused_borders,
            theme_unfocused_fg_color,
            success_color,
            accent_bg_color,
            error_color,
            warning_color,
        }));
    });

    probes
}

pub fn current() -> Option<CssColors> {
    COLOR_PROBES.with(|probes| {
        let probes = probes.borrow();
        let probes = probes.as_ref()?;

        Some(CssColors {
            theme_base_color: probes.theme_base_color.color(),
            theme_text_color: probes.theme_text_color.color(),
            borders: probes.borders.color(),
            unfocused_borders: probes.unfocused_borders.color(),
            theme_unfocused_fg_color: probes.theme_unfocused_fg_color.color(),
            success_color: probes.success_color.color(),
            accent_bg_color: probes.accent_bg_color.color(),
            error_color: probes.error_color.color(),
            warning_color: probes.warning_color.color(),
        })
    })
}
