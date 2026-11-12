use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BlendMode {
    #[default]
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    Add,
    Subtract,
    Difference,
    SoftLight,
    HardLight,
}

impl BlendMode {
    pub const ALL: [BlendMode; 13] = [
        BlendMode::Normal,
        BlendMode::Multiply,
        BlendMode::Screen,
        BlendMode::Overlay,
        BlendMode::Darken,
        BlendMode::Lighten,
        BlendMode::ColorDodge,
        BlendMode::ColorBurn,
        BlendMode::Add,
        BlendMode::Subtract,
        BlendMode::Difference,
        BlendMode::SoftLight,
        BlendMode::HardLight,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            BlendMode::Normal => "Normal",
            BlendMode::Multiply => "Multiply",
            BlendMode::Screen => "Screen",
            BlendMode::Overlay => "Overlay",
            BlendMode::Darken => "Darken",
            BlendMode::Lighten => "Lighten",
            BlendMode::ColorDodge => "Color Dodge",
            BlendMode::ColorBurn => "Color Burn",
            BlendMode::Add => "Add (Linear Dodge)",
            BlendMode::Subtract => "Subtract",
            BlendMode::Difference => "Difference",
            BlendMode::SoftLight => "Soft Light",
            BlendMode::HardLight => "Hard Light",
        }
    }

    /// Pure mathematical blend formula for a single color channel in [0.0, 1.0].
    /// cb: background/destination channel, cs: source channel
    #[inline(always)]
    pub fn blend_channel(&self, cb: f32, cs: f32) -> f32 {
        match self {
            BlendMode::Normal => cs,
            BlendMode::Multiply => cb * cs,
            BlendMode::Screen => cb + cs - (cb * cs),
            BlendMode::Overlay => {
                if cb <= 0.5 {
                    2.0 * cb * cs
                } else {
                    1.0 - 2.0 * (1.0 - cb) * (1.0 - cs)
                }
            }
            BlendMode::Darken => cb.min(cs),
            BlendMode::Lighten => cb.max(cs),
            BlendMode::ColorDodge => {
                if cb <= 0.0 {
                    0.0
                } else if cs >= 1.0 {
                    1.0
                } else {
                    (cb / (1.0 - cs)).min(1.0)
                }
            }
            BlendMode::ColorBurn => {
                if cb >= 1.0 {
                    1.0
                } else if cs <= 0.0 {
                    0.0
                } else {
                    1.0 - ((1.0 - cb) / cs).min(1.0)
                }
            }
            BlendMode::Add => (cb + cs).min(1.0),
            BlendMode::Subtract => (cb - cs).max(0.0),
            BlendMode::Difference => (cb - cs).abs(),
            BlendMode::SoftLight => {
                if cs <= 0.5 {
                    cb - (1.0 - 2.0 * cs) * cb * (1.0 - cb)
                } else {
                    let d = if cb <= 0.25 {
                        ((16.0 * cb - 12.0) * cb + 4.0) * cb
                    } else {
                        cb.sqrt()
                    };
                    cb + (2.0 * cs - 1.0) * (d - cb)
                }
            }
            BlendMode::HardLight => {
                if cs <= 0.5 {
                    2.0 * cb * cs
                } else {
                    1.0 - 2.0 * (1.0 - cb) * (1.0 - cs)
                }
            }
        }
    }

    /// Full Porter-Duff compositing with blend mode for RGBA8 pixels.
    /// `dst`: [r, g, b, a] in [0, 255] (destination pixel)
    /// `src`: [r, g, b, a] in [0, 255] (source pixel)
    /// `opacity`: layer opacity factor [0.0, 1.0]
    /// `alpha_locked`: if true, destination alpha remains unchanged and blends only into existing color
    #[inline]
    pub fn composite_pixel(
        &self,
        dst: [u8; 4],
        src: [u8; 4],
        opacity: f32,
        alpha_locked: bool,
    ) -> [u8; 4] {
        let sa_raw = (src[3] as f32 / 255.0) * opacity.clamp(0.0, 1.0);
        if sa_raw <= 0.0 {
            return dst;
        }

        let da = dst[3] as f32 / 255.0;
        if da <= 0.0 && !alpha_locked {
            // Background is completely transparent: output is directly source scaled by opacity
            return [src[0], src[1], src[2], (sa_raw * 255.0).round() as u8];
        }

        if alpha_locked && da <= 0.0 {
            // Cannot paint on transparent area when alpha is locked
            return dst;
        }

        let dr = dst[0] as f32 / 255.0;
        let dg = dst[1] as f32 / 255.0;
        let db = dst[2] as f32 / 255.0;

        let sr = src[0] as f32 / 255.0;
        let sg = src[1] as f32 / 255.0;
        let sb = src[2] as f32 / 255.0;

        let out_a = if alpha_locked {
            da
        } else {
            sa_raw + da * (1.0 - sa_raw)
        };

        if out_a <= 0.0 {
            return [0, 0, 0, 0];
        }

        // Blend per channel
        let br = self.blend_channel(dr, sr);
        let bg = self.blend_channel(dg, sg);
        let bb = self.blend_channel(db, sb);

        // General Porter-Duff Over composite with blend function:
        // C_out = (sa * (1 - da) * Cs + sa * da * B(Cd, Cs) + (1 - sa) * da * Cd) / out_a
        let blend_comp = |cd: f32, cs: f32, b_val: f32| -> u8 {
            let res = (sa_raw * (1.0 - da) * cs
                + sa_raw * da * b_val
                + (1.0 - sa_raw) * da * cd)
                / out_a;
            (res.clamp(0.0, 1.0) * 255.0).round() as u8
        };

        let out_r = blend_comp(dr, sr, br);
        let out_g = blend_comp(dg, sg, bg);
        let out_b = blend_comp(db, sb, bb);
        let out_a_u8 = (out_a.clamp(0.0, 1.0) * 255.0).round() as u8;

        [out_r, out_g, out_b, out_a_u8]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_composite() {
        let dst = [255, 0, 0, 255]; // Red opaque
        let src = [0, 0, 255, 255]; // Blue opaque
        let res = BlendMode::Normal.composite_pixel(dst, src, 1.0, false);
        assert_eq!(res, [0, 0, 255, 255]);
    }

    #[test]
    fn test_multiply() {
        let white = 1.0f32;
        let half = 0.5f32;
        assert_eq!(BlendMode::Multiply.blend_channel(white, half), 0.5);
        assert_eq!(BlendMode::Multiply.blend_channel(0.5, 0.5), 0.25);
    }

    #[test]
    fn test_alpha_lock() {
        let dst = [255, 0, 0, 0]; // completely transparent
        let src = [0, 255, 0, 255]; // green
        let res = BlendMode::Normal.composite_pixel(dst, src, 1.0, true);
        assert_eq!(res, dst); // Nothing painted on alpha lock
    }
}
