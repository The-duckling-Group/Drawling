use crate::color::Color;
use crate::layer::Layer;
use rayon::prelude::*;

pub struct ColorAdjust;

impl ColorAdjust {
    /// Adjust brightness (-1.0 to 1.0) and contrast (-1.0 to 1.0)
    pub fn brightness_contrast(layer: &mut Layer, brightness: f32, contrast: f32) {
        if layer.locked {
            return;
        }

        // Contrast factor: contrast in [-1.0, 1.0]
        let factor = (259.0 * (contrast + 1.0)) / (255.0 * (1.0 - contrast * 0.99));
        let b_offset = brightness * 255.0;

        layer.pixels.par_chunks_exact_mut(4).for_each(|pixel| {
            if pixel[3] > 0 {
                for c in 0..3 {
                    let v = pixel[c] as f32;
                    let adjusted = factor * (v - 128.0) + 128.0 + b_offset;
                    pixel[c] = adjusted.clamp(0.0, 255.0).round() as u8;
                }
            }
        });
    }

    /// Adjust hue shift (-180.0 to 180.0 deg) and saturation multiplier (0.0 to 3.0)
    pub fn hue_saturation(layer: &mut Layer, hue_shift_deg: f32, saturation_scale: f32) {
        if layer.locked {
            return;
        }

        layer.pixels.par_chunks_exact_mut(4).for_each(|pixel| {
            if pixel[3] > 0 {
                let color = Color::from_rgba8(pixel[0], pixel[1], pixel[2], pixel[3]);
                let (h, s, v) = color.to_hsv();

                let new_h = (h + hue_shift_deg).rem_euclid(360.0);
                let new_s = (s * saturation_scale).clamp(0.0, 1.0);

                let adjusted = Color::from_hsv(new_h, new_s, v, color.a);
                let [r, g, b, _] = adjusted.to_rgba8();
                pixel[0] = r;
                pixel[1] = g;
                pixel[2] = b;
            }
        });
    }

    /// Gradient Map: Maps grayscale intensity to an interpolated color ramp.
    /// `stops`: sequence of `(position in [0.0, 1.0], Color)` sorted by position.
    pub fn gradient_map(layer: &mut Layer, stops: &[(f32, Color)]) {
        if layer.locked || stops.is_empty() {
            return;
        }

        // Precompute 256-entry lookup table for high-speed multi-threaded mapping
        let mut lut = [[0u8; 3]; 256];
        for i in 0..256 {
            let t = i as f32 / 255.0;
            let mapped_color = sample_gradient(stops, t);
            let [r, g, b, _] = mapped_color.to_rgba8();
            lut[i] = [r, g, b];
        }

        layer.pixels.par_chunks_exact_mut(4).for_each(|pixel| {
            if pixel[3] > 0 {
                // ITU-R BT.601 standard luminance calculation
                let lum = (0.299 * pixel[0] as f32
                    + 0.587 * pixel[1] as f32
                    + 0.114 * pixel[2] as f32)
                    .round()
                    .clamp(0.0, 255.0) as usize;

                let mapped = lut[lum];
                pixel[0] = mapped[0];
                pixel[1] = mapped[1];
                pixel[2] = mapped[2];
            }
        });
    }

    /// Invert RGB color channels of layer
    pub fn invert(layer: &mut Layer) {
        if layer.locked {
            return;
        }

        layer.pixels.par_chunks_exact_mut(4).for_each(|pixel| {
            if pixel[3] > 0 {
                pixel[0] = 255 - pixel[0];
                pixel[1] = 255 - pixel[1];
                pixel[2] = 255 - pixel[2];
            }
        });
    }

