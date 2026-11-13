use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaterialCategory {
    Hintergruende,
    Muster,
    Posen3D,
    Texturen,
    Stempel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialItem {
    pub id: String,
    pub name: String,
    pub category: MaterialCategory,
    pub description: String,
    pub tags: Vec<String>,
}

pub struct MaterialLibrary;

impl MaterialLibrary {
    pub fn default_catalog() -> Vec<MaterialItem> {
        vec![
            // Hintergründe
            MaterialItem {
                id: "bg-anime-sky".into(),
                name: "Anime Sommerhimmel (Cumulus)".into(),
                category: MaterialCategory::Hintergruende,
                description: "Malerischer Anime-Himmel mit dramatischen Wolkentürmen.".into(),
                tags: vec!["Himmel".into(), "Wolken".into(), "Anime".into()],
            },
            MaterialItem {
                id: "bg-manga-classroom".into(),
                name: "Manga Klassenzimmer (Lineart)".into(),
                category: MaterialCategory::Hintergruende,
                description: "Saubere 2-Punkt-Perspektive eines Schulraums mit Tischen und Fenstern.".into(),
                tags: vec!["Schule".into(), "Innenraum".into(), "Lineart".into()],
            },
            MaterialItem {
                id: "bg-cyberpunk-city".into(),
                name: "Cyberpunk Häuserschlucht".into(),
                category: MaterialCategory::Hintergruende,
                description: "Futuristische Metropole mit Neonschildern und Perspektivflucht.".into(),
                tags: vec!["Stadt".into(), "Sci-Fi".into(), "Nacht".into()],
            },

            // Muster
            MaterialItem {
                id: "pat-manga-screentone-40".into(),
                name: "Manga Screentone 40% LPI 60".into(),
                category: MaterialCategory::Muster,
                description: "Klassisches Punktraster für Manga-Schattierung.".into(),
                tags: vec!["Raster".into(), "Schattierung".into(), "Punkte".into()],
            },
            MaterialItem {
                id: "pat-crosshatch-fine".into(),
                name: "Feine Kreuzschraffur".into(),
                category: MaterialCategory::Muster,
                description: "Traditionelle Stift-Kreuzschraffur für dramatische Schatten.".into(),
                tags: vec!["Schraffur".into(), "Schatten".into()],
            },

            // 3D Posenmodelle
            MaterialItem {
                id: "pose-standing-hero".into(),
                name: "3D Körpermodell: Heroische Standpose".into(),
                category: MaterialCategory::Posen3D,
                description: "Standard Anime-Proportionen (1:7.5) mit leichter Drehung als Zeichenreferenz.".into(),
                tags: vec!["3D".into(), "Figur".into(), "Anatomie".into(), "Stehen".into()],
            },
            MaterialItem {
                id: "pose-action-jump".into(),
                name: "3D Körpermodell: Dynamischer Sprung".into(),
                category: MaterialCategory::Posen3D,
                description: "Dramatische Verkürzung und Flugphase für Shōnen-Kampfszenen.".into(),
                tags: vec!["3D".into(), "Action".into(), "Kampf".into(), "Sprung".into()],
            },
            MaterialItem {
                id: "pose-hands-gestures".into(),
                name: "3D Handmodell: Griff- & Zeigegesten".into(),
                category: MaterialCategory::Posen3D,
                description: "Präzise Fingerglieder und Daumenhaltung zur Vermeidung von Zeichenfehlern.".into(),
                tags: vec!["3D".into(), "Hand".into(), "Referenz".into()],
            },

            // Texturen
            MaterialItem {
                id: "tex-watercolor-paper".into(),
                name: "Aquarellpapier rau (Cold Press)".into(),
                category: MaterialCategory::Texturen,
                description: "Feine Körnung und Faserstruktur für authentischen traditionellen Look.".into(),
                tags: vec!["Papier".into(), "Aquarell".into(), "Korn".into()],
            },

            // Stempel
            MaterialItem {
                id: "stamp-speedlines-radial".into(),
                name: "Speedlines (Radiale Fokuslinien)".into(),
                category: MaterialCategory::Stempel,
                description: "Dynamische Effektlinien für Aufprall, Schock und hohe Geschwindigkeit.".into(),
                tags: vec!["Effekt".into(), "Speedlines".into(), "Action".into()],
            },
            MaterialItem {
                id: "stamp-sakura-petals".into(),
                name: "Sakura Kirschblütenblätter".into(),
                category: MaterialCategory::Stempel,
                description: "Wirbelnde Kirschblüten für Frühlings- und Romantikszenen.".into(),
                tags: vec!["Blüten".into(), "Natur".into(), "Shōjo".into()],
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_library_catalog() {
        let catalog = MaterialLibrary::default_catalog();
        assert!(!catalog.is_empty());
        assert!(catalog.iter().any(|m| m.category == MaterialCategory::Posen3D));
        assert!(catalog.iter().any(|m| m.category == MaterialCategory::Hintergruende));
    }
}
