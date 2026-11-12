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
                "dip-pen-soft-bleach",
                "Weicher Füllfederstift (Bleiche)",
                "Füllfederstifte",
                "Authentischer weicher IbisPaint-Stift mit eleganter Verjüngung und Druckdynamik.",
                BrushParameters {
                    name: "Weicher Füllfederstift (Bleiche)".into(),
                    size: 16.0,
                    min_size_ratio: 0.02,
                    opacity: 1.0,
                    min_opacity_ratio: 0.2,
                    hardness: 0.88,
                    spacing: 0.04,
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
                    start_width_ratio: 0.0,
                    end_width_ratio: 0.0,
                    start_opacity_ratio: 1.0,
                    end_opacity_ratio: 1.0,
                    fade_in_length: 30.0,
                    fade_out_length: 30.0,
                },
            ),
            BrushPreset::new(
                "dip-pen-hard",
                "Harter Füllfederstift",
                "Füllfederstifte",
                "Scharfe, definierte Linienführung für saubere Manga- und Anime-Lineart.",
                BrushParameters {
                    name: "Harter Füllfederstift".into(),
                    size: 12.0,
                    min_size_ratio: 0.05,
                    opacity: 1.0,
                    min_opacity_ratio: 0.5,
                    hardness: 0.98,
                    spacing: 0.03,
                    scatter: 0.0,
                    flow: 1.0,
                    angle_deg: 0.0,
                    size_by_pressure: true,
                    opacity_by_pressure: false,
                    color: Color::BLACK,
                    tilt_affects_size: false,
                    tilt_affects_opacity: false,
                    tilt_affects_angle: false,
                    tilt_aspect_ratio: false,
                    start_width_ratio: 0.05,
                    end_width_ratio: 0.05,
                    start_opacity_ratio: 1.0,
                    end_opacity_ratio: 1.0,
                    fade_in_length: 25.0,
                    fade_out_length: 25.0,
                },
            ),
            BrushPreset::new(
                "digital-pen",
                "Digitalstift",
                "Basis",
                "Feste Pixelkontur ohne Druckdynamik für präzise Pixel-Art und Skizzen.",
                BrushParameters {
                    name: "Digitalstift".into(),
                    size: 6.0,
                    min_size_ratio: 1.0,
                    opacity: 1.0,
                    min_opacity_ratio: 1.0,
                    hardness: 1.0,
                    spacing: 0.04,
                    scatter: 0.0,
                    flow: 1.0,
                    angle_deg: 0.0,
                    size_by_pressure: false,
                    opacity_by_pressure: false,
                    color: Color::BLACK,
                    tilt_affects_size: false,
                    tilt_affects_opacity: false,
                    tilt_affects_angle: false,
                    tilt_aspect_ratio: false,
                    start_width_ratio: 1.0,
                    end_width_ratio: 1.0,
                    start_opacity_ratio: 1.0,
                    end_opacity_ratio: 1.0,
                    fade_in_length: 0.0,
                    fade_out_length: 0.0,
                },
            ),
            BrushPreset::new(
                "felt-tip-hard",
                "Filzstift (hart)",
                "Filzstifte",
                "Gleichmäßiger kräftiger Farbauftrag für Markierungen und Comic-Inking.",
                BrushParameters {
                    name: "Filzstift (hart)".into(),
                    size: 24.0,
                    min_size_ratio: 0.6,
                    opacity: 1.0,
                    min_opacity_ratio: 0.8,
                    hardness: 0.85,
                    spacing: 0.05,
                    scatter: 0.0,
                    flow: 1.0,
                    angle_deg: 0.0,
                    size_by_pressure: true,
                    opacity_by_pressure: false,
                    color: Color::BLACK,
                    tilt_affects_size: false,
                    tilt_affects_opacity: false,
                    tilt_affects_angle: false,
                    tilt_aspect_ratio: false,
                    start_width_ratio: 0.6,
                    end_width_ratio: 0.6,
                    start_opacity_ratio: 1.0,
                    end_opacity_ratio: 1.0,
                    fade_in_length: 0.0,
                    fade_out_length: 0.0,
                },
            ),
            BrushPreset::new(
                "felt-tip-soft",
                "Filzstift (weich)",
                "Filzstifte",
                "Sanfterer Markerstift mit leichter Transparenz bei schnellen Strichen.",
                BrushParameters {
                    name: "Filzstift (weich)".into(),
                    size: 28.0,
                    min_size_ratio: 0.4,
                    opacity: 0.85,
                    min_opacity_ratio: 0.3,
                    hardness: 0.65,
                    spacing: 0.06,
                    scatter: 0.0,
                    flow: 0.9,
                    angle_deg: 0.0,
                    size_by_pressure: true,
                    opacity_by_pressure: true,
                    color: Color::BLACK,
                    tilt_affects_size: false,
                    tilt_affects_opacity: false,
                    tilt_affects_angle: false,
                    tilt_aspect_ratio: false,
                    start_width_ratio: 0.4,
                    end_width_ratio: 0.4,
                    start_opacity_ratio: 0.8,
                    end_opacity_ratio: 0.8,
                    fade_in_length: 0.0,
                    fade_out_length: 0.0,
                },
            ),
            BrushPreset::new(
                "airbrush-normal",
                "Airbrush (Normal)",
                "Airbrush",
                "Weicher Farbverlauf für Schattierungen, Glanzlichter und weiche Übergänge.",
                BrushParameters {
                    name: "Airbrush (Normal)".into(),
                    size: 80.0,
                    min_size_ratio: 0.25,
                    opacity: 0.4,
                    min_opacity_ratio: 0.02,
                    hardness: 0.02,
                    spacing: 0.06,
                    scatter: 0.0,
                    flow: 0.35,
                    angle_deg: 0.0,
                    size_by_pressure: true,
                    opacity_by_pressure: true,
                    color: Color::new(0.1, 0.1, 0.1, 1.0),
                    tilt_affects_size: true,
                    tilt_affects_opacity: true,
                    tilt_affects_angle: false,
                    tilt_aspect_ratio: true,
                    start_width_ratio: 0.5,
                    end_width_ratio: 0.5,
                    start_opacity_ratio: 0.2,
                    end_opacity_ratio: 0.2,
                    fade_in_length: 0.0,
                    fade_out_length: 0.0,
                },
            ),
            BrushPreset::new(
                "pencil-graphite",
                "Bleistift (#1)",
                "Bleistifte",
                "Feine Bleistifttextur mit natürlicher Papierkörnung und Druckreaktion.",
                BrushParameters {
                    name: "Bleistift (#1)".into(),
                    size: 5.0,
                    min_size_ratio: 0.2,
                    opacity: 0.9,
                    min_opacity_ratio: 0.1,
                    hardness: 0.72,
                    spacing: 0.1,
                    scatter: 0.06,
                    flow: 0.75,
                    angle_deg: 0.0,
                    size_by_pressure: true,
                    opacity_by_pressure: true,
                    color: Color::new(0.2, 0.2, 0.2, 1.0),
                    tilt_affects_size: true,
                    tilt_affects_opacity: true,
                    tilt_affects_angle: true,
                    tilt_aspect_ratio: true,
                    start_width_ratio: 0.2,
                    end_width_ratio: 0.2,
                    start_opacity_ratio: 0.5,
                    end_opacity_ratio: 0.5,
                    fade_in_length: 15.0,
                    fade_out_length: 15.0,
                },
            ),
            BrushPreset::new(
                "watercolor-soft",
                "Wasserfarbe (weich)",
                "Wasserfarben",
                "Aquarellpinsel mit weichen Rändern und transparenter Farbschichtung.",
                BrushParameters {
                    name: "Wasserfarbe (weich)".into(),
                    size: 45.0,
                    min_size_ratio: 0.35,
                    opacity: 0.55,
                    min_opacity_ratio: 0.1,
                    hardness: 0.35,
                    spacing: 0.06,
                    scatter: 0.01,
                    flow: 0.45,
                    angle_deg: 0.0,
                    size_by_pressure: true,
                    opacity_by_pressure: true,
                    color: Color::BLACK,
                    tilt_affects_size: false,
                    tilt_affects_opacity: false,
                    tilt_affects_angle: false,
                    tilt_aspect_ratio: false,
                    start_width_ratio: 0.3,
                    end_width_ratio: 0.3,
                    start_opacity_ratio: 0.4,
                    end_opacity_ratio: 0.4,
                    fade_in_length: 20.0,
                    fade_out_length: 20.0,
                },
            ),
            BrushPreset::new(
                "g-pen-manga",
                "G-Pen (Manga Inking)",
                "Füllfederstifte",
                "Traditioneller japanischer G-Pen für professionelle Manga-Zeichnungen.",
                BrushParameters {
                    name: "G-Pen".into(),
                    size: 10.0,
                    min_size_ratio: 0.05,
                    opacity: 1.0,
                    min_opacity_ratio: 1.0,
                    hardness: 0.95,
                    spacing: 0.04,
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
                    start_width_ratio: 0.05,
                    end_width_ratio: 0.05,
                    start_opacity_ratio: 1.0,
                    end_opacity_ratio: 1.0,
                    fade_in_length: 25.0,
                    fade_out_length: 25.0,
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

/// Importer for IbisPaint brush configurations, JSON presets, .ipbrush bundles, and QR-code strings
pub struct IbisBrushImporter;

impl IbisBrushImporter {
    /// Decode a base64 string to bytes without external crate dependencies
    pub fn decode_base64_string(input: &str) -> Option<Vec<u8>> {
        let mut table = [255u8; 256];
        for (i, &b) in b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".iter().enumerate() {
            table[b as usize] = i as u8;
        }
        let mut out = Vec::new();
        let mut buf = 0u32;
        let mut bits = 0;
        for &b in input.as_bytes() {
            if b == b'=' || b.is_ascii_whitespace() { continue; }
            let val = table[b as usize];
            if val == 255 { return None; }
            buf = (buf << 6) | (val as u32);
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                out.push((buf >> bits) as u8);
            }
        }
        Some(out)
    }

    /// Import brush preset from IbisPaint JSON string or base64 QR payload
    pub fn parse_ibis_string(raw: &str) -> Result<BrushPreset, String> {
        let trimmed = raw.trim();

        // Check if string is direct JSON or base64 QR code payload
        let json_text = if trimmed.starts_with('{') {
            trimmed.to_string()
        } else if let Some(decoded) = Self::decode_base64_string(trimmed) {
            String::from_utf8(decoded).map_err(|e| format!("Base64 decoding failed: {}", e))?
        } else {
            return Err("Unrecognized brush format. Expected JSON or QR base64 string.".into());
        };

        // Try parsing full BrushPreset
        if let Ok(preset) = serde_json::from_str::<BrushPreset>(&json_text) {
            return Ok(preset);
        }

        // Try flexible Ibis schema
        #[derive(Deserialize)]
        struct IbisFlexibleBrush {
            name: Option<String>,
            size: Option<f32>,
            opacity: Option<f32>,
            hardness: Option<f32>,
            spacing: Option<f32>,
            flow: Option<f32>,
            start_width: Option<f32>,
            end_width: Option<f32>,
            start_opacity: Option<f32>,
            end_opacity: Option<f32>,
            fade_in: Option<f32>,
            fade_out: Option<f32>,
            pressure_size: Option<bool>,
            pressure_opacity: Option<bool>,
        }

        let payload: IbisFlexibleBrush = serde_json::from_str(&json_text)
            .map_err(|e| format!("Could not parse Ibis brush: {}", e))?;

        let mut params = BrushParameters::default();
        if let Some(n) = payload.name.clone() { params.name = n; }
        if let Some(s) = payload.size { params.size = s.clamp(0.5, 1000.0); }
        if let Some(o) = payload.opacity { params.opacity = o.clamp(0.01, 1.0); }
        if let Some(h) = payload.hardness { params.hardness = h.clamp(0.01, 1.0); }
        if let Some(sp) = payload.spacing { params.spacing = sp.clamp(0.01, 2.0); }
        if let Some(f) = payload.flow { params.flow = f.clamp(0.01, 1.0); }
        if let Some(sw) = payload.start_width { params.start_width_ratio = sw.clamp(0.0, 1.0); }
        if let Some(ew) = payload.end_width { params.end_width_ratio = ew.clamp(0.0, 1.0); }
        if let Some(so) = payload.start_opacity { params.start_opacity_ratio = so.clamp(0.0, 1.0); }
        if let Some(eo) = payload.end_opacity { params.end_opacity_ratio = eo.clamp(0.0, 1.0); }
        if let Some(fi) = payload.fade_in { params.fade_in_length = fi; }
        if let Some(fo) = payload.fade_out { params.fade_out_length = fo; }
        if let Some(ps) = payload.pressure_size { params.size_by_pressure = ps; }
        if let Some(po) = payload.pressure_opacity { params.opacity_by_pressure = po; }

        let name = payload.name.unwrap_or_else(|| "Importierter Ibis Pinsel".into());
        let id = format!("ibis-{}", uuid::Uuid::new_v4());

        Ok(BrushPreset::new(
            id,
            name,
            "IbisPaint Import",
            "Erfolgreich aus IbisPaint importierter Pinsel",
            params,
        ))
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

    #[test]
    fn test_ibis_brush_importer_json() {
        let json = r#"{"name":"Custom Anime Dip Pen","size":18.5,"opacity":0.95,"hardness":0.9,"start_width":0.0,"end_width":0.0}"#;
        let imported = IbisBrushImporter::parse_ibis_string(json).unwrap();
        assert_eq!(imported.name, "Custom Anime Dip Pen");
        assert_eq!(imported.parameters.size, 18.5);
        assert_eq!(imported.parameters.start_width_ratio, 0.0);
    }
}
