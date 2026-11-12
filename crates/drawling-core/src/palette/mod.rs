use crate::color::Color;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColorPalette {
    pub name: String,
    pub colors: Vec<Color>,
}

impl ColorPalette {
    pub fn new(name: impl Into<String>, colors: Vec<Color>) -> Self {
        Self {
            name: name.into(),
            colors,
        }
    }

    /// Manga & Inking screentone palette (Black, 10%, 20%, 30%, 50%, 70%, White)
    pub fn manga_screentone() -> Self {
        Self::new(
            "Manga Inking & Tone",
            vec![
                Color::from_rgba8(0, 0, 0, 255),
                Color::from_rgba8(26, 26, 26, 255),
                Color::from_rgba8(51, 51, 51, 255),
                Color::from_rgba8(102, 102, 102, 255),
                Color::from_rgba8(153, 153, 153, 255),
                Color::from_rgba8(204, 204, 204, 255),
                Color::from_rgba8(255, 255, 255, 255),
            ],
        )
    }

    /// Character illustration skin tone palette
    pub fn skin_tones() -> Self {
        Self::new(
            "Anime & Portrait Skin Tones",
            vec![
                Color::from_hex("#FFF0E5").unwrap(),
                Color::from_hex("#FAD9C5").unwrap(),
                Color::from_hex("#E8B99D").unwrap(),
                Color::from_hex("#D69F7E").unwrap(),
                Color::from_hex("#A66B49").unwrap(),
                Color::from_hex("#6E4028").unwrap(),
                Color::from_hex("#4A2818").unwrap(),
            ],
        )
    }

    /// Soft pastel anime illustration palette
    pub fn pastel_anime() -> Self {
        Self::new(
            "Pastel Anime",
            vec![
                Color::from_hex("#FFB7B2").unwrap(), // Sakura Pink
                Color::from_hex("#FFDAC1").unwrap(), // Soft Peach
                Color::from_hex("#E2F0CB").unwrap(), // Mint Cream
                Color::from_hex("#B5EAD7").unwrap(), // Pastel Aqua
                Color::from_hex("#C7CEEA").unwrap(), // Periwinkle Blue
                Color::from_hex("#E0BBE4").unwrap(), // Lavender
            ],
        )
    }

    /// Cyberpunk & Neon synthwave palette
    pub fn neon_cyberpunk() -> Self {
        Self::new(
            "Neon Cyberpunk",
            vec![
                Color::from_hex("#00F0FF").unwrap(), // Electric Cyan
                Color::from_hex("#FF0055").unwrap(), // Neon Crimson
                Color::from_hex("#FFE600").unwrap(), // Acid Yellow
                Color::from_hex("#7B00FF").unwrap(), // Cyber Violet
                Color::from_hex("#00FF66").unwrap(), // Matrix Green
                Color::from_hex("#120024").unwrap(), // Deep Night Void
            ],
        )
    }

    /// Traditional oil pigments
    pub fn classic_oil() -> Self {
        Self::new(
            "Classic Oil & Gouache",
            vec![
                Color::from_hex("#8A3324").unwrap(), // Burnt Sienna
                Color::from_hex("#CC7722").unwrap(), // Ochre
                Color::from_hex("#120A8F").unwrap(), // Ultramarine
                Color::from_hex("#004225").unwrap(), // British Racing Green
                Color::from_hex("#E30022").unwrap(), // Cadmium Red
                Color::from_hex("#FFFDD0").unwrap(), // Cream White
            ],
        )
    }

    /// Export colors as vector of hex strings
    pub fn to_hex_list(&self) -> Vec<String> {
        self.colors.iter().map(|c| c.to_hex()).collect()
    }

    /// Import colors from hex strings
    pub fn from_hex_list(name: impl Into<String>, hexes: &[&str]) -> Self {
        let colors = hexes.iter().filter_map(|h| Color::from_hex(h)).collect();
        Self::new(name, colors)
    }
}

/// Recent Colors buffer (stores last N unique picked/used colors)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentColors {
    pub max_size: usize,
    colors: VecDeque<Color>,
}

impl RecentColors {
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size: max_size.max(1),
            colors: VecDeque::with_capacity(max_size),
        }
    }

    pub fn push(&mut self, color: Color) {
        // Remove duplicate if already present
        if let Some(pos) = self.colors.iter().position(|&c| c == color) {
            self.colors.remove(pos);
        }

        self.colors.push_front(color);

        if self.colors.len() > self.max_size {
            self.colors.pop_back();
        }
    }

    pub fn get_colors(&self) -> Vec<Color> {
        self.colors.iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palettes_and_hex_roundtrip() {
        let neon = ColorPalette::neon_cyberpunk();
        assert_eq!(neon.colors.len(), 6);

        let hexes = neon.to_hex_list();
        assert_eq!(hexes[0], "#00F0FF");

        let reimported = ColorPalette::from_hex_list("Reimported", &["#00F0FF", "#FF0055"]);
        assert_eq!(reimported.colors.len(), 2);
    }

    #[test]
    fn test_recent_colors_buffer() {
        let mut recents = RecentColors::new(3);
        recents.push(Color::RED);
        recents.push(Color::GREEN);
        recents.push(Color::BLUE);
        assert_eq!(recents.get_colors().len(), 3);

        // Push duplicate RED - moves to front
        recents.push(Color::RED);
        let list = recents.get_colors();
        assert_eq!(list[0], Color::RED);
        assert_eq!(list.len(), 3);

        // Push new color - drops oldest
        recents.push(Color::WHITE);
        assert_eq!(recents.get_colors().len(), 3);
        assert_eq!(recents.get_colors()[0], Color::WHITE);
    }
}
