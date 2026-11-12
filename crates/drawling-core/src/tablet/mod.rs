use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TabletTool {
    /// Standard pen drawing tip (e.g. XP-Pen stylus tip)
    Pen,
    /// Physical eraser on the back/top of the pen
    Eraser,
    /// Airbrush with finger wheel
    Airbrush,
    /// Mouse / puck
    Cursor,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct StylusButtons {
    /// Stylus tip touching tablet surface
    pub tip: bool,
    /// Primary barrel button (usually right-click or eyedropper)
    pub button1: bool,
    /// Secondary barrel button (usually eraser toggle or pan)
    pub button2: bool,
    /// Tertiary barrel button
    pub button3: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TabletInputEvent {
    /// Stylus hovering in range over the tablet surface
    ProximityIn {
        tool: TabletTool,
    },
    /// Stylus left tablet detection range
    ProximityOut {
        tool: TabletTool,
    },
    /// Stylus hover motion (not pressing down)
    Hover {
        x: f32,
        y: f32,
        tilt_x: f32,
        tilt_y: f32,
        tool: TabletTool,
    },
    /// Stylus touched surface
    Down {
        x: f32,
        y: f32,
        pressure: f32,
        tilt_x: f32,
        tilt_y: f32,
        tool: TabletTool,
        buttons: StylusButtons,
    },
    /// Stylus moving while pressed down
    Motion {
        x: f32,
        y: f32,
        pressure: f32,
        tilt_x: f32,
        tilt_y: f32,
        tool: TabletTool,
        buttons: StylusButtons,
        timestamp_ms: u64,
    },
    /// Stylus lifted from surface
    Up {
        tool: TabletTool,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PressureCurveMode {
    Linear,
    /// S-curve: sensitive for light sketching
    Soft,
    /// Exponential curve: requires firm pressure for inking
    Hard,
    /// Power curve with arbitrary gamma: output = (input)^gamma
    Gamma(f32),
    /// Cubic Bezier curve defined by control points (0,0) -> p1 -> p2 -> (1,1)
    Bezier {
        p1: (f32, f32),
        p2: (f32, f32),
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PressureCurve {
    pub mode: PressureCurveMode,
    /// Minimum pressure threshold below which pressure is 0.0 (deadzone)
    pub deadzone: f32,
    /// Raw pressure value that corresponds to 100% output
    pub max_threshold: f32,
}

impl Default for PressureCurve {
    fn default() -> Self {
        Self {
            mode: PressureCurveMode::Linear,
            deadzone: 0.02, // 2% deadzone to prevent accidental scratches
            max_threshold: 0.98,
        }
    }
}

impl PressureCurve {
    /// Maps raw pressure [0.0, 1.0] to calibrated pressure [0.0, 1.0]
    pub fn map(&self, raw: f32) -> f32 {
        if raw <= self.deadzone {
            return 0.0;
        }

        let normalized = ((raw - self.deadzone) / (self.max_threshold - self.deadzone)).clamp(0.0, 1.0);

        match self.mode {
            PressureCurveMode::Linear => normalized,
            PressureCurveMode::Soft => {
                // S-curve favoring light touches: sqrt-like response
                normalized.powf(0.65)
            }
            PressureCurveMode::Hard => {
                // Requires firmer hand: quadratic-like response
                normalized.powf(1.6)
            }
            PressureCurveMode::Gamma(g) => {
                let g = if g <= 0.0 { 1.0 } else { g };
                normalized.powf(g)
            }
            PressureCurveMode::Bezier { p1, p2 } => {
                // Approximate cubic bezier mapping: B(t) for t = normalized
                let t = normalized;
                let one_minus_t = 1.0 - t;
                // y(t) = 3*(1-t)^2 * t * y1 + 3*(1-t) * t^2 * y2 + t^3
                let y = 3.0 * one_minus_t * one_minus_t * t * p1.1
                    + 3.0 * one_minus_t * t * t * p2.1
                    + t * t * t;
                y.clamp(0.0, 1.0)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TabletCalibrationProfile {
    pub name: String,
    pub manufacturer: String,
    /// Hardware pressure levels (e.g. 8192 for XP-Pen, 4096 for older tablets)
    pub max_pressure_levels: u32,
    /// Does the pen support physical tilt (altitude/azimuth)
    pub supports_tilt: bool,
    /// Does the pen have an inverted physical eraser tip
    pub supports_eraser_tip: bool,
    /// Calibration curve
    pub pressure_curve: PressureCurve,
}

impl TabletCalibrationProfile {
    /// XP-Pen Artist & Deco standard profile (8192 levels, tilt support, P05/PA2/X3 pens)
    pub fn xp_pen_standard() -> Self {
        Self {
            name: "XP-Pen Artist/Deco Default".into(),
            manufacturer: "XP-Pen".into(),
            max_pressure_levels: 8192,
            supports_tilt: true,
            supports_eraser_tip: true,
            pressure_curve: PressureCurve {
                mode: PressureCurveMode::Linear,
                deadzone: 0.015,
                max_threshold: 0.985,
            },
        }
    }

    /// XP-Pen Soft: sensitive response for light-handed line sketchers
    pub fn xp_pen_soft() -> Self {
        Self {
            name: "XP-Pen High Sensitivity (Soft)".into(),
            manufacturer: "XP-Pen".into(),
            max_pressure_levels: 8192,
            supports_tilt: true,
            supports_eraser_tip: true,
            pressure_curve: PressureCurve {
                mode: PressureCurveMode::Soft,
                deadzone: 0.01,
                max_threshold: 0.95,
            },
        }
    }

    /// XP-Pen Hard: firm response for high-control comic inking
    pub fn xp_pen_hard() -> Self {
        Self {
            name: "XP-Pen Firm Inking (Hard)".into(),
            manufacturer: "XP-Pen".into(),
            max_pressure_levels: 8192,
            supports_tilt: true,
            supports_eraser_tip: true,
            pressure_curve: PressureCurve {
                mode: PressureCurveMode::Hard,
                deadzone: 0.03,
                max_threshold: 0.99,
            },
        }
    }

    /// Wacom Intuos & Cintiq Pro profile (Pro Pen 2, 8192 levels, tilt, eraser)
    pub fn wacom_pro_pen_2() -> Self {
        Self {
            name: "Wacom Pro Pen 2".into(),
            manufacturer: "Wacom".into(),
            max_pressure_levels: 8192,
            supports_tilt: true,
            supports_eraser_tip: true,
            pressure_curve: PressureCurve::default(),
        }
    }

    /// Huion Kamvas & Inspiroy profile (PW517 pen, 8192 levels)
    pub fn huion_standard() -> Self {
        Self {
            name: "Huion Kamvas (PW517)".into(),
            manufacturer: "Huion".into(),
            max_pressure_levels: 8192,
            supports_tilt: true,
            supports_eraser_tip: false,
            pressure_curve: PressureCurve::default(),
        }
    }
}

/// Helper function to convert tilt_x and tilt_y (in degrees [-90, +90] or radians)
/// to azimuth (angle in plane) and altitude (angle above tablet surface).
pub fn tilt_to_azimuth_altitude(tilt_x_deg: f32, tilt_y_deg: f32) -> (f32, f32) {
    let tx = tilt_x_deg.to_radians();
    let ty = tilt_y_deg.to_radians();

    let tan_tx = tx.tan();
    let tan_ty = ty.tan();

    // Altitude: angle with tablet surface (0 = flat on surface, PI/2 = upright perpendicular)
    let alt = (1.0 / (tan_tx * tan_tx + tan_ty * tan_ty + 1.0).sqrt()).acos();
    let altitude = std::f32::consts::FRAC_PI_2 - alt;

    // Azimuth: directional angle on the 2D plane
    let azimuth = tan_ty.atan2(tan_tx);

    (azimuth, altitude.clamp(0.0, std::f32::consts::FRAC_PI_2))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pressure_curve_mapping() {
        let curve = PressureCurve {
            mode: PressureCurveMode::Soft,
            deadzone: 0.05,
            max_threshold: 0.95,
        };

        // Below deadzone is zero
        assert_eq!(curve.map(0.02), 0.0);
        // Above max_threshold is 1.0
        assert_eq!(curve.map(0.99), 1.0);
        // Mid-range value should be elevated due to soft curve
        let mid = curve.map(0.5);
        assert!(mid > 0.5, "Soft curve should give higher output for mid pressure: {}", mid);
    }

    #[test]
    fn test_xp_pen_profile_creation() {
        let profile = TabletCalibrationProfile::xp_pen_standard();
        assert_eq!(profile.manufacturer, "XP-Pen");
        assert_eq!(profile.max_pressure_levels, 8192);
        assert!(profile.supports_tilt);
        assert!(profile.supports_eraser_tip);
    }

    #[test]
    fn test_tilt_conversion() {
        // Pen tilted 45 deg along X axis
        let (azimuth, altitude) = tilt_to_azimuth_altitude(45.0, 0.0);
        assert!(altitude < std::f32::consts::FRAC_PI_2);
        assert_eq!(azimuth.round(), 0.0);
    }
}
