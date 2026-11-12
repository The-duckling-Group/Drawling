use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Ruler {
    Linear {
        p0: (f32, f32),
        p1: (f32, f32),
        snap_distance: f32,
    },
    Circular {
        center: (f32, f32),
        radius: f32,
        snap_distance: f32,
    },
    CurvedBezier {
        p0: (f32, f32),
        p1: (f32, f32), // Control point 1
        p2: (f32, f32), // Control point 2
        p3: (f32, f32), // End point
        snap_distance: f32,
    },
}

impl Ruler {
    /// Constrains/snaps an input coordinate to the ruler if within `snap_distance`
    pub fn snap(&self, pt: (f32, f32)) -> (f32, f32) {
        match self {
            Ruler::Linear { p0, p1, snap_distance } => {
                let dx = p1.0 - p0.0;
                let dy = p1.1 - p0.1;
                let len_sq = dx * dx + dy * dy;
                if len_sq < 1e-4 {
                    return pt;
                }
                let t = ((pt.0 - p0.0) * dx + (pt.1 - p0.1) * dy) / len_sq;
                let proj_x = p0.0 + t * dx;
                let proj_y = p0.1 + t * dy;
                let dist = ((proj_x - pt.0).powi(2) + (proj_y - pt.1).powi(2)).sqrt();
                if dist <= *snap_distance {
                    (proj_x, proj_y)
                } else {
                    pt
                }
            }

            Ruler::Circular { center, radius, snap_distance } => {
                let dx = pt.0 - center.0;
                let dy = pt.1 - center.1;
                let current_radius = (dx * dx + dy * dy).sqrt();
                if current_radius < 1e-4 {
                    return pt;
                }
                let dist = (current_radius - radius).abs();
                if dist <= *snap_distance {
                    let scale = radius / current_radius;
                    (center.0 + dx * scale, center.1 + dy * scale)
                } else {
                    pt
                }
            }

            Ruler::CurvedBezier { p0, p1, p2, p3, snap_distance } => {
                // Approximate closest point along cubic Bézier by sampling 32 steps
                let steps = 32;
                let mut best_pt = pt;
                let mut min_dist = *snap_distance;

                for i in 0..=steps {
                    let t = i as f32 / steps as f32;
                    let u = 1.0 - t;
                    let tt = t * t;
                    let uu = u * u;
                    let uuu = uu * u;
                    let ttt = tt * t;

                    let bx = uuu * p0.0 + 3.0 * uu * t * p1.0 + 3.0 * u * tt * p2.0 + ttt * p3.0;
                    let by = uuu * p0.1 + 3.0 * uu * t * p1.1 + 3.0 * u * tt * p2.1 + ttt * p3.1;

                    let dist = ((bx - pt.0).powi(2) + (by - pt.1).powi(2)).sqrt();
                    if dist < min_dist {
                        min_dist = dist;
                        best_pt = (bx, by);
                    }
                }

                best_pt
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_ruler_snap() {
        let ruler = Ruler::Linear {
            p0: (0.0, 0.0),
            p1: (100.0, 0.0),
            snap_distance: 10.0,
        };

        let pt = (50.0, 4.0); // within 10px from horizontal axis
        let snapped = ruler.snap(pt);
        assert_eq!(snapped, (50.0, 0.0));

        let far_pt = (50.0, 25.0); // outside snap distance
        assert_eq!(ruler.snap(far_pt), far_pt);
    }

    #[test]
    fn test_circular_ruler_snap() {
        let ruler = Ruler::Circular {
            center: (100.0, 100.0),
            radius: 50.0,
            snap_distance: 10.0,
        };

        let pt = (100.0, 154.0); // radius 54, target radius 50
        let snapped = ruler.snap(pt);
        assert_eq!(snapped, (100.0, 150.0));
    }
}
