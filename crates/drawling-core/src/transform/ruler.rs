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
    Ellipse {
        center: (f32, f32),
        radius_x: f32,
        radius_y: f32,
        snap_distance: f32,
    },
    Perspective {
        vanishing_points: Vec<(f32, f32)>,
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

            Ruler::Ellipse { center, radius_x, radius_y, snap_distance } => {
                if *radius_x < 1e-4 || *radius_y < 1e-4 {
                    return pt;
                }
                let angle = (pt.1 - center.1).atan2(pt.0 - center.0);
                let target_x = center.0 + radius_x * angle.cos();
                let target_y = center.1 + radius_y * angle.sin();
                let dist = ((target_x - pt.0).powi(2) + (target_y - pt.1).powi(2)).sqrt();
                if dist <= *snap_distance {
                    (target_x, target_y)
                } else {
                    pt
                }
            }

            Ruler::Perspective { vanishing_points, snap_distance } => {
                let mut best_pt = pt;
                let mut min_dist = *snap_distance;
                let ray_step = std::f32::consts::PI / 12.0; // 15-degree guide rays
                for vp in vanishing_points {
                    let dx = pt.0 - vp.0;
                    let dy = pt.1 - vp.1;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist > 1e-4 {
                        let angle = dy.atan2(dx);
                        let snapped_angle = (angle / ray_step).round() * ray_step;
                        let target_x = vp.0 + dist * snapped_angle.cos();
                        let target_y = vp.1 + dist * snapped_angle.sin();
                        let d = ((target_x - pt.0).powi(2) + (target_y - pt.1).powi(2)).sqrt();
                        if d < min_dist {
                            min_dist = d;
                            best_pt = (target_x, target_y);
                        }
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

    #[test]
    fn test_ellipse_and_perspective_ruler_snap() {
        let ellipse = Ruler::Ellipse {
            center: (200.0, 200.0),
            radius_x: 100.0,
            radius_y: 50.0,
            snap_distance: 10.0,
        };
        let snapped_e = ellipse.snap((302.0, 200.0));
        assert!((snapped_e.0 - 300.0).abs() < 0.1);
        assert!((snapped_e.1 - 200.0).abs() < 0.1);

        let persp = Ruler::Perspective {
            vanishing_points: vec![(0.0, 100.0)],
            snap_distance: 15.0,
        };
        let snapped_p = persp.snap((100.0, 104.0));
        assert!((snapped_p.1 - 100.0).abs() < 1.0);
    }
}
