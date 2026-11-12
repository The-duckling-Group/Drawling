use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StrokePoint {
    pub x: f32,
    pub y: f32,
    pub pressure: f32,
    pub tilt_x: f32,
    pub tilt_y: f32,
    pub timestamp_ms: u64,
}

impl StrokePoint {
    pub fn new(x: f32, y: f32, pressure: f32) -> Self {
        Self {
            x,
            y,
            pressure: pressure.clamp(0.0, 1.0),
            tilt_x: 0.0,
            tilt_y: 0.0,
            timestamp_ms: 0,
        }
    }

    pub fn with_details(
        x: f32,
        y: f32,
        pressure: f32,
        tilt_x: f32,
        tilt_y: f32,
        timestamp_ms: u64,
    ) -> Self {
        Self {
            x,
            y,
            pressure: pressure.clamp(0.0, 1.0),
            tilt_x: tilt_x.clamp(-1.0, 1.0),
            tilt_y: tilt_y.clamp(-1.0, 1.0),
            timestamp_ms,
        }
    }

    pub fn distance_to(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
            pressure: self.pressure + (other.pressure - self.pressure) * t,
            tilt_x: self.tilt_x + (other.tilt_x - self.tilt_x) * t,
            tilt_y: self.tilt_y + (other.tilt_y - self.tilt_y) * t,
            timestamp_ms: (self.timestamp_ms as f32
                + (other.timestamp_ms as f32 - self.timestamp_ms as f32) * t)
                as u64,
        }
    }
}
