use super::engine::BrushDab;
use crate::layer::Layer;

pub struct BrushTipRasterizer;

impl BrushTipRasterizer {
    /// Render a single brush dab onto a target layer
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

        for py in min_y..=max_y {
            let dy = py as f32 + 0.5 - dab.y;
            let dy_sq = dy * dy;

            for px in min_x..=max_x {
                let dx = px as f32 + 0.5 - dab.x;
                let dist_sq = dx * dx + dy_sq;

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
}
