use crate::color::Color;
use crate::layer::{Layer, LayerStack};

pub struct ColorPicker;

impl ColorPicker {
    /// Sample color from a layer at (x, y) with a sampling radius.
    /// radius 0 = 1x1 pixel, radius 1 = 3x3 window, radius 2 = 5x5 window.
    pub fn sample_layer(layer: &Layer, x: i32, y: i32, radius: u32) -> Color {
        let width = layer.width as i32;
        let height = layer.height as i32;
        let r = radius as i32;

        let mut sum_r: f32 = 0.0;
        let mut sum_g: f32 = 0.0;
        let mut sum_b: f32 = 0.0;
        let mut sum_a: f32 = 0.0;
        let mut count: f32 = 0.0;

        for dy in -r..=r {
            for dx in -r..=r {
                let px = x + dx;
                let py = y + dy;

                if px >= 0 && px < width && py >= 0 && py < height {
                    if let Some(c) = layer.get_color(px as u32, py as u32) {
                        sum_r += c.r * c.a;
                        sum_g += c.g * c.a;
                        sum_b += c.b * c.a;
                        sum_a += c.a;
                        count += 1.0;
                    }
                }
            }
        }

        if count == 0.0 || sum_a == 0.0 {
            Color::TRANSPARENT
        } else {
            Color::new(
                (sum_r / sum_a).clamp(0.0, 1.0),
                (sum_g / sum_a).clamp(0.0, 1.0),
                (sum_b / sum_a).clamp(0.0, 1.0),
                (sum_a / count).clamp(0.0, 1.0),
            )
        }
    }

    /// Sample composited color from layer stack at (x, y).
    pub fn sample_stack(stack: &LayerStack, x: i32, y: i32, radius: u32) -> Color {
        let width = stack.width as i32;
        let height = stack.height as i32;
        let r = radius as i32;

        let mut sum_r: f32 = 0.0;
        let mut sum_g: f32 = 0.0;
        let mut sum_b: f32 = 0.0;
        let mut sum_a: f32 = 0.0;
        let mut count: f32 = 0.0;

        for dy in -r..=r {
            for dx in -r..=r {
                let px = x + dx;
                let py = y + dy;

                if px >= 0 && px < width && py >= 0 && py < height {
                    let mut comp: [u8; 4] = [0, 0, 0, 0];
                    for layer in stack.layers.iter().filter(|l| l.visible) {
                        if let Some(c) = layer.get_pixel(px as u32, py as u32) {
                            comp = layer.blend_mode.composite_pixel(comp, c, layer.opacity, false);
                        }
                    }
                    sum_r += comp[0] as f32 / 255.0;
                    sum_g += comp[1] as f32 / 255.0;
                    sum_b += comp[2] as f32 / 255.0;
                    sum_a += comp[3] as f32 / 255.0;
                    count += 1.0;
                }
            }
        }

        if count == 0.0 {
            Color::TRANSPARENT
        } else {
            Color::new(
                (sum_r / count).clamp(0.0, 1.0),
                (sum_g / count).clamp(0.0, 1.0),
                (sum_b / count).clamp(0.0, 1.0),
                (sum_a / count).clamp(0.0, 1.0),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_picker_sample() {
        let mut layer = Layer::new("Picker Layer", 10, 10);
        layer.fill_color(Color::RED);
        layer.set_color(5, 5, Color::BLUE);

        // 1x1 point sample at (5, 5)
        let exact = ColorPicker::sample_layer(&layer, 5, 5, 0);
        assert_eq!(exact, Color::BLUE);

        // 3x3 average sample at (5, 5) should blend blue and red
        let avg = ColorPicker::sample_layer(&layer, 5, 5, 1);
        assert!(avg.r > 0.0 && avg.b > 0.0);
    }
}
