pub const COMBINED_CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/combined.css"));

pub const THEME_BREEZE_DARK: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/themes/breeze-dark.css"
));
pub const THEME_BREEZE_LIGHT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/themes/breeze-light.css"
));

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