    /// Apply Sepia tone effect with configurable intensity in [0.0, 1.0]
    pub fn sepia(layer: &mut Layer, intensity: f32) {
        if layer.locked {
            return;
        }
        let k = intensity.clamp(0.0, 1.0);

        layer.pixels.par_chunks_exact_mut(4).for_each(|pixel| {
            if pixel[3] > 0 {
                let r = pixel[0] as f32;
                let g = pixel[1] as f32;
                let b = pixel[2] as f32;

                let tr = (0.393 * r + 0.769 * g + 0.189 * b).min(255.0);
                let tg = (0.349 * r + 0.686 * g + 0.168 * b).min(255.0);
                let tb = (0.272 * r + 0.534 * g + 0.131 * b).min(255.0);

                pixel[0] = (r + (tr - r) * k).round() as u8;
                pixel[1] = (g + (tg - g) * k).round() as u8;
                pixel[2] = (b + (tb - b) * k).round() as u8;
            }
        });
    }

    /// Apply radial Vignette darkening around canvas center
    pub fn vignette(layer: &mut Layer, radius_ratio: f32, darkness: f32) {
        if layer.locked {
            return;
        }

        let w = layer.width as f32;
        let h = layer.height as f32;
        let cx = w * 0.5;
        let cy = h * 0.5;
        let max_r = (cx * cx + cy * cy).sqrt() * radius_ratio.max(0.1);
        let dark = darkness.clamp(0.0, 1.0);

        let width = layer.width as usize;

        layer.pixels
            .par_chunks_exact_mut(width * 4)
            .enumerate()
            .for_each(|(y, row)| {
                let dy = y as f32 + 0.5 - cy;
                for x in 0..width {
                    let dx = x as f32 + 0.5 - cx;
                    let dist = (dx * dx + dy * dy).sqrt();

                    if dist > max_r * 0.5 {
                        let factor = ((dist - max_r * 0.5) / (max_r * 0.5)).clamp(0.0, 1.0);
                        let dim = 1.0 - factor * dark;
                        let idx = x * 4;
                        row[idx] = (row[idx] as f32 * dim).round() as u8;
                        row[idx + 1] = (row[idx + 1] as f32 * dim).round() as u8;
                        row[idx + 2] = (row[idx + 2] as f32 * dim).round() as u8;
                    }
                }
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invert_filter() {
        let mut layer = Layer::new_with_color("Test", 2, 2, [100, 150, 200, 255]);
        ColorAdjust::invert(&mut layer);
        assert_eq!(layer.get_pixel(0, 0), Some([155, 105, 55, 255]));
    }

    #[test]
    fn test_sepia_filter() {
        let mut layer = Layer::new_with_color("Test", 2, 2, [100, 100, 100, 255]);
        ColorAdjust::sepia(&mut layer, 1.0);
        let px = layer.get_pixel(0, 0).unwrap();
        // Sepia red should be greater than green, which is greater than blue
        assert!(px[0] >= px[1]);
        assert!(px[1] >= px[2]);
    }

    #[test]
    fn test_gradient_map() {
        let mut layer = Layer::new_with_color("Test", 2, 2, [0, 0, 0, 255]);
        let stops = vec![
            (0.0, Color::new(1.0, 0.0, 0.0, 1.0)), // Red for dark
            (1.0, Color::new(0.0, 0.0, 1.0, 1.0)), // Blue for light
        ];
        ColorAdjust::gradient_map(&mut layer, &stops);
        assert_eq!(layer.get_pixel(0, 0), Some([255, 0, 0, 255]));
    }
}

fn sample_gradient(stops: &[(f32, Color)], t: f32) -> Color {
    if stops.len() == 1 || t <= stops[0].0 {
        return stops[0].1;
    }
    if t >= stops[stops.len() - 1].0 {
        return stops[stops.len() - 1].1;
    }

    for window in stops.windows(2) {
        let (p0, c0) = window[0];
        let (p1, c1) = window[1];
        if t >= p0 && t <= p1 {
            let range = p1 - p0;
            let local_t = if range > 0.0001 { (t - p0) / range } else { 0.0 };
            return Color::new(
                c0.r + (c1.r - c0.r) * local_t,
                c0.g + (c1.g - c0.g) * local_t,
                c0.b + (c1.b - c0.b) * local_t,
                c0.a + (c1.a - c0.a) * local_t,
            );
        }
    }

    stops[stops.len() - 1].1
}
