use crate::format::StorageError;
use drawling_core::LayerStack;
use image::{ColorType, DynamicImage, ImageFormat, RgbaImage};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub struct ImageExporter;

impl ImageExporter {
    /// Flatten layer stack and export to PNG
    pub fn export_png(path: impl AsRef<Path>, stack: &LayerStack) -> Result<(), StorageError> {
        let mut flat = vec![0u8; (stack.width * stack.height * 4) as usize];
        stack.compose_to_buffer(&mut flat);

        let img = RgbaImage::from_raw(stack.width, stack.height, flat)
            .ok_or_else(|| StorageError::Format("Failed to create RGBA buffer".into()))?;

        img.save_with_format(path, ImageFormat::Png)?;
        Ok(())
    }

    /// Flatten layer stack and export to JPEG with white background for transparency
    pub fn export_jpeg(
        path: impl AsRef<Path>,
        stack: &LayerStack,
        quality: u8,
    ) -> Result<(), StorageError> {
        let mut flat = vec![0u8; (stack.width * stack.height * 4) as usize];
        stack.compose_to_buffer(&mut flat);

        // Composite onto white background for JPEG
        let mut rgb = Vec::with_capacity((stack.width * stack.height * 3) as usize);
        for chunk in flat.chunks_exact(4) {
            let a = chunk[3] as f32 / 255.0;
            let r = ((chunk[0] as f32 * a) + 255.0 * (1.0 - a)).round() as u8;
            let g = ((chunk[1] as f32 * a) + 255.0 * (1.0 - a)).round() as u8;
            let b = ((chunk[2] as f32 * a) + 255.0 * (1.0 - a)).round() as u8;
            rgb.push(r);
            rgb.push(g);
            rgb.push(b);
        }

        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, quality.clamp(1, 100));
        encoder.encode(&rgb, stack.width, stack.height, ColorType::Rgb8.into())?;

        Ok(())
    }

    /// Export SVG containing canvas artwork with scalable vector container
    pub fn export_svg(path: impl AsRef<Path>, stack: &LayerStack) -> Result<(), StorageError> {
        let mut flat = vec![0u8; (stack.width * stack.height * 4) as usize];
        stack.compose_to_buffer(&mut flat);

        let img = RgbaImage::from_raw(stack.width, stack.height, flat)
            .ok_or_else(|| StorageError::Format("Failed to create RGBA buffer".into()))?;

        let mut png_bytes = Vec::new();
        DynamicImage::ImageRgba8(img).write_to(&mut std::io::Cursor::new(&mut png_bytes), ImageFormat::Png)?;

        // Base64 encode PNG
        const B64_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut b64 = String::with_capacity(png_bytes.len() * 4 / 3 + 4);
        for chunk in png_bytes.chunks(3) {
            let b0 = chunk[0] as usize;
            let b1 = if chunk.len() > 1 { chunk[1] as usize } else { 0 };
            let b2 = if chunk.len() > 2 { chunk[2] as usize } else { 0 };

            let n = (b0 << 16) | (b1 << 8) | b2;
            b64.push(B64_CHARS[(n >> 18) & 63] as char);
            b64.push(B64_CHARS[(n >> 12) & 63] as char);
            b64.push(if chunk.len() > 1 { B64_CHARS[(n >> 6) & 63] as char } else { '=' });
            b64.push(if chunk.len() > 2 { B64_CHARS[n & 63] as char } else { '=' });
        }

        let svg_content = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{w}" height="{h}" viewBox="0 0 {w} {h}">
  <title>Drawling for Duckling Export</title>
  <image width="{w}" height="{h}" xlink:href="data:image/png;base64,{b64}" />
</svg>"#,
            w = stack.width,
            h = stack.height,
            b64 = b64
        );

        let mut file = File::create(path)?;
        file.write_all(svg_content.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_png_jpeg_svg() {
        let mut stack = LayerStack::new(8, 8);
        stack.layers[0].fill([255, 128, 64, 255]);

        let dir = std::env::temp_dir();
        let png_path = dir.join(format!("test_export_{}.png", uuid::Uuid::new_v4()));
        let jpg_path = dir.join(format!("test_export_{}.jpg", uuid::Uuid::new_v4()));
        let svg_path = dir.join(format!("test_export_{}.svg", uuid::Uuid::new_v4()));

        ImageExporter::export_png(&png_path, &stack).unwrap();
        assert!(std::fs::metadata(&png_path).unwrap().len() > 0);

        ImageExporter::export_jpeg(&jpg_path, &stack, 90).unwrap();
        assert!(std::fs::metadata(&jpg_path).unwrap().len() > 0);

        ImageExporter::export_svg(&svg_path, &stack).unwrap();
        let svg_str = std::fs::read_to_string(&svg_path).unwrap();
        assert!(svg_str.contains("<svg"));
        assert!(svg_str.contains("data:image/png;base64,"));

        let _ = std::fs::remove_file(png_path);
        let _ = std::fs::remove_file(jpg_path);
        let _ = std::fs::remove_file(svg_path);
    }
}
