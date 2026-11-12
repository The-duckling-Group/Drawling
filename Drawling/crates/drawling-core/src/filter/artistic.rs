use crate::filter::GaussianBlur;
use crate::layer::Layer;
use rayon::prelude::*;

pub struct ArtisticFilter;

impl ArtisticFilter {
    /// Motion Blur: Directional blur along an angle with distance.
    pub fn motion_blur(layer: &mut Layer, angle_deg: f32, distance: f32) {
        if layer.locked || distance <= 0.0 {
            return;
        }

        let width = layer.width as usize;
        let height = layer.height as usize;
        let rad = angle_deg.to_radians();
        let dx = rad.cos();
        let dy = rad.sin();

        let num_samples = (distance.abs().ceil() as usize).max(3).min(64);
        let src_pixels = layer.pixels.clone();

        layer.pixels.par_chunks_exact_mut(4).enumerate().for_each(|(i, pixel)| {
            let x = (i % width) as f32;
            let y = (i / width) as f32;

            let mut sum_r = 0.0;
            let mut sum_g = 0.0;
            let mut sum_b = 0.0;
            let mut sum_a = 0.0;
            let mut count = 0.0;

            let half = (num_samples as f32) / 2.0;
            for s in 0..num_samples {
                let offset = (s as f32 - half) * (distance / num_samples as f32);
                let sx = (x + dx * offset).round() as isize;
                let sy = (y + dy * offset).round() as isize;

                if sx >= 0 && sx < width as isize && sy >= 0 && sy < height as isize {
                    let idx = ((sy as usize) * width + (sx as usize)) * 4;
                    sum_r += src_pixels[idx] as f32;
                    sum_g += src_pixels[idx + 1] as f32;
                    sum_b += src_pixels[idx + 2] as f32;
                    sum_a += src_pixels[idx + 3] as f32;
                    count += 1.0;
                }
            }

            if count > 0.0 {
                pixel[0] = (sum_r / count).clamp(0.0, 255.0).round() as u8;
                pixel[1] = (sum_g / count).clamp(0.0, 255.0).round() as u8;
                pixel[2] = (sum_b / count).clamp(0.0, 255.0).round() as u8;
                pixel[3] = (sum_a / count).clamp(0.0, 255.0).round() as u8;
            }
        });
    }

    /// Radial / Zoom Blur: Blur radiating outward from a focus center (cx, cy).
    pub fn radial_blur(layer: &mut Layer, center_x: f32, center_y: f32, amount: f32, samples: usize) {
        if layer.locked || amount <= 0.0 {
            return;
        }

        let width = layer.width as usize;
        let height = layer.height as usize;
        let num_samples = samples.clamp(4, 48);
        let src_pixels = layer.pixels.clone();

        layer.pixels.par_chunks_exact_mut(4).enumerate().for_each(|(i, pixel)| {
            let x = (i % width) as f32;
            let y = (i / width) as f32;

            let vx = x - center_x;
            let vy = y - center_y;

            let mut sum_r = 0.0;
            let mut sum_g = 0.0;
            let mut sum_b = 0.0;
            let mut sum_a = 0.0;
            let mut count = 0.0;

            for s in 0..num_samples {
                let scale = 1.0 - (s as f32 / num_samples as f32) * amount * 0.1;
                let sx = (center_x + vx * scale).round() as isize;
                let sy = (center_y + vy * scale).round() as isize;

                if sx >= 0 && sx < width as isize && sy >= 0 && sy < height as isize {
                    let idx = ((sy as usize) * width + (sx as usize)) * 4;
                    sum_r += src_pixels[idx] as f32;
                    sum_g += src_pixels[idx + 1] as f32;
                    sum_b += src_pixels[idx + 2] as f32;
                    sum_a += src_pixels[idx + 3] as f32;
                    count += 1.0;
                }
            }

            if count > 0.0 {
                pixel[0] = (sum_r / count).clamp(0.0, 255.0).round() as u8;
                pixel[1] = (sum_g / count).clamp(0.0, 255.0).round() as u8;
                pixel[2] = (sum_b / count).clamp(0.0, 255.0).round() as u8;
                pixel[3] = (sum_a / count).clamp(0.0, 255.0).round() as u8;
            }
        });
    }

