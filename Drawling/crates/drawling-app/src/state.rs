#![allow(dead_code)]
use chrono::Utc;
use drawling_core::*;
use drawling_render::{CanvasCompositor, CanvasViewport};
use drawling_storage::{DrawlingArchive, DrawlingManifest, LayerMetadata};
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTool {
    Brush,
    Eraser,
    Eyedropper,
    Bucket,
    Lasso,
    MagicWand,
    RectSelect,
    EllipseSelect,
    Transform,
    Pan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GalleryViewMode {
    Grid,
    List,
}

pub struct EditorState {
    pub manifest: DrawlingManifest,
    pub stack: LayerStack,
    pub compositor: CanvasCompositor,
    pub viewport: CanvasViewport,
    pub brush_engine: BrushEngine,
    pub brush_presets: Vec<BrushPreset>,
    pub active_preset_index: usize,
    pub stabilizer: StrokeStabilizer,
    pub selection_mask: SelectionMask,
    pub symmetry_mode: SymmetryMode,
    pub history: HistoryStack,
    pub primary_color: Color,
    pub secondary_color: Color,
    pub active_tool: ActiveTool,
    pub zen_mode: bool,
    pub is_drawing: bool,
    pub current_stroke_before_pixels: Option<Vec<u8>>,
    pub lasso_points: Vec<(f32, f32)>,
    pub file_path: Option<PathBuf>,
}

impl EditorState {
    pub fn new(
        title: impl Into<String>,
        width: u32,
        height: u32,
        screen_w: f32,
        screen_h: f32,
    ) -> Self {
        let manifest = DrawlingManifest {
            id: Uuid::new_v4(),
            title: title.into(),
            author: "Duckling Artist".into(),
            version: 1,
            canvas_width: width,
            canvas_height: height,
            dpi: 300.0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            edit_duration_secs: 0,
            tags: vec!["Illustration".into()],
            layers: vec![],
            active_layer_index: 1,
        };

        let stack = LayerStack::new(width, height);
        let mut compositor = CanvasCompositor::new(width, height);
        compositor.update_composite(&stack);

        let viewport = CanvasViewport::new(screen_w, screen_h, width as f32, height as f32);

        let presets = BrushPreset::default_library();
        let default_params = presets[0].parameters.clone();

        Self {
            manifest,
            stack,
            compositor,
            viewport,
            brush_engine: BrushEngine::new(default_params),
            brush_presets: presets,
            active_preset_index: 0,
            stabilizer: StrokeStabilizer::new(StabilizerMode::LazyMouse {
                rope_length: 15.0,
                tension: 0.6,
            }),
            selection_mask: SelectionMask::new(width, height),
            symmetry_mode: SymmetryMode::None,
            history: HistoryStack::new(50),
            primary_color: Color::BLACK,
            secondary_color: Color::WHITE,
            active_tool: ActiveTool::Brush,
            zen_mode: false,
            is_drawing: false,
            current_stroke_before_pixels: None,
            lasso_points: Vec::new(),
            file_path: None,
        }
    }

    /// Select a brush preset by index
    pub fn select_preset(&mut self, index: usize) {
        if index < self.brush_presets.len() {
            self.active_preset_index = index;
            let mut params = self.brush_presets[index].parameters.clone();
            params.color = self.primary_color;
            *self.brush_engine.params_mut() = params;
        }
    }

    /// Set foreground / primary painting color
    pub fn set_primary_color(&mut self, color: Color) {
        self.primary_color = color;
        self.brush_engine.params_mut().color = color;
    }

    /// Pen down: begins drawing stroke or selection
    pub fn pointer_down(&mut self, screen_x: f32, screen_y: f32, pressure: f32) {
        let (cx, cy) = self.viewport.screen_to_canvas(screen_x, screen_y);

        match self.active_tool {
            ActiveTool::Brush | ActiveTool::Eraser => {
                let _active_layer_id = match self.stack.active_layer() {
                    Some(l) if !l.locked => l.id,
                    _ => return,
                };

                self.is_drawing = true;
                self.stabilizer.reset();
                self.brush_engine.reset_stroke();

                // Save snapshot for Undo
                if let Some(layer) = self.stack.active_layer() {
                    self.current_stroke_before_pixels = Some(layer.pixels.clone());
                }

                let raw_pt = StrokePoint::new(cx, cy, pressure);
                if let Some(stab_pt) = self.stabilizer.push(raw_pt) {
                    self.render_stroke_point(stab_pt);
                }
            }

            ActiveTool::Eyedropper => {
                if cx >= 0.0 && cx < self.manifest.canvas_width as f32
                    && cy >= 0.0 && cy < self.manifest.canvas_height as f32
                {
                    let px_idx = ((cy.floor() as u32 * self.manifest.canvas_width + cx.floor() as u32) * 4) as usize;
                    let comp = self.compositor.composite_buffer();
                    if px_idx + 4 <= comp.len() {
                        let c = Color::from_rgba8(comp[px_idx], comp[px_idx + 1], comp[px_idx + 2], comp[px_idx + 3]);
                        self.set_primary_color(c);
                    }
                }
            }

            ActiveTool::Bucket => {
                if cx >= 0.0 && cx < self.manifest.canvas_width as f32
                    && cy >= 0.0 && cy < self.manifest.canvas_height as f32
                {
                    let x = cx.floor() as u32;
                    let y = cy.floor() as u32;
                    if let Some(layer) = self.stack.active_layer_mut() {
                        let before = layer.pixels.clone();
                        let mut mask = SelectionMask::new(layer.width, layer.height);
                        mask.select_magic_wand(layer, x, y, 32.0, true);

                        let fill_rgba = self.primary_color.to_rgba8();
                        for my in 0..mask.height {
                            for mx in 0..mask.width {
                                if mask.get(mx, my) > 0 {
                                    layer.blend_pixel_onto(mx, my, fill_rgba, layer.blend_mode, 1.0);
                                }
                            }
                        }

                        let after = layer.pixels.clone();
                        self.history.push(HistoryAction::LayerPixelsChanged {
                            layer_id: layer.id,
                            before_pixels: before,
                            after_pixels: after,
                        });
                        self.compositor.update_composite(&self.stack);
                    }
                }
            }

            ActiveTool::Lasso => {
                self.is_drawing = true;
                self.lasso_points.clear();
                self.lasso_points.push((cx, cy));
            }

            ActiveTool::MagicWand => {
                if cx >= 0.0 && cx < self.manifest.canvas_width as f32
                    && cy >= 0.0 && cy < self.manifest.canvas_height as f32
                {
                    let x = cx.floor() as u32;
                    let y = cy.floor() as u32;
                    if let Some(layer) = self.stack.active_layer() {
                        self.selection_mask.clear();
                        self.selection_mask.select_magic_wand(layer, x, y, 32.0, true);
                    }
                }
            }

            _ => {}
        }
    }

    /// Pen move
    pub fn pointer_move(&mut self, screen_x: f32, screen_y: f32, pressure: f32) {
        let (cx, cy) = self.viewport.screen_to_canvas(screen_x, screen_y);

        if !self.is_drawing {
            return;
        }

        match self.active_tool {
            ActiveTool::Brush | ActiveTool::Eraser => {
                let raw_pt = StrokePoint::new(cx, cy, pressure);
                if let Some(stab_pt) = self.stabilizer.push(raw_pt) {
                    self.render_stroke_point(stab_pt);
                }
            }
            ActiveTool::Lasso => {
                self.lasso_points.push((cx, cy));
            }
            _ => {}
        }
    }

    /// Pen lift
    pub fn pointer_up(&mut self) {
        if !self.is_drawing {
            return;
        }

        self.is_drawing = false;

        match self.active_tool {
            ActiveTool::Brush | ActiveTool::Eraser => {
                if let Some(last_pt) = self.stabilizer.finish() {
                    self.render_stroke_point(last_pt);
                }

                // Commit action to HistoryStack
                if let (Some(layer), Some(before)) = (
                    self.stack.active_layer(),
                    self.current_stroke_before_pixels.take(),
                ) {
                    self.history.push(HistoryAction::LayerPixelsChanged {
                        layer_id: layer.id,
                        before_pixels: before,
                        after_pixels: layer.pixels.clone(),
                    });
                }

                self.compositor.update_composite(&self.stack);
            }

            ActiveTool::Lasso => {
                if self.lasso_points.len() >= 3 {
                    self.selection_mask.clear();
                    self.selection_mask.select_lasso(&self.lasso_points);
                }
                self.lasso_points.clear();
            }

            _ => {}
        }
    }

    fn render_stroke_point(&mut self, pt: StrokePoint) {
        // Symmetry propagation
        let sym_points = SymmetryGuide::generate_points(&self.symmetry_mode, pt);

        for sym_pt in sym_points {
            let dabs = self.brush_engine.push_point(sym_pt);
            let is_eraser = self.active_tool == ActiveTool::Eraser;

            if let Some(layer) = self.stack.active_layer_mut() {
                for mut dab in dabs {
                    if is_eraser {
                        dab.color = Color::TRANSPARENT;
                    }
                    BrushTipRasterizer::render_dab(layer, &dab);
                }
            }
        }

        self.compositor.update_composite(&self.stack);
    }

    /// Save current project to .drawling file
    pub fn save_project(&mut self, path: impl AsRef<Path>) -> Result<(), drawling_storage::StorageError> {
        let p = path.as_ref();
        self.manifest.updated_at = Utc::now();
        self.manifest.layers = self
            .stack
            .layers
            .iter()
            .map(|l| LayerMetadata {
                id: l.id,
                name: l.name.clone(),
                width: l.width,
                height: l.height,
                opacity: l.opacity,
                blend_mode: l.blend_mode,
                visible: l.visible,
                locked: l.locked,
                alpha_locked: l.alpha_locked,
                clipping_mask: l.clipping_mask,
            })
            .collect();
        self.manifest.active_layer_index = self.stack.active_index;

        DrawlingArchive::save(p, &self.manifest, &self.stack, None)?;
        self.file_path = Some(p.to_path_buf());
        Ok(())
    }
}
