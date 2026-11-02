use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppColorScheme {
    #[default]
    Auto,
    Light,
    Dark,
}

impl From<AppColorScheme> for adw::ColorScheme {
    fn from(value: AppColorScheme) -> Self {
        match value {
            AppColorScheme::Auto => adw::ColorScheme::Default,
            AppColorScheme::Light => adw::ColorScheme::ForceLight,
            AppColorScheme::Dark => adw::ColorScheme::ForceDark,
        }
    }
}

impl AppColorScheme {
    pub fn apply(&self) {
        adw::StyleManager::default().set_color_scheme((*self).into());
    }
}