    /// Unsharp Mask: Enhances edges by subtracting Gaussian blur and adding back high frequencies.
    pub fn unsharp_mask(layer: &mut Layer, radius: f32, amount: f32, threshold: u8) {
        if layer.locked || radius <= 0.0 || amount <= 0.0 {
            return;
        }

        let mut blurred = layer.clone();
        GaussianBlur::apply(&mut blurred, radius);

        layer.pixels.par_chunks_exact_mut(4)
            .zip(blurred.pixels.par_chunks_exact(4))
            .for_each(|(orig, blur)| {
                if orig[3] == 0 {
                    return;
                }

                for c in 0..3 {
                    let diff = orig[c] as f32 - blur[c] as f32;
                    if diff.abs() >= threshold as f32 {
                        let sharpened = orig[c] as f32 + diff * amount;
                        orig[c] = sharpened.clamp(0.0, 255.0).round() as u8;
                    }
                }
            });
    }

    /// Manga Halftone / Screentone: Generates a dot screentone pattern from layer luminance.
    pub fn halftone(layer: &mut Layer, dot_size: f32, angle_deg: f32, invert: bool) {
        if layer.locked || dot_size <= 1.0 {
            return;
        }

        let width = layer.width as usize;
        let rad = angle_deg.to_radians();
        let cos_a = rad.cos();
        let sin_a = rad.sin();

        layer.pixels.par_chunks_exact_mut(4).enumerate().for_each(|(i, pixel)| {
            let a = pixel[3];
            if a == 0 {
                return;
            }

            let x = (i % width) as f32;
            let y = (i / width) as f32;

            // Rotate coordinates to halftone grid angle
            let rx = x * cos_a - y * sin_a;
            let ry = x * sin_a + y * cos_a;

            // Grid cell coordinates
            let cell_x = (rx.rem_euclid(dot_size)) - dot_size * 0.5;
            let cell_y = (ry.rem_euclid(dot_size)) - dot_size * 0.5;
            let dist_from_center = (cell_x * cell_x + cell_y * cell_y).sqrt();

            // Luminance in [0, 1]
            let lum = (0.299 * pixel[0] as f32 + 0.587 * pixel[1] as f32 + 0.114 * pixel[2] as f32) / 255.0;
            let darkness = if invert { lum } else { 1.0 - lum };

            // Dot radius proportional to darkness
            let max_radius = dot_size * std::f32::consts::FRAC_1_SQRT_2;
            let dot_radius = darkness * max_radius;

            if dist_from_center <= dot_radius {
                pixel[0] = 0;
                pixel[1] = 0;
                pixel[2] = 0;
                pixel[3] = 255;
            } else {
                pixel[0] = 255;
                pixel[1] = 255;
                pixel[2] = 255;
                pixel[3] = 255;
            }
        });
    }

    /// Chromatic Aberration: Color fringing displacement for retro / lens aesthetics.
    pub fn chromatic_aberration(layer: &mut Layer, offset_x: f32, offset_y: f32) {
        if layer.locked || (offset_x.abs() < 0.01 && offset_y.abs() < 0.01) {
            return;
        }

        let width = layer.width as usize;
        let height = layer.height as usize;
        let src = layer.pixels.clone();

        let ox = offset_x.round() as isize;
        let oy = offset_y.round() as isize;

        layer.pixels.par_chunks_exact_mut(4).enumerate().for_each(|(i, pixel)| {
            let x = (i % width) as isize;
            let y = (i / width) as isize;

            // Red channel shifted +offset
            let rx = x + ox;
            let ry = y + oy;
            if rx >= 0 && rx < width as isize && ry >= 0 && ry < height as isize {
                let ridx = ((ry as usize) * width + (rx as usize)) * 4;
                pixel[0] = src[ridx];
            }

            // Blue channel shifted -offset
            let bx = x - ox;
            let by = y - oy;
            if bx >= 0 && bx < width as isize && by >= 0 && by < height as isize {
                let bidx = ((by as usize) * width + (bx as usize)) * 4;
                pixel[2] = src[bidx + 2];
            }
            // Green channel remains unmodified at original position
        });
    }

