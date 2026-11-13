use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelapseFrame {
    pub frame_index: usize,
    pub timestamp: DateTime<Utc>,
    pub width: u32,
    pub height: u32,
    pub thumbnail_data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelapseRecorder {
    pub is_recording: bool,
    pub record_interval_strokes: usize,
    pub current_stroke_counter: usize,
    pub frames: Vec<TimelapseFrame>,
    pub started_at: DateTime<Utc>,
}

impl Default for TimelapseRecorder {
    fn default() -> Self {
        Self::new()
    }
}

impl TimelapseRecorder {
    pub fn new() -> Self {
        Self {
            is_recording: true,
            record_interval_strokes: 1, // Record after every stroke
            current_stroke_counter: 0,
            frames: Vec::new(),
            started_at: Utc::now(),
        }
    }

    pub fn toggle_recording(&mut self) -> bool {
        self.is_recording = !self.is_recording;
        self.is_recording
    }

    pub fn record_stroke_snapshot(&mut self, full_w: u32, full_h: u32, pixels: &[u8]) {
        if !self.is_recording {
            return;
        }

        self.current_stroke_counter += 1;
        if self.current_stroke_counter % self.record_interval_strokes != 0 {
            return;
        }

        // Downscale to 160x90 or 320x180 preview thumbnail for memory efficiency
        let thumb_w = 160u32;
        let thumb_h = 90u32;
        let mut thumb_pixels = vec![255u8; (thumb_w * thumb_h * 4) as usize];

        let scale_x = full_w as f32 / thumb_w as f32;
        let scale_y = full_h as f32 / thumb_h as f32;

        for ty in 0..thumb_h {
            let sy = ((ty as f32 * scale_y) as u32).min(full_h.saturating_sub(1));
            for tx in 0..thumb_w {
                let sx = ((tx as f32 * scale_x) as u32).min(full_w.saturating_sub(1));
                let src_idx = ((sy * full_w + sx) * 4) as usize;
                let dst_idx = ((ty * thumb_w + tx) * 4) as usize;

                if src_idx + 4 <= pixels.len() && dst_idx + 4 <= thumb_pixels.len() {
                    thumb_pixels[dst_idx..dst_idx + 4].copy_from_slice(&pixels[src_idx..src_idx + 4]);
                }
            }
        }

        self.frames.push(TimelapseFrame {
            frame_index: self.frames.len(),
            timestamp: Utc::now(),
            width: thumb_w,
            height: thumb_h,
            thumbnail_data: thumb_pixels,
        });
    }

    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    pub fn clear(&mut self) {
        self.frames.clear();
        self.current_stroke_counter = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timelapse_recorder() {
        let mut rec = TimelapseRecorder::new();
        assert!(rec.is_recording);

        let pixels = vec![128u8; 1280 * 720 * 4];
        rec.record_stroke_snapshot(1280, 720, &pixels);
        assert_eq!(rec.frame_count(), 1);

        rec.record_stroke_snapshot(1280, 720, &pixels);
        assert_eq!(rec.frame_count(), 2);
    }
}
