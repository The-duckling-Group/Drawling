#![allow(dead_code)]
use drawling_core::Color;

/// Visual Theme adhering to KDE Plasma Breeze Dark & Modern Windows 11 Dark aesthetic
#[derive(Debug, Clone)]
pub struct AppTheme {
    pub name: &'static str,
    pub background: Color,
    pub surface: Color,
    pub surface_hover: Color,
    pub surface_active: Color,
    pub border: Color,
    pub accent: Color,
    pub accent_hover: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub danger: Color,
    pub success: Color,
}

impl AppTheme {
    pub fn breeze_dark() -> Self {
        Self {
            name: "KDE Breeze Dark",
            background: Color::from_hex("#1b1e20").unwrap(),
            surface: Color::from_hex("#232629").unwrap(),
            surface_hover: Color::from_hex("#2d3136").unwrap(),
            surface_active: Color::from_hex("#353a3f").unwrap(),
            border: Color::from_hex("#3b4045").unwrap(),
            accent: Color::from_hex("#3daee9").unwrap(), // Plasma Blue
            accent_hover: Color::from_hex("#59bcf3").unwrap(),
            text_primary: Color::from_hex("#fcfcfc").unwrap(),
            text_secondary: Color::from_hex("#bdc3c7").unwrap(),
            text_muted: Color::from_hex("#7f8c8d").unwrap(),
            danger: Color::from_hex("#ed1515").unwrap(),
            success: Color::from_hex("#11d116").unwrap(),
        }
    }

    pub fn modern_windows_dark() -> Self {
        Self {
            name: "Modern Windows Dark",
            background: Color::from_hex("#202020").unwrap(),
            surface: Color::from_hex("#2c2c2c").unwrap(),
            surface_hover: Color::from_hex("#383838").unwrap(),
            surface_active: Color::from_hex("#444444").unwrap(),
            border: Color::from_hex("#404040").unwrap(),
            accent: Color::from_hex("#60cdff").unwrap(), // Win11 Mica Accent
            accent_hover: Color::from_hex("#7ad5ff").unwrap(),
            text_primary: Color::from_hex("#ffffff").unwrap(),
            text_secondary: Color::from_hex("#d1d1d1").unwrap(),
            text_muted: Color::from_hex("#8a8a8a").unwrap(),
            danger: Color::from_hex("#ff6b6b").unwrap(),
            success: Color::from_hex("#6bff8a").unwrap(),
        }
    }
}

impl Default for AppTheme {
    fn default() -> Self {
        Self::breeze_dark()
    }
}
