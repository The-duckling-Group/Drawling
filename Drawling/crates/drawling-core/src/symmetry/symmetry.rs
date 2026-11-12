use crate::stroke::point::StrokePoint;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SymmetryMode {
    None,
    HorizontalMirror { axis_y: f32 },
    VerticalMirror { axis_x: f32 },
    DualAxis { center_x: f32, center_y: f32 },
    Radial {
        center_x: f32,
        center_y: f32,
        segments: u32,
        mirror: bool,
    },
}

impl Default for SymmetryMode {
    fn default() -> Self {
        SymmetryMode::None
    }
}

pub struct SymmetryGuide;

impl SymmetryGuide {
    /// Generates all mirrored/rotated symmetrical points corresponding to the input stroke point
    pub fn generate_points(mode: &SymmetryMode, pt: StrokePoint) -> Vec<StrokePoint> {
        match mode {
            SymmetryMode::None => vec![pt],

            SymmetryMode::HorizontalMirror { axis_y } => {
                let mirrored_y = 2.0 * axis_y - pt.y;
                let mut mirrored = pt;
                mirrored.y = mirrored_y;
                mirrored.tilt_y = -pt.tilt_y;
                vec![pt, mirrored]
            }

            SymmetryMode::VerticalMirror { axis_x } => {
                let mirrored_x = 2.0 * axis_x - pt.x;
                let mut mirrored = pt;
                mirrored.x = mirrored_x;
                mirrored.tilt_x = -pt.tilt_x;
                vec![pt, mirrored]
            }

            SymmetryMode::DualAxis { center_x, center_y } => {
                let mx = 2.0 * center_x - pt.x;
                let my = 2.0 * center_y - pt.y;

                let mut p_h = pt;
                p_h.y = my;
                p_h.tilt_y = -pt.tilt_y;

                let mut p_v = pt;
                p_v.x = mx;
                p_v.tilt_x = -pt.tilt_x;

                let mut p_d = pt;
                p_d.x = mx;
                p_d.y = my;
                p_d.tilt_x = -pt.tilt_x;
                p_d.tilt_y = -pt.tilt_y;

                vec![pt, p_v, p_h, p_d]
            }

            SymmetryMode::Radial {
                center_x,
                center_y,
                segments,
                mirror,
            } => {
                let n = (*segments).clamp(2, 36);
                let angle_step = 2.0 * std::f32::consts::PI / (n as f32);

                let dx = pt.x - center_x;
                let dy = pt.y - center_y;
                let base_radius = (dx * dx + dy * dy).sqrt();
                let base_angle = dy.atan2(dx);

                let mut points = Vec::with_capacity((n * if *mirror { 2 } else { 1 }) as usize);

                for i in 0..n {
                    let theta = base_angle + (i as f32) * angle_step;
                    let mut p = pt;
                    p.x = center_x + base_radius * theta.cos();
                    p.y = center_y + base_radius * theta.sin();
                    points.push(p);

                    if *mirror {
                        let m_theta = -base_angle + (i as f32) * angle_step;
                        let mut pm = pt;
                        pm.x = center_x + base_radius * m_theta.cos();
                        pm.y = center_y + base_radius * m_theta.sin();
                        points.push(pm);
                    }
                }

                points
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_horizontal_and_vertical_symmetry() {
        let pt = StrokePoint::new(10.0, 20.0, 0.5);

        let h_mode = SymmetryMode::HorizontalMirror { axis_y: 50.0 };
        let h_points = SymmetryGuide::generate_points(&h_mode, pt);
        assert_eq!(h_points.len(), 2);
        assert_eq!(h_points[0], pt);
        assert_eq!(h_points[1].y, 80.0); // 2*50 - 20 = 80

        let v_mode = SymmetryMode::VerticalMirror { axis_x: 50.0 };
        let v_points = SymmetryGuide::generate_points(&v_mode, pt);
        assert_eq!(v_points.len(), 2);
        assert_eq!(v_points[1].x, 90.0); // 2*50 - 10 = 90
    }

    #[test]
    fn test_radial_symmetry_count() {
        let pt = StrokePoint::new(150.0, 100.0, 0.8);
        let rad_mode = SymmetryMode::Radial {
            center_x: 100.0,
            center_y: 100.0,
            segments: 6,
            mirror: false,
        };
        let rad_points = SymmetryGuide::generate_points(&rad_mode, pt);
        assert_eq!(rad_points.len(), 6);

        // With mirror
        let rad_mirror = SymmetryMode::Radial {
            center_x: 100.0,
            center_y: 100.0,
            segments: 6,
            mirror: true,
        };
        let rad_mirror_points = SymmetryGuide::generate_points(&rad_mirror, pt);
        assert_eq!(rad_mirror_points.len(), 12);
    }
}
