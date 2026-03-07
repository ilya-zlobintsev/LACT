use gtk::{
    gio::{self, prelude::SettingsExt},
    style_context_add_provider_for_display, style_context_remove_provider_for_display,
};
use i18n_embed_fl::fl;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

use crate::I18N;

pub const COMBINED_CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/combined.css"));

macro_rules! include_theme_str {
    ($file:literal) => {
        include_str!(concat!(
            concat!(env!("CARGO_MANIFEST_DIR"), "/themes/",),
            $file
        ))
    };
}

const BREEZE_DARK_CSS: &str = include_theme_str!("breeze-dark.css");
const BREEZE_LIGHT_CSS: &str = include_theme_str!("breeze-light.css");

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
pub enum AppTheme {
    #[default]
    Automatic,
    Adwaita,
    Breeze,
}

impl AppTheme {
    pub const ALL: &[Self] = &[Self::Automatic, Self::Adwaita, Self::Breeze];

    pub fn display_name(&self) -> String {
        match self {
            AppTheme::Automatic => fl!(I18N, "theme-auto"),
            AppTheme::Adwaita => "Adwaita".to_owned(),
            AppTheme::Breeze => "Breeze".to_owned(),
        }
    }

    pub fn from_idx(idx: u32) -> Option<Self> {
        match idx {
            0 => Some(Self::Automatic),
            1 => Some(Self::Adwaita),
            2 => Some(Self::Breeze),
            _ => None,
        }
    }
}

thread_local! {
    static EXISTING_STYLE_PROVIDER: RefCell<Option<gtk::CssProvider>> = const { RefCell::new(None) };
}

pub fn apply_theme(theme: AppTheme) -> anyhow::Result<()> {
    let display = gtk::gdk::Display::default().unwrap();

    if let Some(existing_style_provider) = EXISTING_STYLE_PROVIDER.take() {
        style_context_remove_provider_for_display(&display, &existing_style_provider);
    }

    let theme_css = match theme {
        AppTheme::Automatic => {
            let settings = gio::Settings::new("org.gnome.desktop.interface");
            let system_theme = settings.string("gtk-theme").to_ascii_lowercase();

            if matches!(
                system_theme.as_str(),
                "breeze" | "breeze-light" | "breeze-dark"
            ) {
                Some(breeze_css())
            } else {
                None
            }
        }
        AppTheme::Adwaita => None,
        AppTheme::Breeze => Some(breeze_css()),
    };

    if let Some(css) = theme_css {
        let provider = gtk::CssProvider::new();
        #[allow(deprecated)]
        provider.load_from_data(css);

        style_context_add_provider_for_display(&display, &provider, 900);

        EXISTING_STYLE_PROVIDER.set(Some(provider));
    }

    Ok(())
}

fn breeze_css() -> &'static str {
    let settings = gio::Settings::new("org.gnome.desktop.interface");
    match settings.string("color-scheme").as_str() {
        "prefer-dark" => BREEZE_DARK_CSS,
        "prefer-light" => BREEZE_LIGHT_CSS,
        _ => BREEZE_LIGHT_CSS,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_css_is_loaded() {
        assert!(!COMBINED_CSS.is_empty(), "Combined CSS should not be empty");
        assert!(
            COMBINED_CSS.contains(".app"),
            "Combined CSS should contain the .app class"
        );
    }
}