    /// Bloom / Glow: Extracts bright pixels, blurs them, and screens them back over original.
    pub fn bloom(layer: &mut Layer, threshold: f32, blur_radius: f32, intensity: f32) {
        if layer.locked || blur_radius <= 0.0 || intensity <= 0.0 {
            return;
        }

        let mut highlights = layer.clone();
        let thresh_val = (threshold * 255.0).clamp(0.0, 255.0);

        // High-pass filter for luminance
        highlights.pixels.par_chunks_exact_mut(4).for_each(|pixel| {
            let lum = 0.299 * pixel[0] as f32 + 0.587 * pixel[1] as f32 + 0.114 * pixel[2] as f32;
            if lum < thresh_val {
                pixel[0] = 0;
                pixel[1] = 0;
                pixel[2] = 0;
                pixel[3] = 0;
            }
        });

        GaussianBlur::apply(&mut highlights, blur_radius);

        // Screen / additive blend glow
        layer.pixels.par_chunks_exact_mut(4)
            .zip(highlights.pixels.par_chunks_exact(4))
            .for_each(|(orig, glow)| {
                if glow[3] > 0 {
                    for c in 0..3 {
                        let base = orig[c] as f32 / 255.0;
                        let light = (glow[c] as f32 / 255.0) * intensity;
                        // Additive blend with clamp
                        let blended = (base + light).clamp(0.0, 1.0);
                        orig[c] = (blended * 255.0).round() as u8;
                    }
                }
            });
    }

    /// Mosaic / Pixelate: Downsamples pixel blocks into solid color tiles.
    pub fn mosaic(layer: &mut Layer, block_size: u32) {
        if layer.locked || block_size <= 1 {
            return;
        }

        let width = layer.width;
        let height = layer.height;
        let b = block_size;

        for by in (0..height).step_by(b as usize) {
            for bx in (0..width).step_by(b as usize) {
                let max_x = (bx + b).min(width);
                let max_y = (by + b).min(height);

                let mut sum_r = 0u64;
                let mut sum_g = 0u64;
                let mut sum_b = 0u64;
                let mut sum_a = 0u64;
                let mut count = 0u64;

                for py in by..max_y {
                    for px in bx..max_x {
                        let idx = ((py * width + px) * 4) as usize;
                        sum_r += layer.pixels[idx] as u64;
                        sum_g += layer.pixels[idx + 1] as u64;
                        sum_b += layer.pixels[idx + 2] as u64;
                        sum_a += layer.pixels[idx + 3] as u64;
                        count += 1;
                    }
                }

                if count > 0 {
                    let avg_r = (sum_r / count) as u8;
                    let avg_g = (sum_g / count) as u8;
                    let avg_b = (sum_b / count) as u8;
                    let avg_a = (sum_a / count) as u8;

                    for py in by..max_y {
                        for px in bx..max_x {
                            let idx = ((py * width + px) * 4) as usize;
                            layer.pixels[idx] = avg_r;
                            layer.pixels[idx + 1] = avg_g;
                            layer.pixels[idx + 2] = avg_b;
                            layer.pixels[idx + 3] = avg_a;
                        }
                    }
                }
            }
        }
    }

    /// Film Grain / Noise: Adds subtle photographic noise.
    pub fn film_grain(layer: &mut Layer, intensity: f32, monochrome: bool) {
        if layer.locked || intensity <= 0.0 {
            return;
        }

        let width = layer.width as usize;
        let scale = intensity * 128.0;

        layer.pixels.par_chunks_exact_mut(4).enumerate().for_each(|(i, pixel)| {
            if pixel[3] == 0 {
                return;
            }

            let x = (i % width) as u32;
            let y = (i / width) as u32;

            // Fast deterministic hash noise for repeatable rendering
            let hash = |seed: u32| -> f32 {
                let mut n = x.wrapping_mul(374761393)
                    .wrapping_add(y.wrapping_mul(668265263))
                    .wrapping_add(seed.wrapping_mul(942187));
                n = (n ^ (n >> 13)).wrapping_mul(1274126177);
                ((n ^ (n >> 16)) & 0xFF) as f32 / 255.0 - 0.5
            };

            if monochrome {
                let n = hash(42) * scale;
                for c in 0..3 {
                    let v = pixel[c] as f32 + n;
                    pixel[c] = v.clamp(0.0, 255.0).round() as u8;
                }
            } else {
                let nr = hash(11) * scale;
                let ng = hash(22) * scale;
                let nb = hash(33) * scale;
                pixel[0] = (pixel[0] as f32 + nr).clamp(0.0, 255.0).round() as u8;
                pixel[1] = (pixel[1] as f32 + ng).clamp(0.0, 255.0).round() as u8;
                pixel[2] = (pixel[2] as f32 + nb).clamp(0.0, 255.0).round() as u8;
            }
        });
    }

