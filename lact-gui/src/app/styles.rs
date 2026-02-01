pub const COMBINED_CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/combined.css"));

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
