use crate::layer::Layer;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshWarpGrid {
    pub rows: usize,
    pub cols: usize,
    pub width: f32,
    pub height: f32,
    /// Current deformed control points, length = rows * cols
    pub points: Vec<(f32, f32)>,
    /// Initial undeformed control points
    pub base_points: Vec<(f32, f32)>,
}

impl MeshWarpGrid {
    pub fn new(rows: usize, cols: usize, width: f32, height: f32) -> Self {
        let r = rows.max(2);
        let c = cols.max(2);
        let mut points = Vec::with_capacity(r * c);

        for row in 0..r {
            let v = row as f32 / (r - 1) as f32;
            let y = v * height;
            for col in 0..c {
                let u = col as f32 / (c - 1) as f32;
                let x = u * width;
                points.push((x, y));
            }
        }

        Self {
            rows: r,
            cols: c,
            width,
            height,
            base_points: points.clone(),
            points,
        }
    }

    pub fn get_point(&self, row: usize, col: usize) -> Option<(f32, f32)> {
        if row < self.rows && col < self.cols {
            Some(self.points[row * self.cols + col])
        } else {
            None
        }
    }

    pub fn set_point(&mut self, row: usize, col: usize, point: (f32, f32)) {
        if row < self.rows && col < self.cols {
            self.points[row * self.cols + col] = point;
        }
    }

    pub fn reset(&mut self) {
        self.points = self.base_points.clone();
    }

    /// Evaluates forward mapping for a normalized (u, v) in [0, 1] using bilinear patch interpolation
    pub fn map_uv(&self, u: f32, v: f32) -> (f32, f32) {
        let u_clamped = u.clamp(0.0, 1.0);
        let v_clamped = v.clamp(0.0, 1.0);

        let col_f = u_clamped * (self.cols - 1) as f32;
        let row_f = v_clamped * (self.rows - 1) as f32;

        let c0 = (col_f.floor() as usize).min(self.cols - 2);
        let c1 = c0 + 1;
        let r0 = (row_f.floor() as usize).min(self.rows - 2);
        let r1 = r0 + 1;

        let fu = col_f - c0 as f32;
        let fv = row_f - r0 as f32;

        let p00 = self.points[r0 * self.cols + c0];
        let p10 = self.points[r0 * self.cols + c1];
        let p01 = self.points[r1 * self.cols + c0];
        let p11 = self.points[r1 * self.cols + c1];

        let top_x = p00.0 * (1.0 - fu) + p10.0 * fu;
        let top_y = p00.1 * (1.0 - fu) + p10.1 * fu;
        let bot_x = p01.0 * (1.0 - fu) + p11.0 * fu;
        let bot_y = p01.1 * (1.0 - fu) + p11.1 * fu;

        (
            top_x * (1.0 - fv) + bot_x * fv,
            top_y * (1.0 - fv) + bot_y * fv,
        )
    }

    /// Deforms the target layer using this mesh grid
    pub fn warp_layer(&self, layer: &mut Layer) {
        if layer.locked {
            return;
        }

        let orig = layer.clone_data();
        layer.clear();

        let w = layer.width;
        let h = layer.height;

        // Forward supersampling patch rasterization
        let steps_x = (w * 2) as usize;
        let steps_y = (h * 2) as usize;

        for sy in 0..steps_y {
            let v = sy as f32 / steps_y as f32;
            let src_y = (v * (h - 1) as f32).round() as u32;

            for sx in 0..steps_x {
                let u = sx as f32 / steps_x as f32;
                let src_x = (u * (w - 1) as f32).round() as u32;

                if let Some(src_px) = orig.get_pixel(src_x, src_y) {
                    if src_px[3] == 0 {
                        continue;
                    }
                    let (dx, dy) = self.map_uv(u, v);
                    let target_x = dx.round() as i32;
                    let target_y = dy.round() as i32;

                    if target_x >= 0 && target_x < w as i32 && target_y >= 0 && target_y < h as i32 {
                        layer.set_pixel(target_x as u32, target_y as u32, src_px);
                    }
                }
            }
        }
    }
}

/// Perspective Grid with Vanishing Point Snapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerspectiveGrid {
    pub horizon_y: f32,
    pub vanishing_points: Vec<(f32, f32)>,
    pub visible: bool,
    pub snap_threshold_px: f32,
}

impl Default for PerspectiveGrid {
    fn default() -> Self {
        Self::two_point(300.0, -400.0, 1600.0)
    }
}

impl PerspectiveGrid {
    pub fn one_point(horizon_y: f32, vp_x: f32) -> Self {
        Self {
            horizon_y,
            vanishing_points: vec![(vp_x, horizon_y)],
            visible: true,
            snap_threshold_px: 12.0,
        }
    }

    pub fn two_point(horizon_y: f32, left_vp_x: f32, right_vp_x: f32) -> Self {
        Self {
            horizon_y,
            vanishing_points: vec![(left_vp_x, horizon_y), (right_vp_x, horizon_y)],
            visible: true,
            snap_threshold_px: 12.0,
        }
    }

    pub fn three_point(horizon_y: f32, left_vp_x: f32, right_vp_x: f32, vertical_vp: (f32, f32)) -> Self {
        Self {
            horizon_y,
            vanishing_points: vec![(left_vp_x, horizon_y), (right_vp_x, horizon_y), vertical_vp],
            visible: true,
            snap_threshold_px: 12.0,
        }
    }

    /// Snaps an input point to the closest perspective ray radiating from any vanishing point
    pub fn snap_point(&self, pt: (f32, f32)) -> (f32, f32) {
        if self.vanishing_points.is_empty() {
            return pt;
        }

        let mut best_pt = pt;
        let mut min_dist = self.snap_threshold_px;
        let angle_step = std::f32::consts::PI / 18.0; // Ray every 10 degrees

        for &vp in &self.vanishing_points {
            let dx = pt.0 - vp.0;
            let dy = pt.1 - vp.1;
            let ray_len = (dx * dx + dy * dy).sqrt();
            if ray_len > 0.001 {
                let current_angle = dy.atan2(dx);
                // Nearest guide ray angle
                let snapped_angle = (current_angle / angle_step).round() * angle_step;

                let proj_x = vp.0 + snapped_angle.cos() * ray_len;
                let proj_y = vp.1 + snapped_angle.sin() * ray_len;

                let dist = ((proj_x - pt.0).powi(2) + (proj_y - pt.1).powi(2)).sqrt();
                if dist < min_dist {
                    min_dist = dist;
                    best_pt = (proj_x, proj_y);
                }
            }
        }

        best_pt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_grid_interpolation() {
        let mut grid = MeshWarpGrid::new(3, 3, 100.0, 100.0);
        let center = grid.map_uv(0.5, 0.5);
        assert!((center.0 - 50.0).abs() < 1e-4);
        assert!((center.1 - 50.0).abs() < 1e-4);

        // Deform center point
        grid.set_point(1, 1, (75.0, 75.0));
        let deformed_center = grid.map_uv(0.5, 0.5);
        assert!((deformed_center.0 - 75.0).abs() < 1e-4);
        assert!((deformed_center.1 - 75.0).abs() < 1e-4);
    }

    #[test]
    fn test_perspective_snapping() {
        let grid = PerspectiveGrid::one_point(200.0, 500.0);
        // Point close to the ray from (500, 200) to (500, 400)
        let pt = (502.0, 300.0);
        let snapped = grid.snap_point(pt);
        assert!((snapped.0 - 500.0).abs() < 1e-3);
    }
}
