use crate::color::Color;
use crate::layer::Layer;

pub struct ShapeDrawer;

impl ShapeDrawer {
    /// Draws a straight line with line width from (x0, y0) to (x1, y1).
    pub fn draw_line(layer: &mut Layer, x0: f32, y0: f32, x1: f32, y1: f32, color: Color, width: f32) {
        if layer.locked {
            return;
        }

        let dx = x1 - x0;
        let dy = y1 - y0;
        let len = (dx * dx + dy * dy).sqrt();
        let steps = (len * 2.0).ceil() as usize;
        let radius = (width * 0.5).max(0.5);

        for s in 0..=steps {
            let t = if steps == 0 { 0.0 } else { s as f32 / steps as f32 };
            let cx = x0 + dx * t;
            let cy = y0 + dy * t;
            Self::draw_filled_circle(layer, cx, cy, radius, color);
        }
    }

    /// Draws a rectangle. If `stroke_width` is Some(w), draws outline; if None, fills.
    pub fn draw_rect(
        layer: &mut Layer,
        x: i32,
        y: i32,
        w: u32,
        h: u32,
        color: Color,
        stroke_width: Option<f32>,
    ) {
        if layer.locked || w == 0 || h == 0 {
            return;
        }

        match stroke_width {
            None => {
                let x_end = (x + w as i32).min(layer.width as i32);
                let y_end = (y + h as i32).min(layer.height as i32);
                let x_start = x.max(0);
                let y_start = y.max(0);

                for py in y_start..y_end {
                    for px in x_start..x_end {
                        layer.set_color(px as u32, py as u32, color);
                    }
                }
            }
            Some(sw) => {
                let fx = x as f32;
                let fy = y as f32;
                let fw = w as f32;
                let fh = h as f32;

                Self::draw_line(layer, fx, fy, fx + fw, fy, color, sw);
                Self::draw_line(layer, fx + fw, fy, fx + fw, fy + fh, color, sw);
                Self::draw_line(layer, fx + fw, fy + fh, fx, fy + fh, color, sw);
                Self::draw_line(layer, fx, fy + fh, fx, fy, color, sw);
            }
        }
    }

    /// Draws an ellipse. If `stroke_width` is Some(w), draws outline; if None, fills.
    pub fn draw_ellipse(
        layer: &mut Layer,
        cx: f32,
        cy: f32,
        rx: f32,
        ry: f32,
        color: Color,
        stroke_width: Option<f32>,
    ) {
        if layer.locked || rx <= 0.0 || ry <= 0.0 {
            return;
        }

        match stroke_width {
            None => {
                let min_x = (cx - rx).floor().max(0.0) as u32;
                let max_x = (cx + rx).ceil().min((layer.width - 1) as f32) as u32;
                let min_y = (cy - ry).floor().max(0.0) as u32;
                let max_y = (cy + ry).ceil().min((layer.height - 1) as f32) as u32;

                let inv_rx2 = 1.0 / (rx * rx);
                let inv_ry2 = 1.0 / (ry * ry);

                for py in min_y..=max_y {
                    for px in min_x..=max_x {
                        let dx = px as f32 - cx;
                        let dy = py as f32 - cy;
                        if (dx * dx * inv_rx2 + dy * dy * inv_ry2) <= 1.0 {
                            layer.set_color(px, py, color);
                        }
                    }
                }
            }
            Some(sw) => {
                let perimeter = std::f32::consts::PI * (3.0 * (rx + ry) - ((3.0 * rx + ry) * (rx + 3.0 * ry)).sqrt());
                let steps = (perimeter * 2.0).ceil().max(24.0) as usize;

                let mut prev_x = cx + rx;
                let mut prev_y = cy;

                for s in 1..=steps {
                    let theta = (s as f32 / steps as f32) * std::f32::consts::TAU;
                    let curr_x = cx + rx * theta.cos();
                    let curr_y = cy + ry * theta.sin();
                    Self::draw_line(layer, prev_x, prev_y, curr_x, curr_y, color, sw);
                    prev_x = curr_x;
                    prev_y = curr_y;
                }
            }
        }
    }

    /// Draws a regular star with `points` rays.
    pub fn draw_star(
        layer: &mut Layer,
        cx: f32,
        cy: f32,
        points: usize,
        inner_r: f32,
        outer_r: f32,
        color: Color,
        stroke_width: f32,
    ) {
        if points < 3 || layer.locked {
            return;
        }

        let num_vertices = points * 2;
        let mut vertices = Vec::with_capacity(num_vertices);

        for i in 0..num_vertices {
            let r = if i % 2 == 0 { outer_r } else { inner_r };
            let angle = (i as f32 / num_vertices as f32) * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
            vertices.push((cx + r * angle.cos(), cy + r * angle.sin()));
        }

        for i in 0..num_vertices {
            let (x0, y0) = vertices[i];
            let (x1, y1) = vertices[(i + 1) % num_vertices];
            Self::draw_line(layer, x0, y0, x1, y1, color, stroke_width);
        }
    }

    fn draw_filled_circle(layer: &mut Layer, cx: f32, cy: f32, r: f32, color: Color) {
        let min_x = (cx - r).floor().max(0.0) as u32;
        let max_x = (cx + r).ceil().min((layer.width - 1) as f32) as u32;
        let min_y = (cy - r).floor().max(0.0) as u32;
        let max_y = (cy + r).ceil().min((layer.height - 1) as f32) as u32;
        let r2 = r * r;

        for py in min_y..=max_y {
            for px in min_x..=max_x {
                let dx = px as f32 - cx;
                let dy = py as f32 - cy;
                if dx * dx + dy * dy <= r2 {
                    layer.set_color(px, py, color);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_rect_and_ellipse() {
        let mut layer = Layer::new("Shapes Test", 30, 30);

        // Filled rectangle
        ShapeDrawer::draw_rect(&mut layer, 5, 5, 10, 10, Color::RED, None);
        assert_eq!(layer.get_color(6, 6), Some(Color::RED));
        assert_eq!(layer.get_color(20, 20), Some(Color::TRANSPARENT));

        // Filled ellipse
        ShapeDrawer::draw_ellipse(&mut layer, 20.0, 20.0, 5.0, 5.0, Color::BLUE, None);
        assert_eq!(layer.get_color(20, 20), Some(Color::BLUE));

        // Star
        ShapeDrawer::draw_star(&mut layer, 15.0, 15.0, 5, 3.0, 8.0, Color::WHITE, 1.0);
    }
}
