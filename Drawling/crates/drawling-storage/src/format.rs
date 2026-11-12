use chrono::{DateTime, Utc};
use drawling_core::{BlendMode, Layer, LayerStack};
use image::{DynamicImage, ImageFormat, RgbaImage};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::Path;
use thiserror::Error;
use uuid::Uuid;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),
    #[error("Invalid project file: {0}")]
    Format(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerMetadata {
    pub id: Uuid,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub visible: bool,
    pub locked: bool,
    pub alpha_locked: bool,
    pub clipping_mask: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawlingManifest {
    pub id: Uuid,
    pub title: String,
    pub author: String,
    pub version: u32,
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub dpi: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub edit_duration_secs: u64,
    pub tags: Vec<String>,
    pub layers: Vec<LayerMetadata>,
    pub active_layer_index: usize,
}

pub struct DrawlingArchive;

impl DrawlingArchive {
    /// Save a canvas and metadata into a compressed `.drawling` archive
    pub fn save(
        path: impl AsRef<Path>,
        manifest: &DrawlingManifest,
        stack: &LayerStack,
        thumbnail_rgba: Option<&[u8]>,
    ) -> Result<(), StorageError> {
        let file = File::create(path)?;
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        // 1. Write project.json manifest
        zip.start_file("project.json", options)?;
        let manifest_bytes = serde_json::to_vec_pretty(manifest)?;
        zip.write_all(&manifest_bytes)?;

        // 2. Write thumbnail.png if provided, or generate from layer stack
        zip.start_file("thumbnail.png", options)?;
        let mut thumb_buf = Vec::new();
        if let Some(thumb_data) = thumbnail_rgba {
            let img = RgbaImage::from_raw(manifest.canvas_width, manifest.canvas_height, thumb_data.to_vec())
                .ok_or_else(|| StorageError::Format("Invalid thumbnail dimensions".into()))?;
            // Downscale to 256x256 max for fast gallery preview
            let dyn_img = DynamicImage::ImageRgba8(img).thumbnail(256, 256);
            dyn_img.write_to(&mut Cursor::new(&mut thumb_buf), ImageFormat::Png)?;
        } else {
            let mut flat = vec![0u8; (manifest.canvas_width * manifest.canvas_height * 4) as usize];
            stack.compose_to_buffer(&mut flat);
            let img = RgbaImage::from_raw(manifest.canvas_width, manifest.canvas_height, flat)
                .ok_or_else(|| StorageError::Format("Failed to generate composite".into()))?;
            let dyn_img = DynamicImage::ImageRgba8(img).thumbnail(256, 256);
            dyn_img.write_to(&mut Cursor::new(&mut thumb_buf), ImageFormat::Png)?;
        }
        zip.write_all(&thumb_buf)?;

        // 3. Write individual layer PNGs
        for layer in &stack.layers {
            let layer_filename = format!("layers/{}.png", layer.id);
            zip.start_file(layer_filename, options)?;

            let img = RgbaImage::from_raw(layer.width, layer.height, layer.pixels.clone())
                .ok_or_else(|| StorageError::Format(format!("Invalid layer buffer for {}", layer.id)))?;
            let mut layer_buf = Vec::new();
            img.write_to(&mut Cursor::new(&mut layer_buf), ImageFormat::Png)?;
            zip.write_all(&layer_buf)?;
        }

        zip.finish()?;
        Ok(())
    }

    /// Load a project from a compressed `.drawling` archive
    pub fn load(path: impl AsRef<Path>) -> Result<(DrawlingManifest, LayerStack, Vec<u8>), StorageError> {
        let file = File::open(path)?;
        let mut archive = ZipArchive::new(file)?;

        // 1. Read project.json
        let manifest_str = {
            let mut manifest_file = archive.by_name("project.json")
                .map_err(|_| StorageError::Format("Missing project.json in .drawling container".into()))?;
            let mut s = String::new();
            manifest_file.read_to_string(&mut s)?;
            s
        };
        let manifest: DrawlingManifest = serde_json::from_str(&manifest_str)?;

        // 2. Read thumbnail.png
        let mut thumbnail_bytes = Vec::new();
        if let Ok(mut thumb_file) = archive.by_name("thumbnail.png") {
            thumb_file.read_to_end(&mut thumbnail_bytes)?;
        }

        // 3. Reconstruct LayerStack
        let mut layers = Vec::new();
        for meta in &manifest.layers {
            let layer_filename = format!("layers/{}.png", meta.id);
            let img_bytes = {
                let mut layer_file = archive.by_name(&layer_filename)
                    .map_err(|_| StorageError::Format(format!("Missing layer file: {}", layer_filename)))?;
                let mut bytes = Vec::new();
                layer_file.read_to_end(&mut bytes)?;
                bytes
            };

            let dyn_img = image::load_from_memory_with_format(&img_bytes, ImageFormat::Png)?;
            let rgba = dyn_img.to_rgba8();

            let mut layer = Layer::new(&meta.name, meta.width, meta.height);
            layer.id = meta.id;
            layer.opacity = meta.opacity;
            layer.blend_mode = meta.blend_mode;
            layer.visible = meta.visible;
            layer.locked = meta.locked;
            layer.alpha_locked = meta.alpha_locked;
            layer.clipping_mask = meta.clipping_mask;
            layer.pixels = rgba.into_raw();

            layers.push(layer);
        }

        let stack = LayerStack {
            width: manifest.canvas_width,
            height: manifest.canvas_height,
            active_index: manifest.active_layer_index.min(layers.len().saturating_sub(1)),
            layers,
        };

        Ok((manifest, stack, thumbnail_bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drawling_archive_roundtrip() {
        let manifest = DrawlingManifest {
            id: Uuid::new_v4(),
            title: "Roundtrip Test".into(),
            author: "Tester".into(),
            version: 1,
            canvas_width: 16,
            canvas_height: 16,
            dpi: 300.0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            edit_duration_secs: 42,
            tags: vec!["Test".into()],
            layers: vec![
                LayerMetadata {
                    id: Uuid::new_v4(),
                    name: "Base".into(),
                    width: 16,
                    height: 16,
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                    visible: true,
                    locked: false,
                    alpha_locked: false,
                    clipping_mask: false,
                },
            ],
            active_layer_index: 0,
        };

        let mut stack = LayerStack::new(16, 16);
        stack.layers[0].id = manifest.layers[0].id;
        stack.layers[0].name = "Base".into();
        stack.layers[0].set_pixel(5, 5, [128, 64, 32, 255]);
        stack.layers.truncate(1);

        let temp_path = std::env::temp_dir().join(format!("test_archive_{}.drawling", Uuid::new_v4()));
        DrawlingArchive::save(&temp_path, &manifest, &stack, None).unwrap();

        let (loaded_man, loaded_stack, thumb) = DrawlingArchive::load(&temp_path).unwrap();
        assert_eq!(loaded_man.title, "Roundtrip Test");
        assert_eq!(loaded_stack.layers.len(), 1);
        assert_eq!(loaded_stack.layers[0].get_pixel(5, 5), Some([128, 64, 32, 255]));
        assert!(!thumb.is_empty());

        let _ = std::fs::remove_file(temp_path);
    }
}
