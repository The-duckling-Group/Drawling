use crate::color::Color;
use crate::layer::Layer;

pub struct SmudgeEngine {
    pub size: f32,
    pub strength: f32, // 0.0 to 1.0 (how much paint is deposited)
    pub bleed: f32,    // 0.0 to 1.0 (how quickly canvas color mixes into brush)
    reservoir: Option<Color>,
}

impl SmudgeEngine {
    pub fn new(size: f32, strength: f32, bleed: f32) -> Self {
        Self {
            size,
            strength: strength.clamp(0.01, 1.0),
            bleed: bleed.clamp(0.01, 1.0),
            reservoir: None,
        }
    }

    /// Reset internal paint reservoir on pen down
    pub fn begin_stroke(&mut self, layer: &Layer, x: f32, y: f32) {
        let ix = x.round() as i32;
        let iy = y.round() as i32;
        if ix >= 0 && ix < layer.width as i32 && iy >= 0 && iy < layer.height as i32 {
            self.reservoir = layer.get_color(ix as u32, iy as u32);
        } else {
            self.reservoir = Some(Color::TRANSPARENT);
        }
    }

    /// Smudge along a segment from (x0, y0) to (x1, y1)
    pub fn smudge_to(&mut self, layer: &mut Layer, x0: f32, y0: f32, x1: f32, y1: f32) {
        if layer.locked {
            return;
        }

        let dx = x1 - x0;
        let dy = y1 - y0;
        let dist = (dx * dx + dy * dy).sqrt();
        let step_size = (self.size * 0.25).max(1.0);
        let steps = (dist / step_size).ceil() as usize;

        if steps == 0 {
            self.apply_dab(layer, x1, y1);
            return;
        }

        for s in 1..=steps {
            let t = s as f32 / steps as f32;
            let cx = x0 + dx * t;
            let cy = y0 + dy * t;
            self.apply_dab(layer, cx, cy);
        }
    }

    fn apply_dab(&mut self, layer: &mut Layer, cx: f32, cy: f32) {
        let radius = self.size * 0.5;
        let min_x = (cx - radius).floor().max(0.0) as u32;
        let max_x = (cx + radius).ceil().min((layer.width - 1) as f32) as u32;
        let min_y = (cy - radius).floor().max(0.0) as u32;
        let max_y = (cy + radius).ceil().min((layer.height - 1) as f32) as u32;

        let mut sampled_colors: Vec<Color> = Vec::new();

        let res_color = self.reservoir.unwrap_or(Color::TRANSPARENT);

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                let d = (dx * dx + dy * dy).sqrt();

                if d <= radius {
                    let falloff = 1.0 - (d / radius);
                    let blend_factor = falloff * self.strength;

                    if let Some(current) = layer.get_color(x, y) {
                        sampled_colors.push(current);

                        // Blend reservoir onto layer
                        let mixed = Color::new(
                            current.r * (1.0 - blend_factor) + res_color.r * blend_factor,
                            current.g * (1.0 - blend_factor) + res_color.g * blend_factor,
                            current.b * (1.0 - blend_factor) + res_color.b * blend_factor,
                            current.a * (1.0 - blend_factor) + res_color.a * blend_factor,
                        );

                        layer.set_color(x, y, mixed);
                    }
                }
            }
        }

        // Update reservoir with canvas paint
        if !sampled_colors.is_empty() {
            let mut avg_r = 0.0;
            let mut avg_g = 0.0;
            let mut avg_b = 0.0;
            let mut avg_a = 0.0;
            let count = sampled_colors.len() as f32;

            for c in &sampled_colors {
                avg_r += c.r;
                avg_g += c.g;
                avg_b += c.b;
                avg_a += c.a;
            }

            let canvas_avg = Color::new(avg_r / count, avg_g / count, avg_b / count, avg_a / count);

            let new_res = Color::new(
                res_color.r * (1.0 - self.bleed) + canvas_avg.r * self.bleed,
                res_color.g * (1.0 - self.bleed) + canvas_avg.g * self.bleed,
                res_color.b * (1.0 - self.bleed) + canvas_avg.b * self.bleed,
                res_color.a * (1.0 - self.bleed) + canvas_avg.a * self.bleed,
            );
            self.reservoir = Some(new_res);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smudge_stroke_drags_color() {
        let mut layer = Layer::new("Smudge Layer", 20, 20);
        layer.fill_color(Color::WHITE);

        // Put a red blob at (5, 10)
        layer.set_color(5, 10, Color::RED);

        let mut smudge = SmudgeEngine::new(4.0, 0.8, 0.5);
        smudge.begin_stroke(&layer, 5.0, 10.0);
        smudge.smudge_to(&mut layer, 5.0, 10.0, 15.0, 10.0);

        // Pixel at (10, 10) along smudge stroke should now have some red
        let mid = layer.get_color(10, 10).unwrap();
        assert!(mid.g < 1.0 || mid.b < 1.0, "Smudge did not drag color: {:?}", mid);
    }
}