    /// Posterize: Quantizes colors to `levels` steps per channel (e.g. 2 to 16).
    pub fn posterize(layer: &mut Layer, levels: u8) {
        if layer.locked || levels <= 1 {
            return;
        }

        let num_levels = levels.max(2) as f32;
        let step = 255.0 / (num_levels - 1.0);

        layer.pixels.par_chunks_exact_mut(4).for_each(|pixel| {
            if pixel[3] > 0 {
                for c in 0..3 {
                    let v = pixel[c] as f32;
                    let quantized = ((v / step).round() * step).clamp(0.0, 255.0);
                    pixel[c] = quantized.round() as u8;
                }
            }
        });
    }

    /// Threshold: Converts image to strictly binary black and white.
    pub fn threshold(layer: &mut Layer, cutoff: u8) {
        if layer.locked {
            return;
        }

        layer.pixels.par_chunks_exact_mut(4).for_each(|pixel| {
            if pixel[3] > 0 {
                let lum = (0.299 * pixel[0] as f32 + 0.587 * pixel[1] as f32 + 0.114 * pixel[2] as f32).round() as u8;
                let val = if lum >= cutoff { 255 } else { 0 };
                pixel[0] = val;
                pixel[1] = val;
                pixel[2] = val;
            }
        });
    }

    /// Levels & Curves adjustment: Sets black point, white point, and midtone gamma.
    pub fn levels(layer: &mut Layer, in_black: u8, in_white: u8, gamma: f32, out_black: u8, out_white: u8) {
        if layer.locked || in_white <= in_black || gamma <= 0.0 {
            return;
        }

        // Precompute 256-entry lookup table for rapid execution
        let mut lut = [0u8; 256];
        let in_range = (in_white - in_black) as f32;
        let out_range = (out_white as f32) - (out_black as f32);
        let inv_gamma = 1.0 / gamma;

        for i in 0..256 {
            let v = i as f32;
            let normalized = ((v - in_black as f32) / in_range).clamp(0.0, 1.0);
            let gamma_corrected = normalized.powf(inv_gamma);
            let mapped = out_black as f32 + gamma_corrected * out_range;
            lut[i] = mapped.clamp(0.0, 255.0).round() as u8;
        }

        layer.pixels.par_chunks_exact_mut(4).for_each(|pixel| {
            if pixel[3] > 0 {
                pixel[0] = lut[pixel[0] as usize];
                pixel[1] = lut[pixel[1] as usize];
                pixel[2] = lut[pixel[2] as usize];
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;

    #[test]
    fn test_motion_and_radial_blur() {
        let mut layer = Layer::new("Test Layer", 20, 20);
        layer.fill_color(Color::WHITE);
        layer.set_color(10, 10, Color::BLACK);

        ArtisticFilter::motion_blur(&mut layer, 0.0, 6.0);
        // Pixels on horizontal line near (10, 10) should have spread darkness
        let p_spread = layer.get_color(12, 10).unwrap();
        assert!(p_spread.r < 1.0, "Motion blur did not spread along angle");

        ArtisticFilter::radial_blur(&mut layer, 10.0, 10.0, 1.5, 8);
    }

    #[test]
    fn test_unsharp_mask_and_mosaic() {
        let mut layer = Layer::new("Mosaic Test", 16, 16);
        for y in 0..16 {
            for x in 0..16 {
                let c = if (x / 4 + y / 4) % 2 == 0 { Color::BLACK } else { Color::WHITE };
                layer.set_color(x, y, c);
            }
        }

        ArtisticFilter::mosaic(&mut layer, 4);
        // All pixels in cell (0..4, 0..4) must be uniform
        let sample = layer.get_color(0, 0);
        for y in 0..4 {
            for x in 0..4 {
                assert_eq!(layer.get_color(x, y), sample);
            }
        }

        ArtisticFilter::unsharp_mask(&mut layer, 1.0, 1.5, 5);
    }

    #[test]
    fn test_levels_and_threshold() {
        let mut layer = Layer::new("Tone Test", 10, 10);
        layer.fill_color(Color::from_rgba8(128, 128, 128, 255));

        ArtisticFilter::levels(&mut layer, 50, 200, 1.0, 0, 255);
        let adjusted = layer.get_color(5, 5).unwrap();
        assert!(adjusted.r > 0.0);

        ArtisticFilter::threshold(&mut layer, 128);
        let bin = layer.get_color(5, 5).unwrap();
        assert!(bin.r == 1.0 || bin.r == 0.0);
    }
}
