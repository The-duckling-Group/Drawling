use super::engine::BrushDab;
use crate::layer::Layer;

pub struct BrushTipRasterizer;

impl BrushTipRasterizer {
    /// Render a single brush dab onto a target layer, with full support for
    /// circular and tilted elliptical dabs.
    pub fn render_dab(layer: &mut Layer, dab: &BrushDab) {
        if layer.locked || dab.radius <= 0.0 || dab.opacity <= 0.0 {
            return;
        }

        let radius = dab.radius;
        let hardness = dab.hardness.clamp(0.01, 1.0);
        let inner_radius = radius * hardness;
        let radius_sq = radius * radius;
        let inner_sq = inner_radius * inner_radius;

        let min_x = ((dab.x - radius).floor() as i32).max(0) as u32;
        let max_x = ((dab.x + radius).ceil() as i32).min(layer.width as i32 - 1).max(0) as u32;
        let min_y = ((dab.y - radius).floor() as i32).max(0) as u32;
        let max_y = ((dab.y + radius).ceil() as i32).min(layer.height as i32 - 1).max(0) as u32;

        let dab_rgba = dab.color.to_rgba8();

        let is_elliptical = dab.aspect_ratio < 0.99;
        let (cos_a, sin_a) = if is_elliptical {
            (dab.angle_rad.cos(), dab.angle_rad.sin())
        } else {
            (1.0, 0.0)
        };
        let inv_aspect = if is_elliptical {
            1.0 / dab.aspect_ratio.max(0.1)
        } else {
            1.0
        };

        for py in min_y..=max_y {
            let dy = py as f32 + 0.5 - dab.y;

            for px in min_x..=max_x {
                let dx = px as f32 + 0.5 - dab.x;

                let dist_sq = if is_elliptical {
                    let rx = dx * cos_a + dy * sin_a;
                    let ry = (-dx * sin_a + dy * cos_a) * inv_aspect;
                    rx * rx + ry * ry
                } else {
                    dx * dx + dy * dy
                };

                if dist_sq > radius_sq {
                    continue;
                }

                // Falloff factor
                let falloff = if dist_sq <= inner_sq {
                    1.0
                } else {
                    let dist = dist_sq.sqrt();
                    let t = (dist - inner_radius) / (radius - inner_radius);
                    // Smoothstep falloff
                    (1.0 - t * t * (3.0 - 2.0 * t)).clamp(0.0, 1.0)
                };

                let dab_alpha = (dab.opacity * falloff * dab.color.a).clamp(0.0, 1.0);
                if dab_alpha <= 0.001 {
                    continue;
                }

                let src = [
                    dab_rgba[0],
                    dab_rgba[1],
                    dab_rgba[2],
                    (dab_alpha * 255.0).round() as u8,
                ];

                layer.blend_pixel_onto(px, py, src, layer.blend_mode, 1.0);
            }
        }
    }

    /// Render an eraser dab onto target layer, reducing destination alpha according to brush hardness and opacity.
    pub fn render_eraser_dab(layer: &mut Layer, dab: &BrushDab) {
        if layer.locked || dab.radius <= 0.0 || dab.opacity <= 0.0 {
            return;
        }

        let radius = dab.radius;
        let hardness = dab.hardness.clamp(0.01, 1.0);
        let inner_radius = radius * hardness;
        let radius_sq = radius * radius;
        let inner_sq = inner_radius * inner_radius;

        let min_x = ((dab.x - radius).floor() as i32).max(0) as u32;
        let max_x = ((dab.x + radius).ceil() as i32).min(layer.width as i32 - 1).max(0) as u32;
        let min_y = ((dab.y - radius).floor() as i32).max(0) as u32;
        let max_y = ((dab.y + radius).ceil() as i32).min(layer.height as i32 - 1).max(0) as u32;

        let is_elliptical = dab.aspect_ratio < 0.99;
        let (cos_a, sin_a) = if is_elliptical {
            (dab.angle_rad.cos(), dab.angle_rad.sin())
        } else {
            (1.0, 0.0)
        };
        let inv_aspect = if is_elliptical {
            1.0 / dab.aspect_ratio.max(0.1)
        } else {
            1.0
        };

        for py in min_y..=max_y {
            let dy = py as f32 + 0.5 - dab.y;

            for px in min_x..=max_x {
                let dx = px as f32 + 0.5 - dab.x;

                let dist_sq = if is_elliptical {
                    let rx = dx * cos_a + dy * sin_a;
                    let ry = (-dx * sin_a + dy * cos_a) * inv_aspect;
                    rx * rx + ry * ry
                } else {
                    dx * dx + dy * dy
                };

                if dist_sq > radius_sq {
                    continue;
                }

                // Falloff factor
                let falloff = if dist_sq <= inner_sq {
                    1.0
                } else {
                    let dist = dist_sq.sqrt();
                    let t = (dist - inner_radius) / (radius - inner_radius);
                    // Smoothstep falloff
                    (1.0 - t * t * (3.0 - 2.0 * t)).clamp(0.0, 1.0)
                };

                let erase_amount = (dab.opacity * falloff).clamp(0.0, 1.0);
                if erase_amount <= 0.001 {
                    continue;
                }

                let idx = ((py * layer.width + px) * 4) as usize;
                let cur_a = layer.pixels[idx + 3] as f32 / 255.0;
                let new_a = (cur_a * (1.0 - erase_amount)).clamp(0.0, 1.0);
                let new_a_u8 = (new_a * 255.0).round() as u8;
                layer.pixels[idx + 3] = new_a_u8;
                if new_a_u8 == 0 {
                    layer.pixels[idx] = 0;
                    layer.pixels[idx + 1] = 0;
                    layer.pixels[idx + 2] = 0;
                }
            }
        }
    }
}

