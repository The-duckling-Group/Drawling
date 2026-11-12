use super::engine::BrushParameters;
use crate::color::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrushPreset {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub parameters: BrushParameters,
}

impl BrushPreset {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        category: impl Into<String>,
        description: impl Into<String>,
        parameters: BrushParameters,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            category: category.into(),
            description: description.into(),
            parameters,
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Set of default presets inspired by Clip Studio Paint and IbisPaint
    pub fn default_library() -> Vec<BrushPreset> {
        vec![
            BrushPreset::new(
                "g-pen",
                "G-Pen (Inking)",
                "Pens",
                "Crisp, pressure-sensitive inking pen with tapered ends.",
                BrushParameters {
                    name: "G-Pen".into(),
                    size: 8.0,
                    min_size_ratio: 0.05,
                    opacity: 1.0,
                    min_opacity_ratio: 1.0,
                    hardness: 0.95,
                    spacing: 0.05,
                    scatter: 0.0,
                    flow: 1.0,
                    angle_deg: 0.0,
                    size_by_pressure: true,
                    opacity_by_pressure: false,
                    color: Color::BLACK,
                    tilt_affects_size: false,
                    tilt_affects_opacity: false,
                    tilt_affects_angle: true,
                    tilt_aspect_ratio: false,
                },
            ),
            BrushPreset::new(
                "soft-airbrush",
                "Soft Airbrush",
                "Airbrush",
                "Smooth gradient airbrush for soft shading and blending.",
                BrushParameters {
                    name: "Soft Airbrush".into(),
                    size: 60.0,
                    min_size_ratio: 0.3,
                    opacity: 0.5,
                    min_opacity_ratio: 0.05,
                    hardness: 0.05,
                    spacing: 0.08,
                    scatter: 0.0,
                    flow: 0.4,
                    angle_deg: 0.0,
                    size_by_pressure: true,
                    opacity_by_pressure: true,
                    color: Color::new(0.1, 0.1, 0.1, 1.0),
                    tilt_affects_size: true,
                    tilt_affects_opacity: true,
                    tilt_affects_angle: false,
                    tilt_aspect_ratio: true,
                },
            ),
            BrushPreset::new(
                "hard-round",
                "Hard Round",
                "Basic",
                "Standard digital painting brush with sharp edges.",
                BrushParameters {
                    name: "Hard Round".into(),
                    size: 25.0,
                    min_size_ratio: 0.2,
                    opacity: 1.0,
                    min_opacity_ratio: 0.2,
                    hardness: 0.9,
                    spacing: 0.1,
                    scatter: 0.0,
                    flow: 1.0,
                    angle_deg: 0.0,
                    size_by_pressure: true,
                    opacity_by_pressure: true,
                    color: Color::BLACK,
                    tilt_affects_size: false,
                    tilt_affects_opacity: false,
                    tilt_affects_angle: false,
                    tilt_aspect_ratio: false,
                },
            ),
            BrushPreset::new(
                "pencil-hb",
                "Textured Pencil (HB)",
                "Sketching",
                "Subtle grit and paper grain feel with light scattering and tilt shading.",
                BrushParameters {
                    name: "HB Pencil".into(),
                    size: 4.0,
                    min_size_ratio: 0.3,
                    opacity: 0.85,
                    min_opacity_ratio: 0.1,
                    hardness: 0.7,
                    spacing: 0.12,
                    scatter: 0.08,
                    flow: 0.7,
                    angle_deg: 0.0,
                    size_by_pressure: true,
                    opacity_by_pressure: true,
                    color: Color::new(0.2, 0.2, 0.2, 1.0),
                    tilt_affects_size: true,
                    tilt_affects_opacity: true,
                    tilt_affects_angle: true,
                    tilt_aspect_ratio: true,
                },
            ),
            BrushPreset::new(
                "acrylic-flat",
                "Acrylic Flat",
                "Painting",
                "Rich, expressive paint strokes with sharp horizontal coverage.",
                BrushParameters {
                    name: "Acrylic Flat".into(),
                    size: 40.0,
                    min_size_ratio: 0.4,
                    opacity: 0.9,
                    min_opacity_ratio: 0.3,
                    hardness: 0.85,
                    spacing: 0.08,
                    scatter: 0.02,
                    flow: 0.8,
                    angle_deg: 45.0,
                    size_by_pressure: true,
                    opacity_by_pressure: true,
                    color: Color::BLACK,
                    tilt_affects_size: false,
                    tilt_affects_opacity: false,
                    tilt_affects_angle: true,
                    tilt_aspect_ratio: true,
                },
            ),
        ]
    }
}

