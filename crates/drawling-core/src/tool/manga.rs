use crate::color::Color;
use crate::layer::Layer;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MangaPanel {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameDivider {
    pub margin: u32,
    pub gutter: u32,
    pub border_width: u32,
    pub border_color: Color,
}

impl Default for FrameDivider {
    fn default() -> Self {
        Self {
            margin: 40,
            gutter: 20,
            border_width: 4,
            border_color: Color::BLACK,
        }
    }
}

impl FrameDivider {
    pub fn new(margin: u32, gutter: u32, border_width: u32) -> Self {
        Self {
            margin,
            gutter,
            border_width,
            border_color: Color::BLACK,
        }
    }

    /// Subdivides the canvas area into a grid of manga panels (rows x cols)
    pub fn generate_grid(&self, canvas_w: u32, canvas_h: u32, rows: u32, cols: u32) -> Vec<MangaPanel> {
        let rows = rows.max(1);
        let cols = cols.max(1);

        let total_w = canvas_w.saturating_sub(self.margin * 2);
        let total_h = canvas_h.saturating_sub(self.margin * 2);

        let gutter_w = (cols - 1) * self.gutter;
        let gutter_h = (rows - 1) * self.gutter;

        let panel_w = total_w.saturating_sub(gutter_w) / cols;
        let panel_h = total_h.saturating_sub(gutter_h) / rows;

        let mut panels = Vec::new();
        for r in 0..rows {
            for c in 0..cols {
                let px = self.margin + c * (panel_w + self.gutter);
                let py = self.margin + r * (panel_h + self.gutter);
                panels.push(MangaPanel {
                    x: px,
                    y: py,
                    width: panel_w,
                    height: panel_h,
                });
            }
        }
        panels
    }

    /// Renders panel borders onto the layer
    pub fn draw_panels_on_layer(&self, layer: &mut Layer, panels: &[MangaPanel]) {
        let col = self.border_color.to_rgba8();
        let bw = self.border_width;

        for panel in panels {
            let x0 = panel.x;
            let y0 = panel.y;
            let x1 = (panel.x + panel.width).min(layer.width);
            let y1 = (panel.y + panel.height).min(layer.height);

            // Horizontal lines
            for t in 0..bw {
                let yt = y0 + t;
                let yb = y1.saturating_sub(1 + t);
                for x in x0..x1 {
                    layer.set_pixel(x, yt, col);
                    layer.set_pixel(x, yb, col);
                }
            }

            // Vertical lines
            for t in 0..bw {
                let xl = x0 + t;
                let xr = x1.saturating_sub(1 + t);
                for y in y0..y1 {
                    layer.set_pixel(xl, y, col);
                    layer.set_pixel(xr, y, col);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScreentoneStyle {
    Dot20Percent,
    Dot40Percent,
    Dot60Percent,
    Crosshatch,
    MangaLines,
}

pub struct MangaRastertone;

impl MangaRastertone {
    /// Fills layer or area with authentic manga screentone pattern
    pub fn apply_pattern(
        layer: &mut Layer,
        style: ScreentoneStyle,
        color: Color,
        x0: u32,
        y0: u32,
        x1: u32,
        y1: u32,
    ) {
        let x1 = x1.min(layer.width);
        let y1 = y1.min(layer.height);
        let rgba = color.to_rgba8();

        for y in y0..y1 {
            for x in x0..x1 {
                let is_inked = match style {
                    ScreentoneStyle::Dot20Percent => (x % 6 == 0) && (y % 6 == 0),
                    ScreentoneStyle::Dot40Percent => ((x + y) % 4 == 0) || ((x % 4 == 0) && (y % 4 == 0)),
                    ScreentoneStyle::Dot60Percent => (x % 3 != 0) || (y % 3 != 0),
                    ScreentoneStyle::Crosshatch => (x % 5 == 0) || (y % 5 == 0),
                    ScreentoneStyle::MangaLines => (x + y) % 6 == 0,
                };

                if is_inked {
                    layer.set_pixel(x, y, rgba);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpeechBalloonKind {
    Oval,
    RoundedRect,
    ThoughtBubble,
    ActionBurst,
}

pub struct SpeechBalloon;

impl SpeechBalloon {
    /// Draws a manga speech balloon with fill and contour
    pub fn draw(
        layer: &mut Layer,
        kind: SpeechBalloonKind,
        cx: f32,
        cy: f32,
        rx: f32,
        ry: f32,
        fill_color: Color,
        stroke_color: Color,
        stroke_width: f32,
    ) {
        let fill_rgba = fill_color.to_rgba8();
        let stroke_rgba = stroke_color.to_rgba8();

        let min_x = ((cx - rx - stroke_width).floor() as i32).max(0) as u32;
        let max_x = ((cx + rx + stroke_width).ceil() as i32).min(layer.width as i32) as u32;
        let min_y = ((cy - ry - stroke_width).floor() as i32).max(0) as u32;
        let max_y = ((cy + ry + stroke_width).ceil() as i32).min(layer.height as i32) as u32;

        let rx_inner = rx - stroke_width * 0.5;
        let ry_inner = ry - stroke_width * 0.5;

        for y in min_y..max_y {
            let dy = y as f32 + 0.5 - cy;
            for x in min_x..max_x {
                let dx = x as f32 + 0.5 - cx;

                let (inside_outer, inside_inner) = match kind {
                    SpeechBalloonKind::Oval => {
                        let val_out = (dx * dx) / (rx * rx) + (dy * dy) / (ry * ry);
                        let val_in = (dx * dx) / (rx_inner * rx_inner) + (dy * dy) / (ry_inner * ry_inner);
                        (val_out <= 1.0, val_in <= 1.0)
                    }
                    SpeechBalloonKind::RoundedRect => {
                        let in_out = dx.abs() <= rx && dy.abs() <= ry;
                        let in_in = dx.abs() <= rx_inner && dy.abs() <= ry_inner;
                        (in_out, in_in)
                    }
                    SpeechBalloonKind::ThoughtBubble | SpeechBalloonKind::ActionBurst => {
                        let angle = dy.atan2(dx);
                        let bumps = if kind == SpeechBalloonKind::ActionBurst { 12.0 } else { 8.0 };
                        let amp = if kind == SpeechBalloonKind::ActionBurst { 0.22 } else { 0.12 };
                        let wave = 1.0 + amp * (angle * bumps).sin();
                        let d = (dx * dx + dy * dy).sqrt();
                        let rad_out = rx * wave;
                        let rad_in = (rx - stroke_width) * wave;
                        (d <= rad_out, d <= rad_in)
                    }
                };

                if inside_inner {
                    layer.set_pixel(x, y, fill_rgba);
                } else if inside_outer {
                    layer.set_pixel(x, y, stroke_rgba);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextOrientation {
    Horizontal,
    Vertical, // Standard Japanese Manga formatting
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextToolOptions {
    pub text: String,
    pub orientation: TextOrientation,
    pub font_size: f32,
    pub line_spacing: f32,
    pub color: Color,
}

impl Default for TextToolOptions {
    fn default() -> Self {
        Self {
            text: String::new(),
            orientation: TextOrientation::Horizontal,
            font_size: 24.0,
            line_spacing: 1.2,
            color: Color::BLACK,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manga_panel_grid() {
        let divider = FrameDivider::new(20, 10, 2);
        let panels = divider.generate_grid(800, 600, 2, 2);
        assert_eq!(panels.len(), 4);
        assert_eq!(panels[0].x, 20);
        assert_eq!(panels[0].y, 20);
    }

    #[test]
    fn test_manga_screentone_apply() {
        let mut layer = Layer::new("Screen", 100, 100);
        MangaRastertone::apply_pattern(&mut layer, ScreentoneStyle::Dot20Percent, Color::BLACK, 0, 0, 50, 50);
        assert_eq!(layer.get_pixel(0, 0), Some([0, 0, 0, 255]));
    }

    #[test]
    fn test_speech_balloon_draw() {
        let mut layer = Layer::new("Balloon", 200, 200);
        SpeechBalloon::draw(
            &mut layer,
            SpeechBalloonKind::Oval,
            100.0,
            100.0,
            40.0,
            30.0,
            Color::WHITE,
            Color::BLACK,
            2.0,
        );
        // Center should be white fill
        assert_eq!(layer.get_pixel(100, 100), Some([255, 255, 255, 255]));
    }
}
