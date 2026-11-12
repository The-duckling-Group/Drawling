#[derive(Debug, Clone, Copy)]
pub struct CanvasViewport {
    pub screen_width: f32,
    pub screen_height: f32,
    pub canvas_width: f32,
    pub canvas_height: f32,
    pub pan_x: f32,
    pub pan_y: f32,
    pub zoom: f32, // 1.0 = 100%
    pub rotation_rad: f32,
}

impl CanvasViewport {
    pub fn new(screen_width: f32, screen_height: f32, canvas_width: f32, canvas_height: f32) -> Self {
        // Fit canvas centrally on screen with margin
        let zoom_x = (screen_width * 0.85) / canvas_width;
        let zoom_y = (screen_height * 0.85) / canvas_height;
        let initial_zoom = zoom_x.min(zoom_y).clamp(0.1, 5.0);

        let pan_x = (screen_width - canvas_width * initial_zoom) * 0.5;
        let pan_y = (screen_height - canvas_height * initial_zoom) * 0.5;

        Self {
            screen_width,
            screen_height,
            canvas_width,
            canvas_height,
            pan_x,
            pan_y,
            zoom: initial_zoom,
            rotation_rad: 0.0,
        }
    }

    /// Zoom in or out around an anchor screen coordinate
    pub fn zoom_around(&mut self, factor: f32, anchor_screen_x: f32, anchor_screen_y: f32) {
        let old_zoom = self.zoom;
        let new_zoom = (self.zoom * factor).clamp(0.05, 50.0);
        let ratio = new_zoom / old_zoom;

        self.pan_x = anchor_screen_x - (anchor_screen_x - self.pan_x) * ratio;
        self.pan_y = anchor_screen_y - (anchor_screen_y - self.pan_y) * ratio;
        self.zoom = new_zoom;
    }

    pub fn pan_by(&mut self, dx: f32, dy: f32) {
        self.pan_x += dx;
        self.pan_y += dy;
    }

    pub fn reset_view(&mut self) {
        let zoom_x = (self.screen_width * 0.85) / self.canvas_width;
        let zoom_y = (self.screen_height * 0.85) / self.canvas_height;
        self.zoom = zoom_x.min(zoom_y).clamp(0.1, 5.0);
        self.pan_x = (self.screen_width - self.canvas_width * self.zoom) * 0.5;
        self.pan_y = (self.screen_height - self.canvas_height * self.zoom) * 0.5;
        self.rotation_rad = 0.0;
    }

    /// Convert screen coordinate (e.g. mouse cursor) to canvas pixel coordinate
    pub fn screen_to_canvas(&self, screen_x: f32, screen_y: f32) -> (f32, f32) {
        let rel_x = (screen_x - self.pan_x) / self.zoom;
        let rel_y = (screen_y - self.pan_y) / self.zoom;

        if self.rotation_rad.abs() > 1e-5 {
            let cos = (-self.rotation_rad).cos();
            let sin = (-self.rotation_rad).sin();
            let cx = self.canvas_width * 0.5;
            let cy = self.canvas_height * 0.5;
            let ox = rel_x - cx;
            let oy = rel_y - cy;
            (ox * cos - oy * sin + cx, ox * sin + oy * cos + cy)
        } else {
            (rel_x, rel_y)
        }
    }

    /// Convert canvas pixel coordinate to screen coordinate
    pub fn canvas_to_screen(&self, canvas_x: f32, canvas_y: f32) -> (f32, f32) {
        let (rx, ry) = if self.rotation_rad.abs() > 1e-5 {
            let cos = self.rotation_rad.cos();
            let sin = self.rotation_rad.sin();
            let cx = self.canvas_width * 0.5;
            let cy = self.canvas_height * 0.5;
            let ox = canvas_x - cx;
            let oy = canvas_y - cy;
            (ox * cos - oy * sin + cx, ox * sin + oy * cos + cy)
        } else {
            (canvas_x, canvas_y)
        };

        (rx * self.zoom + self.pan_x, ry * self.zoom + self.pan_y)
    }

    /// Generates 4x4 orthographic projection view matrix for WGPU uniforms
    pub fn to_wgpu_matrix(&self) -> [f32; 16] {
        // Orthographic screen mapping to [-1, 1] NDC
        let sx = 2.0 / self.screen_width;
        let sy = -2.0 / self.screen_height; // Flip Y for graphics API
        let tx = -1.0 + self.pan_x * sx;
        let ty = 1.0 + self.pan_y * sy;

        [
            sx * self.zoom, 0.0, 0.0, 0.0,
            0.0, sy * self.zoom, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            tx, ty, 0.0, 1.0,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_coordinate_roundtrip() {
        let vp = CanvasViewport::new(1920.0, 1080.0, 800.0, 600.0);

        let canvas_pt = (350.0, 220.0);
        let (sx, sy) = vp.canvas_to_screen(canvas_pt.0, canvas_pt.1);
        let (cx, cy) = vp.screen_to_canvas(sx, sy);

        assert!((cx - canvas_pt.0).abs() < 1e-4);
        assert!((cy - canvas_pt.1).abs() < 1e-4);
    }

    #[test]
    fn test_viewport_zoom_around_anchor() {
        let mut vp = CanvasViewport::new(1000.0, 1000.0, 500.0, 500.0);
        let anchor = (500.0, 500.0);

        let (c_before_x, c_before_y) = vp.screen_to_canvas(anchor.0, anchor.1);
        vp.zoom_around(2.0, anchor.0, anchor.1);
        let (c_after_x, c_after_y) = vp.screen_to_canvas(anchor.0, anchor.1);

        // The point directly under the zoom anchor must not change canvas coordinate
        assert!((c_before_x - c_after_x).abs() < 1e-4);
        assert!((c_before_y - c_after_y).abs() < 1e-4);
    }
}
