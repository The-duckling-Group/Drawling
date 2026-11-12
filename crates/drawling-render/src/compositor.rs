use crate::viewport::CanvasViewport;
use drawling_core::LayerStack;
use rayon::prelude::*;

pub struct CanvasCompositor {
    composite_buffer: Vec<u8>,
    preview_stroke_buffer: Vec<u8>,
    width: u32,
    height: u32,
}

impl CanvasCompositor {
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height * 4) as usize;
        Self {
            composite_buffer: vec![0; size],
            preview_stroke_buffer: vec![0; size],
            width,
            height,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        let size = (width * height * 4) as usize;
        self.composite_buffer = vec![0; size];
        self.preview_stroke_buffer = vec![0; size];
    }

    pub fn composite_buffer(&self) -> &[u8] {
        &self.composite_buffer
    }

    /// Re-render the composite from all visible layers
    pub fn update_composite(&mut self, stack: &LayerStack) {
        if self.width != stack.width || self.height != stack.height {
            self.resize(stack.width, stack.height);
        }
        stack.compose_to_buffer(&mut self.composite_buffer);
    }

    /// Render a frame to the target screen buffer with checkerboard background and viewport transform
    pub fn render_viewport_frame(
        &self,
        viewport: &CanvasViewport,
        screen_buffer: &mut [u8],
        screen_w: u32,
        screen_h: u32,
    ) {
        assert_eq!(screen_buffer.len(), (screen_w * screen_h * 4) as usize);

        // Dark theme background color (KDE Breeze Dark #232629)
        const BG_R: u8 = 35;
        const BG_G: u8 = 38;
        const BG_B: u8 = 41;

        // Multithreaded row rendering
        screen_buffer
            .par_chunks_exact_mut(4 * screen_w as usize)
            .enumerate()
            .for_each(|(sy, row)| {
                for sx in 0..screen_w as usize {
                    let (cx, cy) = viewport.screen_to_canvas(sx as f32 + 0.5, sy as f32 + 0.5);

                    let out_idx = sx * 4;

                    if cx >= 0.0 && cx < viewport.canvas_width && cy >= 0.0 && cy < viewport.canvas_height {
                        let cxi = cx.floor() as u32;
                        let cyi = cy.floor() as u32;

                        let px_idx = ((cyi * viewport.canvas_width as u32 + cxi) * 4) as usize;
                        let src_px = [
                            self.composite_buffer[px_idx],
                            self.composite_buffer[px_idx + 1],
                            self.composite_buffer[px_idx + 2],
                            self.composite_buffer[px_idx + 3],
                        ];

                        let a = src_px[3] as f32 / 255.0;

                        // Canvas checkerboard pattern for alpha transparency
                        let check = ((cxi / 16) ^ (cyi / 16)) & 1 == 0;
                        let check_gray = if check { 220u8 } else { 180u8 };

                        let r = (src_px[0] as f32 * a + check_gray as f32 * (1.0 - a)).round() as u8;
                        let g = (src_px[1] as f32 * a + check_gray as f32 * (1.0 - a)).round() as u8;
                        let b = (src_px[2] as f32 * a + check_gray as f32 * (1.0 - a)).round() as u8;

                        row[out_idx] = r;
                        row[out_idx + 1] = g;
                        row[out_idx + 2] = b;
                        row[out_idx + 3] = 255;
                    } else {
                        // Viewport workspace background
                        row[out_idx] = BG_R;
                        row[out_idx + 1] = BG_G;
                        row[out_idx + 2] = BG_B;
                        row[out_idx + 3] = 255;
                    }
                }
            });
    }
}
