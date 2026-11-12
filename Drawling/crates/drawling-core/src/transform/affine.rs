use crate::layer::Layer;
use serde::{Deserialize, Serialize};

/// 2D Affine transform matrix:
/// [ a  c  tx ]
/// [ b  d  ty ]
/// [ 0  0  1  ]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform2D {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub tx: f32,
    pub ty: f32,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::identity()
    }
}

impl Transform2D {
    pub fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            tx: 0.0,
            ty: 0.0,
        }
    }

    pub fn translation(tx: f32, ty: f32) -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            tx,
            ty,
        }
    }

    pub fn scale(sx: f32, sy: f32) -> Self {
        Self {
            a: sx,
            b: 0.0,
            c: 0.0,
            d: sy,
            tx: 0.0,
            ty: 0.0,
        }
    }

    pub fn rotation(radians: f32) -> Self {
        let cos = radians.cos();
        let sin = radians.sin();
        Self {
            a: cos,
            b: sin,
            c: -sin,
            d: cos,
            tx: 0.0,
            ty: 0.0,
        }
    }

    pub fn multiply(&self, other: &Self) -> Self {
        Self {
            a: self.a * other.a + self.c * other.b,
            b: self.b * other.a + self.d * other.b,
            c: self.a * other.c + self.c * other.d,
            d: self.b * other.c + self.d * other.d,
            tx: self.a * other.tx + self.c * other.ty + self.tx,
            ty: self.b * other.tx + self.d * other.ty + self.ty,
        }
    }

    pub fn transform_point(&self, x: f32, y: f32) -> (f32, f32) {
        (
            self.a * x + self.c * y + self.tx,
            self.b * x + self.d * y + self.ty,
        )
    }

    pub fn invert(&self) -> Option<Self> {
        let det = self.a * self.d - self.b * self.c;
        if det.abs() < 1e-8 {
            return None;
        }
        let inv_det = 1.0 / det;
        Some(Self {
            a: self.d * inv_det,
            b: -self.b * inv_det,
            c: -self.c * inv_det,
            d: self.a * inv_det,
            tx: (self.c * self.ty - self.d * self.tx) * inv_det,
            ty: (self.b * self.tx - self.a * self.ty) * inv_det,
        })
    }

    /// Resample layer pixels using this affine transform with bilinear interpolation
    pub fn apply_to_layer(&self, layer: &mut Layer) {
        if layer.locked {
            return;
        }

        let inv = match self.invert() {
            Some(inv) => inv,
            None => return,
        };

        let width = layer.width;
        let height = layer.height;
        let original_pixels = layer.pixels.clone();
        layer.clear();

        for y in 0..height {
            for x in 0..width {
                // Map target pixel back to source coordinates
                let (sx, sy) = inv.transform_point(x as f32 + 0.5, y as f32 + 0.5);

                if sx >= 0.0 && sx < width as f32 && sy >= 0.0 && sy < height as f32 {
                    let sampled = bilinear_sample(&original_pixels, width, height, sx - 0.5, sy - 0.5);
                    layer.set_pixel(x, y, sampled);
                }
            }
        }
    }
}

fn bilinear_sample(pixels: &[u8], width: u32, height: u32, x: f32, y: f32) -> [u8; 4] {
    let x0 = (x.floor() as i32).clamp(0, width as i32 - 1) as u32;
    let x1 = (x0 + 1).min(width - 1);
    let y0 = (y.floor() as i32).clamp(0, height as i32 - 1) as u32;
    let y1 = (y0 + 1).min(height - 1);

    let fx = (x - x.floor()).clamp(0.0, 1.0);
    let fy = (y - y.floor()).clamp(0.0, 1.0);

    let get_px = |px: u32, py: u32| -> [f32; 4] {
        let idx = ((py * width + px) * 4) as usize;
        [
            pixels[idx] as f32,
            pixels[idx + 1] as f32,
            pixels[idx + 2] as f32,
            pixels[idx + 3] as f32,
        ]
    };

    let p00 = get_px(x0, y0);
    let p10 = get_px(x1, y0);
    let p01 = get_px(x0, y1);
    let p11 = get_px(x1, y1);

    let mut out = [0u8; 4];
    for c in 0..4 {
        let top = p00[c] * (1.0 - fx) + p10[c] * fx;
        let bot = p01[c] * (1.0 - fx) + p11[c] * fx;
        let val = top * (1.0 - fy) + bot * fy;
        out[c] = val.round().clamp(0.0, 255.0) as u8;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_matrix_operations() {
        let t1 = Transform2D::translation(50.0, 100.0);
        let pt = (10.0, 20.0);
        assert_eq!(t1.transform_point(pt.0, pt.1), (60.0, 120.0));

        let s1 = Transform2D::scale(2.0, 3.0);
        let combined = s1.multiply(&t1);
        // Translation applied first: (10+50)*2 = 120, (20+100)*3 = 360
        assert_eq!(combined.transform_point(pt.0, pt.1), (120.0, 360.0));

        let inv = combined.invert().unwrap();
        let back = inv.transform_point(120.0, 360.0);
        assert!((back.0 - pt.0).abs() < 1e-4);
        assert!((back.1 - pt.1).abs() < 1e-4);
    }

    #[test]
    fn test_rotation() {
        let rot = Transform2D::rotation(std::f32::consts::FRAC_PI_2);
        let (rx, ry) = rot.transform_point(1.0, 0.0);
        assert!((rx - 0.0).abs() < 1e-4);
        assert!((ry - 1.0).abs() < 1e-4);
    }
}
