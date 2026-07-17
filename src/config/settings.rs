//! Application settings
//!
//! Global application settings including font, theme, and behavior preferences.

use serde::{Deserialize, Serialize};

/// Default scrollback lines.
const DEFAULT_SCROLLBACK: u32 = 10000;
/// Default font size.
const DEFAULT_FONT_SIZE: f64 = 12.0;

/// Application-wide settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppSettings {
    /// Terminal font family name.
    #[serde(default = "default_font_family")]
    pub font_family: String,
    /// Terminal font size in points.
    #[serde(default = "default_font_size")]
    pub font_size: f64,
    /// Active terminal color scheme name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_scheme: Option<String>,
    /// Minimize to system tray on close.
    #[serde(default)]
    pub minimize_to_tray: bool,
    /// Save and restore tab layout on exit/start.
    #[serde(default = "default_true")]
    pub save_layout: bool,
    /// Number of scrollback lines.
    #[serde(default = "default_scrollback")]
    pub scrollback_lines: u32,
    /// Auto-check for updates on startup.
    #[serde(default = "default_true")]
    pub auto_update_check: bool,
}

fn default_font_family() -> String {
    "Cascadia Code".to_string()
}

fn default_font_size() -> f64 {
    DEFAULT_FONT_SIZE
}

fn default_scrollback() -> u32 {
    DEFAULT_SCROLLBACK
}

fn default_true() -> bool {
    true
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            font_family: default_font_family(),
            font_size: default_font_size(),
            color_scheme: Some("Default Dark".to_string()),
            minimize_to_tray: false,
            save_layout: true,
            scrollback_lines: DEFAULT_SCROLLBACK,
            auto_update_check: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_default_values() {
        let settings = AppSettings::default();
        assert!(settings.font_size - 12.0 < f64::EPSILON);
        assert_eq!(settings.font_family, "Cascadia Code");
        assert_eq!(settings.scrollback_lines, 10000);
        assert!(settings.save_layout);
        assert!(settings.auto_update_check);
        assert!(!settings.minimize_to_tray);
    }

    #[test]
    fn test_settings_json_roundtrip() {
        let settings = AppSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings, deserialized);
    }

    #[test]
    fn test_settings_custom_values() {
        let settings = AppSettings {
            font_family: "Fira Code".into(),
            font_size: 14.0,
            color_scheme: None,
            minimize_to_tray: true,
            save_layout: false,
            scrollback_lines: 5000,
            auto_update_check: false,
        };
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings, deserialized);
    }

    #[test]
    fn test_settings_color_scheme_none_omitted() {
        let settings = AppSettings {
            color_scheme: None,
            ..Default::default()
        };
        let json = serde_json::to_string(&settings).unwrap();
        assert!(!json.contains("color_scheme"));
    }
}
