use crate::layer::Layer;
use rayon::prelude::*;

pub struct GaussianBlur;

impl GaussianBlur {
    /// Applies a multithreaded Gaussian Blur to a layer using separable horizontal and vertical passes
    pub fn apply(layer: &mut Layer, sigma: f32) {
        if sigma <= 0.05 || layer.locked {
            return;
        }

        let radius = (sigma * 3.0).ceil() as i32;
        let kernel_size = (radius * 2 + 1) as usize;

        // Compute 1D Gaussian Kernel
        let mut kernel = Vec::with_capacity(kernel_size);
        let two_sigma_sq = 2.0 * sigma * sigma;
        let mut kernel_sum = 0.0f32;

        for i in -radius..=radius {
            let val = (-((i * i) as f32) / two_sigma_sq).exp();
            kernel.push(val);
            kernel_sum += val;
        }

        // Normalize kernel
        for k in kernel.iter_mut() {
            *k /= kernel_sum;
        }

        let width = layer.width as usize;
        let height = layer.height as usize;

        // Intermediate buffer for horizontal pass
        let mut temp = vec![0u8; layer.pixels.len()];

        // 1. Horizontal Pass (multithreaded across rows)
        temp.par_chunks_exact_mut(width * 4)
            .zip(layer.pixels.par_chunks_exact(width * 4))
            .for_each(|(dst_row, src_row)| {
                for x in 0..width {
                    let mut r_acc = 0.0f32;
                    let mut g_acc = 0.0f32;
                    let mut b_acc = 0.0f32;
                    let mut a_acc = 0.0f32;

                    for (k_idx, k_offset) in (-radius..=radius).enumerate() {
                        let k = kernel[k_idx];
                        let sample_x = (x as i32 + k_offset).clamp(0, width as i32 - 1) as usize;
                        let s_idx = sample_x * 4;

                        r_acc += src_row[s_idx] as f32 * k;
                        g_acc += src_row[s_idx + 1] as f32 * k;
                        b_acc += src_row[s_idx + 2] as f32 * k;
                        a_acc += src_row[s_idx + 3] as f32 * k;
                    }

                    let d_idx = x * 4;
                    dst_row[d_idx] = r_acc.round().clamp(0.0, 255.0) as u8;
                    dst_row[d_idx + 1] = g_acc.round().clamp(0.0, 255.0) as u8;
                    dst_row[d_idx + 2] = b_acc.round().clamp(0.0, 255.0) as u8;
                    dst_row[d_idx + 3] = a_acc.round().clamp(0.0, 255.0) as u8;
                }
            });

        // 2. Vertical Pass (multithreaded across vertical tile chunks)
        layer.pixels.par_chunks_exact_mut(width * 4)
            .enumerate()
            .for_each(|(y, dst_row)| {
                for x in 0..width {
                    let mut r_acc = 0.0f32;
                    let mut g_acc = 0.0f32;
                    let mut b_acc = 0.0f32;
                    let mut a_acc = 0.0f32;

                    for (k_idx, k_offset) in (-radius..=radius).enumerate() {
                        let k = kernel[k_idx];
                        let sample_y = (y as i32 + k_offset).clamp(0, height as i32 - 1) as usize;
                        let s_idx = (sample_y * width + x) * 4;

                        r_acc += temp[s_idx] as f32 * k;
                        g_acc += temp[s_idx + 1] as f32 * k;
                        b_acc += temp[s_idx + 2] as f32 * k;
                        a_acc += temp[s_idx + 3] as f32 * k;
                    }

                    let d_idx = x * 4;
                    dst_row[d_idx] = r_acc.round().clamp(0.0, 255.0) as u8;
                    dst_row[d_idx + 1] = g_acc.round().clamp(0.0, 255.0) as u8;
                    dst_row[d_idx + 2] = b_acc.round().clamp(0.0, 255.0) as u8;
                    dst_row[d_idx + 3] = a_acc.round().clamp(0.0, 255.0) as u8;
                }
            });
    }
}
