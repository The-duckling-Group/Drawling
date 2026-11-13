use super::blend::BlendMode;
use crate::color::Color;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Layer {
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
    #[serde(default)]
    pub is_folder: bool,
    #[serde(default)]
    pub folder_id: Option<Uuid>,
    #[serde(skip)]
    pub pixels: Vec<u8>,
}

impl Layer {
    pub fn new(name: impl Into<String>, width: u32, height: u32) -> Self {
        let size = (width * height * 4) as usize;
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            width,
            height,
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            visible: true,
            locked: false,
            alpha_locked: false,
            clipping_mask: false,
            is_folder: false,
            folder_id: None,
            pixels: vec![0; size],
        }
    }

    pub fn new_with_color(name: impl Into<String>, width: u32, height: u32, color: [u8; 4]) -> Self {
        let mut layer = Self::new(name, width, height);
        layer.fill(color);
        layer
    }

    pub fn clear(&mut self) {
        self.pixels.fill(0);
    }

    pub fn fill(&mut self, color: [u8; 4]) {
        for chunk in self.pixels.chunks_exact_mut(4) {
            chunk.copy_from_slice(&color);
        }
    }

    pub fn fill_color(&mut self, color: Color) {
        self.fill(color.to_rgba8());
    }

    #[inline]
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        Some([
            self.pixels[idx],
            self.pixels[idx + 1],
            self.pixels[idx + 2],
            self.pixels[idx + 3],
        ])
    }

    #[inline]
    pub fn get_color(&self, x: u32, y: u32) -> Option<Color> {
        self.get_pixel(x, y).map(|p| Color::from_rgba8(p[0], p[1], p[2], p[3]))
    }

    #[inline]
    pub fn set_pixel(&mut self, x: u32, y: u32, color: [u8; 4]) {
        if x >= self.width || y >= self.height || self.locked {
            return;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        if self.alpha_locked {
            let current_a = self.pixels[idx + 3];
            if current_a == 0 {
                return;
            }
            self.pixels[idx] = color[0];
            self.pixels[idx + 1] = color[1];
            self.pixels[idx + 2] = color[2];
        } else {
            self.pixels[idx..idx + 4].copy_from_slice(&color);
        }
    }

    #[inline]
    pub fn set_color(&mut self, x: u32, y: u32, color: Color) {
        self.set_pixel(x, y, color.to_rgba8());
    }

    /// Blend a single pixel onto this layer directly
    #[inline]
    pub fn blend_pixel_onto(&mut self, x: u32, y: u32, src: [u8; 4], mode: BlendMode, opacity: f32) {
        if x >= self.width || y >= self.height || self.locked {
            return;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        let dst = [
            self.pixels[idx],
            self.pixels[idx + 1],
            self.pixels[idx + 2],
            self.pixels[idx + 3],
        ];
        let out = mode.composite_pixel(dst, src, opacity, self.alpha_locked);
        self.pixels[idx..idx + 4].copy_from_slice(&out);
    }

    pub fn clone_data(&self) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: format!("{} Copy", self.name),
            width: self.width,
            height: self.height,
            opacity: self.opacity,
            blend_mode: self.blend_mode,
            visible: self.visible,
            locked: self.locked,
            alpha_locked: self.alpha_locked,
            clipping_mask: self.clipping_mask,
            is_folder: self.is_folder,
            folder_id: self.folder_id,
            pixels: self.pixels.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerStack {
    pub width: u32,
    pub height: u32,
    pub layers: Vec<Layer>,
    pub active_index: usize,
}

impl LayerStack {
    pub fn new(width: u32, height: u32) -> Self {
        let background = Layer::new_with_color("Background", width, height, [255, 255, 255, 255]);
        let mut default_layer = Layer::new("Layer 1", width, height);
        default_layer.clipping_mask = false;

        Self {
            width,
            height,
            layers: vec![background, default_layer],
            active_index: 1,
        }
    }

    pub fn active_layer(&self) -> Option<&Layer> {
        self.layers.get(self.active_index)
    }

    pub fn active_layer_mut(&mut self) -> Option<&mut Layer> {
        self.layers.get_mut(self.active_index)
    }

    pub fn add_layer(&mut self, name: impl Into<String>) -> usize {
        let idx = (self.active_index + 1).min(self.layers.len());
        let new_layer = Layer::new(name, self.width, self.height);
        self.layers.insert(idx, new_layer);
        self.active_index = idx;
        idx
    }

    pub fn add_folder(&mut self, name: impl Into<String>) -> usize {
        let idx = (self.active_index + 1).min(self.layers.len());
        let mut folder = Layer::new(name, self.width, self.height);
        folder.is_folder = true;
        self.layers.insert(idx, folder);
        self.active_index = idx;
        idx
    }

    pub fn remove_layer(&mut self, index: usize) -> Option<Layer> {
        if self.layers.len() <= 1 || index >= self.layers.len() {
            return None;
        }
        let removed = self.layers.remove(index);
        if self.active_index >= self.layers.len() {
            self.active_index = self.layers.len() - 1;
        }
        Some(removed)
    }

    pub fn duplicate_layer(&mut self, index: usize) -> Option<usize> {
        if index >= self.layers.len() {
            return None;
        }
        let cloned = self.layers[index].clone_data();
        let target_idx = index + 1;
        self.layers.insert(target_idx, cloned);
        self.active_index = target_idx;
        Some(target_idx)
    }

    pub fn reorder_layer(&mut self, from: usize, to: usize) -> bool {
        if from >= self.layers.len() || to >= self.layers.len() || from == to {
            return false;
        }
        let layer = self.layers.remove(from);
        self.layers.insert(to, layer);
        self.active_index = to;
        true
    }

    /// Compose all visible layers into a unified RGBA8 buffer.
    /// Handles clipping masks correctly: if a layer has `clipping_mask == true`,
    /// it is clipped against the alpha channel of the base layer underneath it.
    pub fn compose_to_buffer(&self, output: &mut [u8]) {
        assert_eq!(output.len(), (self.width * self.height * 4) as usize);
        output.fill(0);

        let mut temp_target = vec![0u8; output.len()];

        // Find bottom-up compositing order
        for (i, layer) in self.layers.iter().enumerate() {
            if !layer.visible || layer.opacity <= 0.0 {
                continue;
            }

            // Check if this layer is clipped to the layer beneath
            let clip_base_idx = if layer.clipping_mask && i > 0 {
                // Find base layer (the first non-clipping layer below)
                let mut base = i - 1;
                while base > 0 && self.layers[base].clipping_mask {
                    base -= 1;
                }
                Some(base)
            } else {
                None
            };

            let mode = layer.blend_mode;
            let opacity = layer.opacity;

            // Multithreaded pixel-row composition using Rayon for 60+ FPS performance
            temp_target.par_chunks_exact_mut(4 * self.width as usize)
                .zip(output.par_chunks_exact(4 * self.width as usize))
                .zip(layer.pixels.par_chunks_exact(4 * self.width as usize))
                .enumerate()
                .for_each(|(row, ((dst_row, bg_row), src_row))| {
                    for col in 0..self.width as usize {
                        let px_idx = col * 4;
                        let dst_px = [
                            bg_row[px_idx],
                            bg_row[px_idx + 1],
                            bg_row[px_idx + 2],
                            bg_row[px_idx + 3],
                        ];
                        let mut src_px = [
                            src_row[px_idx],
                            src_row[px_idx + 1],
                            src_row[px_idx + 2],
                            src_row[px_idx + 3],
                        ];

                        // If clipped, modulate source alpha by base layer's alpha
                        if let Some(base_idx) = clip_base_idx {
                            let base_layer = &self.layers[base_idx];
                            let pixel_flat_idx = (row * self.width as usize + col) * 4 + 3;
                            let base_alpha = base_layer.pixels[pixel_flat_idx] as f32 / 255.0;
                            src_px[3] = (src_px[3] as f32 * base_alpha).round() as u8;
                        }

                        let res = mode.composite_pixel(dst_px, src_px, opacity, false);
                        dst_row[px_idx..px_idx + 4].copy_from_slice(&res);
                    }
                });

            output.copy_from_slice(&temp_target);
        }
    }

    /// Merges the layer at `upper_index` down into the layer at `upper_index - 1`
    pub fn merge_down(&mut self, upper_index: usize) -> bool {
        if upper_index == 0 || upper_index >= self.layers.len() {
            return false;
        }

        let upper = self.layers.remove(upper_index);
        let lower = &mut self.layers[upper_index - 1];

        // Composite upper onto lower
        let mode = upper.blend_mode;
        let opacity = upper.opacity;

        for (dst_px, src_px) in lower.pixels.chunks_exact_mut(4).zip(upper.pixels.chunks_exact(4)) {
            let d = [dst_px[0], dst_px[1], dst_px[2], dst_px[3]];
            let s = [src_px[0], src_px[1], src_px[2], src_px[3]];
            let res = mode.composite_pixel(d, s, opacity, lower.alpha_locked);
            dst_px.copy_from_slice(&res);
        }

        self.active_index = (upper_index - 1).min(self.layers.len() - 1);
        true
    }

    /// Flattens all visible layers into a single Background layer
    pub fn flatten(&mut self) {
        let mut flat_buffer = vec![0u8; (self.width * self.height * 4) as usize];
        self.compose_to_buffer(&mut flat_buffer);

        let mut base_layer = Layer::new("Background", self.width, self.height);
        base_layer.pixels = flat_buffer;

        self.layers = vec![base_layer];
        self.active_index = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_stack_add_remove() {
        let mut stack = LayerStack::new(64, 64);
        assert_eq!(stack.layers.len(), 2);

        let idx = stack.add_layer("Sketch");
        assert_eq!(stack.layers.len(), 3);
        assert_eq!(stack.active_index, idx);

        let removed = stack.remove_layer(idx);
        assert!(removed.is_some());
        assert_eq!(stack.layers.len(), 2);
    }

    #[test]
    fn test_merge_down() {
        let mut stack = LayerStack::new(2, 2);
        // Lower layer: white opaque
        stack.layers[0].fill([255, 255, 255, 255]);
        // Upper layer: red opaque with Multiply mode
        stack.layers[1].fill([255, 0, 0, 255]);
        stack.layers[1].blend_mode = BlendMode::Multiply;

        assert!(stack.merge_down(1));
        assert_eq!(stack.layers.len(), 1);
        // Multiplying white with red yields red
        assert_eq!(stack.layers[0].get_pixel(0, 0), Some([255, 0, 0, 255]));
    }

    #[test]
    fn test_clipping_mask_compositing() {
        let mut stack = LayerStack::new(2, 2);
        // Base layer: left pixel opaque, right pixel transparent
        stack.layers[0].clear();
        stack.layers[0].set_pixel(0, 0, [255, 255, 255, 255]);

        // Clipped layer: filled completely with blue
        stack.layers[1].fill([0, 0, 255, 255]);
        stack.layers[1].clipping_mask = true;

        let mut output = vec![0u8; 2 * 2 * 4];
        stack.compose_to_buffer(&mut output);

        // Pixel (0,0) has blue on white => blue
        assert_eq!(&output[0..4], &[0, 0, 255, 255]);
        // Pixel (1,0) should remain transparent because base layer alpha is 0
        assert_eq!(&output[4..8], &[0, 0, 0, 0]);
    }
}
