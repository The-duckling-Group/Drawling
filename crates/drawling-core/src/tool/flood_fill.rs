use crate::color::Color;
use crate::layer::Layer;
use std::collections::VecDeque;

pub struct FloodFill;

impl FloodFill {
    /// Performs flood fill (Bucket Fill) on the specified layer.
    /// - `tolerance`: color difference threshold in [0.0, 1.0]
    /// - `expand_pixels`: dilates the filled area by N pixels to close white halos under lineart
    /// - `contiguous`: if true, fills connected area; if false, replaces color globally
    pub fn fill(
        layer: &mut Layer,
        start_x: u32,
        start_y: u32,
        fill_color: Color,
        tolerance: f32,
        expand_pixels: i32,
        contiguous: bool,
    ) {
        if layer.locked || start_x >= layer.width || start_y >= layer.height {
            return;
        }

        let target_color = match layer.get_pixel(start_x, start_y) {
            Some(p) => p,
            None => return,
        };
        let [tr, tg, tb, ta] = target_color;
        let [fr, fg, fb, fa] = fill_color.to_rgba8();

        if tr == fr && tg == fg && tb == fb && ta == fa {
            return; // Target already matches fill color
        }

        let width = layer.width as usize;
        let height = layer.height as usize;
        let tol_sq = (tolerance * 255.0) * (tolerance * 255.0) * 4.0;

        let matches_target = |r: u8, g: u8, b: u8, a: u8| -> bool {
            let dr = (r as f32) - (tr as f32);
            let dg = (g as f32) - (tg as f32);
            let db = (b as f32) - (tb as f32);
            let da = (a as f32) - (ta as f32);
            (dr * dr + dg * dg + db * db + da * da) <= tol_sq
        };

        let mut mask = vec![false; width * height];

        if !contiguous {
            // Global color replacement
            for y in 0..height {
                for x in 0..width {
                    let idx = (y * width + x) * 4;
                    if matches_target(
                        layer.pixels[idx],
                        layer.pixels[idx + 1],
                        layer.pixels[idx + 2],
                        layer.pixels[idx + 3],
                    ) {
                        mask[y * width + x] = true;
                    }
                }
            }
        } else {
            // Contiguous 4-way BFS flood fill
            let mut queue = VecDeque::new();
            queue.push_back((start_x as usize, start_y as usize));
            mask[start_y as usize * width + start_x as usize] = true;

            while let Some((cx, cy)) = queue.pop_front() {
                let neighbors = [
                    (cx.wrapping_sub(1), cy),
                    (cx + 1, cy),
                    (cx, cy.wrapping_sub(1)),
                    (cx, cy + 1),
                ];

                for (nx, ny) in neighbors {
                    if nx < width && ny < height {
                        let n_idx = ny * width + nx;
                        if !mask[n_idx] {
                            let p_idx = n_idx * 4;
                            if matches_target(
                                layer.pixels[p_idx],
                                layer.pixels[p_idx + 1],
                                layer.pixels[p_idx + 2],
                                layer.pixels[p_idx + 3],
                            ) {
                                mask[n_idx] = true;
                                queue.push_back((nx, ny));
                            }
                        }
                    }
                }
            }
        }

        // Apply expand_pixels dilation if requested
        let final_mask = if expand_pixels > 0 {
            let mut expanded = mask.clone();
            let rad = expand_pixels as usize;

            for y in 0..height {
                for x in 0..width {
                    if mask[y * width + x] {
                        let min_y = y.saturating_sub(rad);
                        let max_y = (y + rad).min(height - 1);
                        let min_x = x.saturating_sub(rad);
                        let max_x = (x + rad).min(width - 1);

                        for ey in min_y..=max_y {
                            for ex in min_x..=max_x {
                                expanded[ey * width + ex] = true;
                            }
                        }
                    }
                }
            }
            expanded
        } else {
            mask
        };

        // Write fill color to marked pixels
        for y in 0..height {
            for x in 0..width {
                if final_mask[y * width + x] {
                    let idx = (y * width + x) * 4;
                    layer.pixels[idx] = fr;
                    layer.pixels[idx + 1] = fg;
                    layer.pixels[idx + 2] = fb;
                    layer.pixels[idx + 3] = fa;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flood_fill_bounded_area() {
        let mut layer = Layer::new("Fill Layer", 10, 10);
        layer.fill_color(Color::WHITE);

        // Draw a black box barrier from (2, 2) to (7, 7)
        for x in 2..=7 {
            layer.set_color(x, 2, Color::BLACK);
            layer.set_color(x, 7, Color::BLACK);
        }
        for y in 2..=7 {
            layer.set_color(2, y, Color::BLACK);
            layer.set_color(7, y, Color::BLACK);
        }

        // Fill inside the box with RED
        FloodFill::fill(&mut layer, 4, 4, Color::RED, 0.1, 0, true);

        // Inside should be RED
        assert_eq!(layer.get_color(4, 4), Some(Color::RED));
        // Border should still be BLACK
        assert_eq!(layer.get_color(2, 2), Some(Color::BLACK));
        // Outside should still be WHITE
        assert_eq!(layer.get_color(0, 0), Some(Color::WHITE));
    }

    #[test]
    fn test_flood_fill_with_expansion() {
        let mut layer = Layer::new("Expand Fill", 10, 10);
        layer.fill_color(Color::WHITE);
        layer.set_color(5, 5, Color::BLACK);

        // Fill at (5, 5) with expand_pixels = 1
        FloodFill::fill(&mut layer, 5, 5, Color::RED, 0.1, 1, true);

        // Surrounding pixel (5, 6) should now also be RED due to expansion
        assert_eq!(layer.get_color(5, 6), Some(Color::RED));
    }
}