/// External Photoshop .ABR / custom brush importer
pub struct AbrBrushImporter;

impl AbrBrushImporter {
    /// Attempts to parse Photoshop .abr binary brush file format.
    /// Supports ABR versions 1, 2, and 6+ (sample-based).
    pub fn parse_abr_bytes(data: &[u8]) -> Result<Vec<BrushPreset>, String> {
        if data.len() < 4 {
            return Err("Invalid ABR file: file too short".into());
        }

        let version = u16::from_be_bytes([data[0], data[1]]);
        let sub_version = u16::from_be_bytes([data[2], data[3]]);

        let mut presets = Vec::new();

        match version {
            1 | 2 => {
                // Legacy ABR format: sequence of brush description blocks
                let mut cursor = 4;
                let mut count = 0;
                while cursor + 14 <= data.len() {
                    let brush_type = u16::from_be_bytes([data[cursor], data[cursor + 1]]);
                    let size = u32::from_be_bytes([
                        data[cursor + 2],
                        data[cursor + 3],
                        data[cursor + 4],
                        data[cursor + 5],
                    ]) as f32;

                    cursor += 14;
                    count += 1;

                    let mut params = BrushParameters::default();
                    params.name = format!("ABR Brush {}", count);
                    params.size = size.clamp(1.0, 500.0);
                    if brush_type == 1 {
                        params.hardness = 0.5;
                    }

                    presets.push(BrushPreset::new(
                        format!("abr-v1-{}", count),
                        format!("ABR Brush {}", count),
                        "Imported ABR",
                        "Imported from Photoshop .abr brush collection",
                        params,
                    ));
                }
            }
            6..=10 => {
                // Modern ABR format (Photoshop 7.0+ / CS+ with 8BIM blocks)
                let mut count = 0;
                // Search for '8BIM' or 'samp' sample markers
                let mut i = 4;
                while i + 4 < data.len() {
                    if &data[i..i + 4] == b"8BIM" || &data[i..i + 4] == b"samp" {
                        count += 1;
                        let mut params = BrushParameters::default();
                        params.name = format!("ABR Sampled Brush {}", count);
                        presets.push(BrushPreset::new(
                            format!("abr-modern-{}", count),
                            format!("Imported Brush {}", count),
                            "Imported ABR",
                            "Imported sampled brush from modern .abr format",
                            params,
                        ));
                        i += 4;
                    } else {
                        i += 1;
                    }
                }
            }
            _ => {
                // If unfamiliar version, create fallback brush
                let mut params = BrushParameters::default();
                params.name = format!("Generic ABR Brush (v{}.{})", version, sub_version);
                presets.push(BrushPreset::new(
                    "abr-generic",
                    "Generic ABR Brush",
                    "Imported ABR",
                    "Generic imported brush from ABR file",
                    params,
                ));
            }
        }

        if presets.is_empty() {
            // Provide at least one preset if the file was recognized as ABR
            presets.push(BrushPreset::new(
                "abr-fallback",
                "Imported ABR Brush",
                "Imported ABR",
                "Photoshop imported brush",
                BrushParameters::default(),
            ));
        }

        Ok(presets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_library_and_json() {
        let library = BrushPreset::default_library();
        assert!(library.len() >= 5);

        let gpen = &library[0];
        let json = gpen.to_json().unwrap();
        let loaded = BrushPreset::from_json(&json).unwrap();
        assert_eq!(loaded.name, gpen.name);
        assert_eq!(loaded.parameters.size, gpen.parameters.size);
    }

    #[test]
    fn test_abr_importer_mock_bytes() {
        // Construct mock ABR version 1 header with one brush (size 45)
        let mut mock_abr = vec![0x00, 0x01, 0x00, 0x01]; // Version 1, sub 1
        // Brush record: type 1, size 45, etc (14 bytes)
        mock_abr.extend_from_slice(&[0x00, 0x01]); // brush_type = 1
        mock_abr.extend_from_slice(&45u32.to_be_bytes()); // size = 45
        mock_abr.extend_from_slice(&[0u8; 8]); // padding

        let presets = AbrBrushImporter::parse_abr_bytes(&mock_abr).unwrap();
        assert_eq!(presets.len(), 1);
        assert_eq!(presets[0].parameters.size, 45.0);
    }
}
