use crate::layer::Layer;
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub struct SelectionMask {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>, // 0 = unselected, 255 = fully selected
}

impl SelectionMask {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0; (width * height) as usize],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.data.iter().all(|&v| v == 0)
    }

    pub fn clear(&mut self) {
        self.data.fill(0);
    }

    pub fn select_all(&mut self) {
        self.data.fill(255);
    }

    pub fn invert(&mut self) {
        for v in self.data.iter_mut() {
            *v = 255 - *v;
        }
    }

    #[inline]
    pub fn get(&self, x: u32, y: u32) -> u8 {
        if x >= self.width || y >= self.height {
            0
        } else {
            self.data[(y * self.width + x) as usize]
        }
    }

    #[inline]
    pub fn set(&mut self, x: u32, y: u32, val: u8) {
        if x < self.width && y < self.height {
            self.data[(y * self.width + x) as usize] = val;
        }
    }

    /// Select rectangular region
    pub fn select_rect(&mut self, x0: u32, y0: u32, x1: u32, y1: u32) {
        let min_x = x0.min(x1).min(self.width);
        let max_x = x0.max(x1).min(self.width);
        let min_y = y0.min(y1).min(self.height);
        let max_y = y0.max(y1).min(self.height);

        for y in min_y..max_y {
            let row_start = (y * self.width + min_x) as usize;
            let row_end = (y * self.width + max_x) as usize;
            self.data[row_start..row_end].fill(255);
        }
    }

    /// Select elliptical region
    pub fn select_ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32) {
        if rx <= 0.0 || ry <= 0.0 {
            return;
        }
        let min_x = ((cx - rx).floor() as i32).max(0) as u32;
        let max_x = ((cx + rx).ceil() as i32).min(self.width as i32) as u32;
        let min_y = ((cy - ry).floor() as i32).max(0) as u32;
        let max_y = ((cy + ry).ceil() as i32).min(self.height as i32) as u32;

        let inv_rx_sq = 1.0 / (rx * rx);
        let inv_ry_sq = 1.0 / (ry * ry);

        for y in min_y..max_y {
            let dy = y as f32 + 0.5 - cy;
            let dy_term = dy * dy * inv_ry_sq;

            for x in min_x..max_x {
                let dx = x as f32 + 0.5 - cx;
                let val = dx * dx * inv_rx_sq + dy_term;
                if val <= 1.0 {
                    self.set(x, y, 255);
                }
            }
        }
    }

    /// Freehand Lasso selection: point-in-polygon scanline filling
    pub fn select_lasso(&mut self, polygon: &[(f32, f32)]) {
        if polygon.len() < 3 {
            return;
        }

        let mut min_y = self.height as f32;
        let mut max_y = 0.0f32;
        for &(_, y) in polygon {
            if y < min_y { min_y = y; }
            if y > max_y { max_y = y; }
        }

        let start_y = (min_y.floor() as i32).max(0) as u32;
        let end_y = (max_y.ceil() as i32).min(self.height as i32) as u32;

        let mut node_x = Vec::new();

        for y in start_y..end_y {
            node_x.clear();
            let py = y as f32 + 0.5;

            let mut j = polygon.len() - 1;
            for i in 0..polygon.len() {
                let (xi, yi) = polygon[i];
                let (xj, yj) = polygon[j];

                if (yi < py && yj >= py) || (yj < py && yi >= py) {
                    let intersect_x = xi + (py - yi) / (yj - yi) * (xj - xi);
                    node_x.push(intersect_x);
                }
                j = i;
            }

            node_x.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            for chunk in node_x.chunks(2) {
                if chunk.len() == 2 {
                    let x_start = (chunk[0].ceil() as i32).max(0) as u32;
                    let x_end = (chunk[1].floor() as i32).min(self.width as i32) as u32;
                    for x in x_start..x_end {
                        self.set(x, y, 255);
                    }
                }
            }
        }
    }

    /// Paints brush dab into selection mask (for Quick Mask mode)
    pub fn paint_brush_dab(&mut self, cx: f32, cy: f32, radius: f32, value: u8) {
        if radius <= 0.0 {
            return;
        }
        let min_x = ((cx - radius).floor() as i32).max(0) as u32;
        let max_x = ((cx + radius).ceil() as i32).min(self.width as i32) as u32;
        let min_y = ((cy - radius).floor() as i32).max(0) as u32;
        let max_y = ((cy + radius).ceil() as i32).min(self.height as i32) as u32;
        let r_sq = radius * radius;

        for y in min_y..max_y {
            let dy = y as f32 + 0.5 - cy;
            let dy_sq = dy * dy;
            for x in min_x..max_x {
                let dx = x as f32 + 0.5 - cx;
                if dx * dx + dy_sq <= r_sq {
                    self.set(x, y, value);
                }
            }
        }
    }

    /// Magic Wand flood-fill selection with color tolerance
    /// `seed_x`, `seed_y`: starting coordinate
    /// `tolerance`: Euclidean distance threshold in RGB space [0.0, 441.67] (or 0..255)
    pub fn select_magic_wand(
        &mut self,
        layer: &Layer,
        seed_x: u32,
        seed_y: u32,
        tolerance: f32,
        contiguous: bool,
    ) {
        let target_pixel = match layer.get_pixel(seed_x, seed_y) {
            Some(px) => px,
            None => return,
        };

        let tol_sq = tolerance * tolerance;

        let color_dist_sq = |p1: [u8; 4], p2: [u8; 4]| -> f32 {
            let dr = p1[0] as f32 - p2[0] as f32;
            let dg = p1[1] as f32 - p2[1] as f32;
            let db = p1[2] as f32 - p2[2] as f32;
            let da = p1[3] as f32 - p2[3] as f32;
            dr * dr + dg * dg + db * db + da * da
        };

        if !contiguous {
            // Select all matching pixels across the whole canvas
            for y in 0..self.height {
                for x in 0..self.width {
                    if let Some(px) = layer.get_pixel(x, y) {
                        if color_dist_sq(px, target_pixel) <= tol_sq {
                            self.set(x, y, 255);
                        }
                    }
                }
            }
            return;
        }

        // Contiguous BFS flood fill
        let mut visited = vec![false; (self.width * self.height) as usize];
        let mut queue = VecDeque::new();

        let seed_idx = (seed_y * self.width + seed_x) as usize;
        visited[seed_idx] = true;
        queue.push_back((seed_x, seed_y));

        while let Some((cx, cy)) = queue.pop_front() {
            self.set(cx, cy, 255);

            let neighbors = [
                (cx.wrapping_sub(1), cy),
                (cx + 1, cy),
                (cx, cy.wrapping_sub(1)),
                (cx, cy + 1),
            ];

            for (nx, ny) in neighbors {
                if nx < self.width && ny < self.height {
                    let n_idx = (ny * self.width + nx) as usize;
                    if !visited[n_idx] {
                        visited[n_idx] = true;
                        if let Some(n_px) = layer.get_pixel(nx, ny) {
                            if color_dist_sq(n_px, target_pixel) <= tol_sq {
                                queue.push_back((nx, ny));
                            }
                        }
                    }
                }
            }
        }
    }

    /// Feather selection mask by radius (separable box blur on selection alpha)
    pub fn feather(&mut self, radius: u32) {
        if radius == 0 {
            return;
        }
        let r = radius as i32;
        let mut temp = vec![0u8; self.data.len()];

        // Horizontal pass
        for y in 0..self.height {
            for x in 0..self.width {
                let mut sum = 0u32;
                let mut count = 0u32;
                for dx in -r..=r {
                    let nx = x as i32 + dx;
                    if nx >= 0 && nx < self.width as i32 {
                        sum += self.data[(y * self.width + nx as u32) as usize] as u32;
                        count += 1;
                    }
                }
                temp[(y * self.width + x) as usize] = (sum / count) as u8;
            }
        }

        // Vertical pass
        for y in 0..self.height {
            for x in 0..self.width {
                let mut sum = 0u32;
                let mut count = 0u32;
                for dy in -r..=r {
                    let ny = y as i32 + dy;
                    if ny >= 0 && ny < self.height as i32 {
                        sum += temp[(ny as u32 * self.width + x) as usize] as u32;
                        count += 1;
                    }
                }
                self.data[(y * self.width + x) as usize] = (sum / count) as u8;
            }
        }
    }

    /// Union with another selection mask (A = A ∪ B)
    pub fn union_with(&mut self, other: &SelectionMask) {
        assert_eq!(self.width, other.width);
        assert_eq!(self.height, other.height);
        for (a, b) in self.data.iter_mut().zip(other.data.iter()) {
            *a = (*a).max(*b);
        }
    }

    /// Subtract another selection mask (A = A \ B)
    pub fn subtract(&mut self, other: &SelectionMask) {
        assert_eq!(self.width, other.width);
        assert_eq!(self.height, other.height);
        for (a, b) in self.data.iter_mut().zip(other.data.iter()) {
            *a = a.saturating_sub(*b);
        }
    }

    /// Intersect with another selection mask (A = A ∩ B)
    pub fn intersect_with(&mut self, other: &SelectionMask) {
        assert_eq!(self.width, other.width);
        assert_eq!(self.height, other.height);
        for (a, b) in self.data.iter_mut().zip(other.data.iter()) {
            *a = (*a).min(*b);
        }
    }

    /// Expand (dilate) the selection mask by radius
    pub fn expand(&mut self, radius: u32) {
        if radius == 0 {
            return;
        }
        let r = radius as i32;
        let mut temp = self.data.clone();

        for y in 0..self.height {
            for x in 0..self.width {
                if self.get(x, y) > 0 {
                    let min_x = (x as i32 - r).max(0) as u32;
                    let max_x = (x as i32 + r).min(self.width as i32 - 1) as u32;
                    let min_y = (y as i32 - r).max(0) as u32;
                    let max_y = (y as i32 + r).min(self.height as i32 - 1) as u32;

                    for ny in min_y..=max_y {
                        let dy = ny as i32 - y as i32;
                        for nx in min_x..=max_x {
                            let dx = nx as i32 - x as i32;
                            if dx * dx + dy * dy <= r * r {
                                temp[(ny * self.width + nx) as usize] = 255;
                            }
                        }
                    }
                }
            }
        }
        self.data = temp;
    }

    /// Shrink (erode) the selection mask by radius
    pub fn shrink(&mut self, radius: u32) {
        if radius == 0 {
            return;
        }
        let r = radius as i32;
        let mut temp = self.data.clone();

        for y in 0..self.height {
            for x in 0..self.width {
                if self.get(x, y) > 0 {
                    let min_x = (x as i32 - r).max(0) as u32;
                    let max_x = (x as i32 + r).min(self.width as i32 - 1) as u32;
                    let min_y = (y as i32 - r).max(0) as u32;
                    let max_y = (y as i32 + r).min(self.height as i32 - 1) as u32;

                    let mut all_selected = true;
                    'check: for ny in min_y..=max_y {
                        let dy = ny as i32 - y as i32;
                        for nx in min_x..=max_x {
                            let dx = nx as i32 - x as i32;
                            if dx * dx + dy * dy <= r * r && self.get(nx, ny) == 0 {
                                all_selected = false;
                                break 'check;
                            }
                        }
                    }
                    if !all_selected {
                        temp[(y * self.width + x) as usize] = 0;
                    }
                }
            }
        }
        self.data = temp;
    }

    /// Returns non-zero selection bounding box (min_x, min_y, max_x, max_y)
    pub fn bounding_box(&self) -> Option<(u32, u32, u32, u32)> {
        let mut min_x = self.width;
        let mut max_x = 0;
        let mut min_y = self.height;
        let mut max_y = 0;
        let mut found = false;

        for y in 0..self.height {
            for x in 0..self.width {
                if self.get(x, y) > 0 {
                    found = true;
                    if x < min_x { min_x = x; }
                    if x > max_x { max_x = x; }
                    if y < min_y { min_y = y; }
                    if y > max_y { max_y = y; }
                }
            }
        }

        if found {
            Some((min_x, min_y, max_x, max_y))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_selection_and_bounding_box() {
        let mut mask = SelectionMask::new(100, 100);
        assert!(mask.is_empty());
        assert_eq!(mask.bounding_box(), None);

        mask.select_rect(10, 20, 30, 50);
        assert!(!mask.is_empty());
        assert_eq!(mask.get(15, 25), 255);
        assert_eq!(mask.get(5, 5), 0);
        assert_eq!(mask.bounding_box(), Some((10, 20, 29, 49)));
    }

    #[test]
    fn test_mask_boolean_operations() {
        let mut a = SelectionMask::new(50, 50);
        let mut b = SelectionMask::new(50, 50);

        a.select_rect(0, 0, 30, 30);
        b.select_rect(20, 20, 40, 40);

        let mut union_mask = a.clone();
        union_mask.union_with(&b);
        assert_eq!(union_mask.get(10, 10), 255);
        assert_eq!(union_mask.get(25, 25), 255);
        assert_eq!(union_mask.get(35, 35), 255);

        let mut sub_mask = a.clone();
        sub_mask.subtract(&b);
        assert_eq!(sub_mask.get(10, 10), 255);
        assert_eq!(sub_mask.get(25, 25), 0); // subtracted

        let mut int_mask = a.clone();
        int_mask.intersect_with(&b);
        assert_eq!(int_mask.get(10, 10), 0);
        assert_eq!(int_mask.get(25, 25), 255); // in intersection
    }

    #[test]
    fn test_ellipse_selection() {
        let mut mask = SelectionMask::new(100, 100);
        mask.select_ellipse(50.0, 50.0, 20.0, 10.0);
        assert_eq!(mask.get(50, 50), 255);
        assert_eq!(mask.get(50, 65), 0); // outside y radius
    }
}
