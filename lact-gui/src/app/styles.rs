use anyhow::Context;
use gtk::{
    STYLE_PROVIDER_PRIORITY_APPLICATION, style_context_add_provider_for_display,
    style_context_remove_provider_for_display,
};
use std::cell::RefCell;

pub const COMBINED_CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/combined.css"));

macro_rules! define_themes {
    ($($name:literal: $file:literal,)*) => {
        [
            $(
                ($name, include_str!(concat!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/themes/",
                ), $file))),
            )*
        ]
    };
}

const THEMES: &[(&str, &str)] = &define_themes! {
    "Breeze Dark": "breeze-dark.css",
    "Breeze Light": "breeze-light.css",
};

pub fn theme_names() -> Vec<&'static str> {
    THEMES.iter().map(|(name, _)| *name).collect()
}

thread_local! {
    static EXISTING_STYLE_PROVIDER: RefCell<Option<gtk::CssProvider>> = const { RefCell::new(None) };
}

pub fn apply_theme(name: Option<&str>) -> anyhow::Result<()> {
    let display = gtk::gdk::Display::default().unwrap();

    if let Some(existing_style_provider) = EXISTING_STYLE_PROVIDER.take() {
        style_context_remove_provider_for_display(&display, &existing_style_provider);
    }

    if let Some(name) = name {
        let (_, css) = THEMES
            .iter()
            .find(|(theme, _)| *theme == name)
            .with_context(|| format!("Theme '{name}' does not exist"))?;

        let provider = gtk::CssProvider::new();
        #[allow(deprecated)]
        provider.load_from_data(css);

        style_context_add_provider_for_display(
            &display,
            &provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        EXISTING_STYLE_PROVIDER.set(Some(provider));
    }

    Ok(())
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
