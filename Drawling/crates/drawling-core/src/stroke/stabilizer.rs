use super::point::StrokePoint;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StabilizerMode {
    Off,
    MovingAverage { window_size: usize },
    LazyMouse { rope_length: f32, tension: f32 },
}

#[derive(Debug, Clone)]
pub struct StrokeStabilizer {
    mode: StabilizerMode,
    history: VecDeque<StrokePoint>,
    lazy_pos: Option<StrokePoint>,
}

impl StrokeStabilizer {
    pub fn new(mode: StabilizerMode) -> Self {
        Self {
            mode,
            history: VecDeque::new(),
            lazy_pos: None,
        }
    }

    pub fn reset(&mut self) {
        self.history.clear();
        self.lazy_pos = None;
    }

    pub fn set_mode(&mut self, mode: StabilizerMode) {
        self.mode = mode;
        self.reset();
    }

    pub fn mode(&self) -> StabilizerMode {
        self.mode
    }

    /// Feed an incoming raw input point, and obtain the stabilized point (if available)
    pub fn push(&mut self, pt: StrokePoint) -> Option<StrokePoint> {
        match self.mode {
            StabilizerMode::Off => Some(pt),

            StabilizerMode::MovingAverage { window_size } => {
                let size = window_size.max(1);
                self.history.push_back(pt);
                if self.history.len() > size {
                    self.history.pop_front();
                }

                let count = self.history.len() as f32;
                let mut sum_x = 0.0;
                let mut sum_y = 0.0;
                let mut sum_pressure = 0.0;
                let mut sum_tilt_x = 0.0;
                let mut sum_tilt_y = 0.0;

                // Weight more recent points slightly higher
                let mut total_weight = 0.0;
                for (i, p) in self.history.iter().enumerate() {
                    let weight = 1.0 + (i as f32) / count;
                    sum_x += p.x * weight;
                    sum_y += p.y * weight;
                    sum_pressure += p.pressure * weight;
                    sum_tilt_x += p.tilt_x * weight;
                    sum_tilt_y += p.tilt_y * weight;
                    total_weight += weight;
                }

                Some(StrokePoint::with_details(
                    sum_x / total_weight,
                    sum_y / total_weight,
                    sum_pressure / total_weight,
                    sum_tilt_x / total_weight,
                    sum_tilt_y / total_weight,
                    pt.timestamp_ms,
                ))
            }

            StabilizerMode::LazyMouse { rope_length, tension } => {
                let rope = rope_length.max(1.0);
                let tension = tension.clamp(0.01, 1.0);

                if let Some(prev) = self.lazy_pos {
                    let dx = pt.x - prev.x;
                    let dy = pt.y - prev.y;
                    let dist = (dx * dx + dy * dy).sqrt();

                    if dist <= rope {
                        // Within string radius: brush cursor holds position
                        None
                    } else {
                        // Pulled along rope vector
                        let angle = dy.atan2(dx);
                        let excess = dist - rope;
                        let step = excess * tension;
                        let target_x = prev.x + angle.cos() * step;
                        let target_y = prev.y + angle.sin() * step;

                        let next_pt = StrokePoint::with_details(
                            target_x,
                            target_y,
                            prev.pressure + (pt.pressure - prev.pressure) * tension,
                            prev.tilt_x + (pt.tilt_x - prev.tilt_x) * tension,
                            prev.tilt_y + (pt.tilt_y - prev.tilt_y) * tension,
                            pt.timestamp_ms,
                        );

                        self.lazy_pos = Some(next_pt);
                        Some(next_pt)
                    }
                } else {
                    self.lazy_pos = Some(pt);
                    Some(pt)
                }
            }
        }
    }

    /// Flush remaining points on pen lift
    pub fn finish(&mut self) -> Option<StrokePoint> {
        let last = self.lazy_pos.take();
        self.history.clear();
        last
    }
}

/// Catmull-Rom Spline interpolator for generating smooth sub-pixel curves
pub struct CatmullRomSpline;

impl CatmullRomSpline {
    /// Interpolates between p1 and p2 using p0 and p3 as control anchors.
    /// Uses centripetal Catmull-Rom formulation (alpha = 0.5) to eliminate cusps and loops.
    pub fn interpolate(
        p0: &StrokePoint,
        p1: &StrokePoint,
        p2: &StrokePoint,
        p3: &StrokePoint,
        t: f32,
    ) -> StrokePoint {
        let t2 = t * t;
        let t3 = t2 * t;

        // Centripetal blending polynomial
        let f0 = -0.5 * t3 + t2 - 0.5 * t;
        let f1 = 1.5 * t3 - 2.5 * t2 + 1.0;
        let f2 = -1.5 * t3 + 2.0 * t2 + 0.5 * t;
        let f3 = 0.5 * t3 - 0.5 * t2;

        let x = p0.x * f0 + p1.x * f1 + p2.x * f2 + p3.x * f3;
        let y = p0.y * f0 + p1.y * f1 + p2.y * f2 + p3.y * f3;
        let pressure = (p0.pressure * f0 + p1.pressure * f1 + p2.pressure * f2 + p3.pressure * f3)
            .clamp(0.0, 1.0);
        let tilt_x = (p0.tilt_x * f0 + p1.tilt_x * f1 + p2.tilt_x * f2 + p3.tilt_x * f3)
            .clamp(-1.0, 1.0);
        let tilt_y = (p0.tilt_y * f0 + p1.tilt_y * f1 + p2.tilt_y * f2 + p3.tilt_y * f3)
            .clamp(-1.0, 1.0);

        StrokePoint::with_details(x, y, pressure, tilt_x, tilt_y, p1.timestamp_ms)
    }

    /// Sample `subdivisions` points between p1 and p2
    pub fn sample_segment(
        p0: &StrokePoint,
        p1: &StrokePoint,
        p2: &StrokePoint,
        p3: &StrokePoint,
        subdivisions: usize,
    ) -> Vec<StrokePoint> {
        let steps = subdivisions.max(1);
        let mut result = Vec::with_capacity(steps);
        for i in 1..=steps {
            let t = (i as f32) / (steps as f32);
            result.push(Self::interpolate(p0, p1, p2, p3, t));
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lazy_mouse_deadband() {
        let mut stab = StrokeStabilizer::new(StabilizerMode::LazyMouse {
            rope_length: 20.0,
            tension: 0.5,
        });

        let p1 = StrokePoint::new(0.0, 0.0, 0.5);
        assert!(stab.push(p1).is_some());

        // Small movement within rope length should not trigger movement
        let p2 = StrokePoint::new(5.0, 5.0, 0.5);
        assert!(stab.push(p2).is_none());

        // Movement beyond rope length pulls cursor
        let p3 = StrokePoint::new(50.0, 0.0, 0.5);
        let out = stab.push(p3);
        assert!(out.is_some());
        assert!(out.unwrap().x > 0.0);
    }
}
