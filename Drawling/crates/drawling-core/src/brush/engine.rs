use super::super::color::Color;
use super::super::stroke::point::StrokePoint;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrushParameters {
    pub name: String,
    pub size: f32,
    pub min_size_ratio: f32,
    pub opacity: f32,
    pub min_opacity_ratio: f32,
    pub hardness: f32,
    pub spacing: f32, // fraction of brush radius (e.g. 0.15 = 15%)
    pub scatter: f32, // random offset dispersion
    pub flow: f32,
    pub angle_deg: f32,
    pub size_by_pressure: bool,
    pub opacity_by_pressure: bool,
    pub color: Color,
}

impl Default for BrushParameters {
    fn default() -> Self {
        Self {
            name: "Round Brush".into(),
            size: 20.0,
            min_size_ratio: 0.1,
            opacity: 1.0,
            min_opacity_ratio: 0.0,
            hardness: 0.8,
            spacing: 0.1,
            scatter: 0.0,
            flow: 1.0,
            angle_deg: 0.0,
            size_by_pressure: true,
            opacity_by_pressure: true,
            color: Color::BLACK,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BrushDab {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub opacity: f32,
    pub hardness: f32,
    pub color: Color,
}

#[derive(Debug, Clone)]
pub struct BrushEngine {
    params: BrushParameters,
    last_point: Option<StrokePoint>,
    distance_accumulator: f32,
    rng_seed: u64,
}

impl BrushEngine {
    pub fn new(params: BrushParameters) -> Self {
        Self {
            params,
            last_point: None,
            distance_accumulator: 0.0,
            rng_seed: 123456789,
        }
    }

    pub fn params(&self) -> &BrushParameters {
        &self.params
    }

    pub fn params_mut(&mut self) -> &mut BrushParameters {
        &mut self.params
    }

    pub fn reset_stroke(&mut self) {
        self.last_point = None;
        self.distance_accumulator = 0.0;
    }

    // Simple pseudo-random float generator in [-1.0, 1.0]
    fn next_rand(&mut self) -> f32 {
        self.rng_seed = self.rng_seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let val = (self.rng_seed >> 33) as f32 / (1u32 << 31) as f32;
        val - 1.0
    }

    /// Process a new stroke point and generate all stamp dabs between last and current point
    pub fn push_point(&mut self, current: StrokePoint) -> Vec<BrushDab> {
        let mut dabs = Vec::new();

        let prev = match self.last_point {
            Some(p) => p,
            None => {
                // First point in stroke
                self.last_point = Some(current);
                dabs.push(self.create_dab_at(current));
                return dabs;
            }
        };

        let total_dist = prev.distance_to(&current);
        if total_dist <= 0.0001 {
            return dabs;
        }

        // Calculate dynamic spacing based on brush size and pressure
        let avg_pressure = (prev.pressure + current.pressure) * 0.5;
        let radius = self.calculate_radius(avg_pressure);
        let step_dist = (radius * 2.0 * self.params.spacing.max(0.02)).max(1.0);

        let mut covered = step_dist - self.distance_accumulator;

        while covered <= total_dist {
            let t = covered / total_dist;
            let interpolated = prev.lerp(&current, t);
            dabs.push(self.create_dab_at(interpolated));
            covered += step_dist;
        }

        self.distance_accumulator = total_dist - (covered - step_dist);
        self.last_point = Some(current);

        dabs
    }

    fn calculate_radius(&self, pressure: f32) -> f32 {
        let scale = if self.params.size_by_pressure {
            self.params.min_size_ratio + (1.0 - self.params.min_size_ratio) * pressure
        } else {
            1.0
        };
        (self.params.size * 0.5 * scale).max(0.5)
    }

    fn calculate_opacity(&self, pressure: f32) -> f32 {
        let factor = if self.params.opacity_by_pressure {
            self.params.min_opacity_ratio + (1.0 - self.params.min_opacity_ratio) * pressure
        } else {
            1.0
        };
        (self.params.opacity * factor * self.params.flow).clamp(0.0, 1.0)
    }

    fn create_dab_at(&mut self, pt: StrokePoint) -> BrushDab {
        let radius = self.calculate_radius(pt.pressure);
        let opacity = self.calculate_opacity(pt.pressure);

        let (mut x, mut y) = (pt.x, pt.y);

        if self.params.scatter > 0.0 {
            let scatter_mag = self.params.scatter * radius * 2.0;
            x += self.next_rand() * scatter_mag;
            y += self.next_rand() * scatter_mag;
        }

        BrushDab {
            x,
            y,
            radius,
            opacity,
            hardness: self.params.hardness,
            color: self.params.color,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brush_engine_spacing_and_pressure() {
        let mut params = BrushParameters::default();
        params.size = 20.0;
        params.spacing = 0.5; // step distance = radius * 2 * 0.5 = 10px
        params.size_by_pressure = true;

        let mut engine = BrushEngine::new(params);
        let p0 = StrokePoint::new(0.0, 0.0, 1.0);
        let first_dabs = engine.push_point(p0);
        assert_eq!(first_dabs.len(), 1);
        assert_eq!(first_dabs[0].radius, 10.0); // radius = 20 * 0.5

        // Move 50px along X axis
        let p1 = StrokePoint::new(50.0, 0.0, 1.0);
        let line_dabs = engine.push_point(p1);
        assert!(line_dabs.len() >= 4);

        // Verify dabs progress along X
        for dab in line_dabs {
            assert!(dab.x > 0.0 && dab.x <= 50.0);
            assert_eq!(dab.y, 0.0);
        }
    }
}
