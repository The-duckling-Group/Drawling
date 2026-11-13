#![allow(dead_code)]
use chrono::Utc;
use drawling_core::*;
use drawling_render::{CanvasCompositor, CanvasViewport};
use drawling_storage::{DrawlingArchive, DrawlingManifest, LayerMetadata, TimelapseRecorder};
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTool {
    Brush,
    Eraser,
    Smudge,
    Eyedropper,
    Bucket,
    Line,
    Rectangle,
    Ellipse,
    Star,
    Lasso,
    MagicWand,
    QuickMask,
    FrameDivider,
    Text,
    RectSelect,
    EllipseSelect,
    Transform,
    Pan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToolOptions {
    pub fill_tolerance: f32,
    pub fill_expand: i32,
    pub fill_contiguous: bool,
    pub shape_stroke_width: Option<f32>,
    pub eyedropper_radius: u32,
    pub star_points: usize,
    pub star_inner_ratio: f32,
}

impl Default for ToolOptions {
    fn default() -> Self {
        Self {
            fill_tolerance: 0.12,
            fill_expand: 1,
            fill_contiguous: true,
            shape_stroke_width: None,
            eyedropper_radius: 1,
            star_points: 5,
            star_inner_ratio: 0.5,
        }
    }
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
    pub smudge_engine: SmudgeEngine,
    pub selection_mask: SelectionMask,
    pub symmetry_mode: SymmetryMode,
    pub history: HistoryStack,
    pub primary_color: Color,
    pub secondary_color: Color,
    pub active_tool: ActiveTool,
    pub tool_options: ToolOptions,
    pub tablet_profile: TabletCalibrationProfile,
    pub active_stylus_tool: TabletTool,
    pub is_stylus_hovering: bool,
    pub stylus_hover_pos: Option<(f32, f32)>,
    pub stylus_tilt: (f32, f32),
    pub last_barrel_buttons: StylusButtons,
    pub active_palette: ColorPalette,
    pub available_palettes: Vec<ColorPalette>,
    pub recent_colors: RecentColors,
    pub zen_mode: bool,
    pub is_drawing: bool,
    pub stroke_start_canvas: Option<(f32, f32)>,
    pub last_canvas_pos: Option<(f32, f32)>,
    pub current_stroke_before_pixels: Option<Vec<u8>>,
    pub lasso_points: Vec<(f32, f32)>,
    pub file_path: Option<PathBuf>,
    pub quick_mask_enabled: bool,
    pub timelapser: TimelapseRecorder,
    pub frame_divider: FrameDivider,
    pub text_options: TextToolOptions,
    pub active_ruler: Option<Ruler>,
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

        let default_palette = ColorPalette::neon_cyberpunk();
        let available_palettes = vec![
            ColorPalette::neon_cyberpunk(),
            ColorPalette::skin_tones(),
            ColorPalette::manga_screentone(),
            ColorPalette::pastel_anime(),
            ColorPalette::classic_oil(),
        ];

        let mut recent_colors = RecentColors::new(12);
        recent_colors.push(Color::BLACK);
        recent_colors.push(Color::WHITE);

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
            smudge_engine: SmudgeEngine::new(20.0, 0.7, 0.4),
            selection_mask: SelectionMask::new(width, height),
            symmetry_mode: SymmetryMode::None,
            history: HistoryStack::new(50),
            primary_color: Color::BLACK,
            secondary_color: Color::WHITE,
            active_tool: ActiveTool::Brush,
            tool_options: ToolOptions::default(),
            tablet_profile: TabletCalibrationProfile::xp_pen_standard(),
            active_stylus_tool: TabletTool::Pen,
            is_stylus_hovering: false,
            stylus_hover_pos: None,
            stylus_tilt: (0.0, 0.0),
            last_barrel_buttons: StylusButtons::default(),
            active_palette: default_palette,
            available_palettes,
            recent_colors,
            zen_mode: false,
            is_drawing: false,
            stroke_start_canvas: None,
            last_canvas_pos: None,
            current_stroke_before_pixels: None,
            lasso_points: Vec::new(),
            file_path: None,
            quick_mask_enabled: false,
            timelapser: TimelapseRecorder::new(),
            frame_divider: FrameDivider::default(),
            text_options: TextToolOptions::default(),
            active_ruler: None,
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
        self.recent_colors.push(color);
    }

    /// Standard pointer down (delegates with zero tilt)
    pub fn pointer_down(&mut self, screen_x: f32, screen_y: f32, pressure: f32) {
        self.pointer_down_with_details(screen_x, screen_y, pressure, 0.0, 0.0);
    }

    /// Stylus pointer down with screen coordinates (delegates to canvas after transform)
    pub fn pointer_down_with_details(
        &mut self,
        screen_x: f32,
        screen_y: f32,
        raw_pressure: f32,
        tilt_x: f32,
        tilt_y: f32,
    ) {
        let (cx, cy) = self.viewport.screen_to_canvas(screen_x, screen_y);
        self.pointer_down_canvas(cx, cy, raw_pressure, tilt_x, tilt_y);
    }

    /// Direct canvas-space pointer down (guarantees pixel-perfect alignment directly under cursor)
    pub fn pointer_down_canvas(
        &mut self,
        cx: f32,
        cy: f32,
        raw_pressure: f32,
        tilt_x: f32,
        tilt_y: f32,
    ) {
        self.stroke_start_canvas = Some((cx, cy));
        self.last_canvas_pos = Some((cx, cy));
        self.stylus_tilt = (tilt_x, tilt_y);

        let pressure = self.tablet_profile.pressure_curve.map(raw_pressure);

        let effective_tool = if self.active_stylus_tool == TabletTool::Eraser {
            ActiveTool::Eraser
        } else {
            self.active_tool
        };

        match effective_tool {
            ActiveTool::Brush | ActiveTool::Eraser => {
                let _active_layer_id = match self.stack.active_layer() {
                    Some(l) if !l.locked => l.id,
                    _ => return,
                };

                self.is_drawing = true;
                self.stabilizer.reset();
                self.brush_engine.reset_stroke();

                if let Some(layer) = self.stack.active_layer() {
                    self.current_stroke_before_pixels = Some(layer.pixels.clone());
                }

                let raw_pt = StrokePoint::with_details(cx, cy, pressure, tilt_x, tilt_y, 0);
                if let Some(stab_pt) = self.stabilizer.push(raw_pt) {
                    self.render_stroke_point(stab_pt);
                }
            }

            ActiveTool::Smudge => {
                if let Some(layer) = self.stack.active_layer() {
                    if layer.locked {
                        return;
                    }
                    self.current_stroke_before_pixels = Some(layer.pixels.clone());
                    self.smudge_engine.begin_stroke(layer, cx, cy);
                    self.is_drawing = true;
                }
            }

            ActiveTool::Eyedropper => {
                let picked = ColorPicker::sample_stack(&self.stack, cx.round() as i32, cy.round() as i32, self.tool_options.eyedropper_radius);
                if picked.a > 0.0 {
                    self.set_primary_color(picked);
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
                        FloodFill::fill(
                            layer,
                            x,
                            y,
                            self.primary_color,
                            self.tool_options.fill_tolerance,
                            self.tool_options.fill_expand,
                            self.tool_options.fill_contiguous,
                        );

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

            ActiveTool::Line | ActiveTool::Rectangle | ActiveTool::Ellipse | ActiveTool::Star => {
                if let Some(layer) = self.stack.active_layer() {
                    if !layer.locked {
                        self.is_drawing = true;
                        self.current_stroke_before_pixels = Some(layer.pixels.clone());
                    }
                }
            }

            ActiveTool::Lasso => {
                self.is_drawing = true;
                self.lasso_points.clear();
                self.lasso_points.push((cx, cy));
            }

            ActiveTool::MagicWand
                if cx >= 0.0
                    && cx < self.manifest.canvas_width as f32
                    && cy >= 0.0
                    && cy < self.manifest.canvas_height as f32 =>
            {
                let x = cx.floor() as u32;
                let y = cy.floor() as u32;
                if let Some(layer) = self.stack.active_layer() {
                    self.selection_mask.clear();
                    self.selection_mask.select_magic_wand(layer, x, y, 32.0, true);
                }
            }

            ActiveTool::QuickMask => {
                self.is_drawing = true;
                let r = (self.brush_engine.params().size * 0.5).max(1.0);
                self.selection_mask.paint_brush_dab(cx, cy, r, 255);
            }

            ActiveTool::FrameDivider | ActiveTool::Text => {
                self.is_drawing = true;
            }

            _ => {}
        }
    }

    /// Standard pointer move (delegates with zero tilt)
    pub fn pointer_move(&mut self, screen_x: f32, screen_y: f32, pressure: f32) {
        self.pointer_move_with_details(screen_x, screen_y, pressure, 0.0, 0.0, 0);
    }

    /// Stylus pointer move with screen coordinates (delegates to canvas after transform)
    pub fn pointer_move_with_details(
        &mut self,
        screen_x: f32,
        screen_y: f32,
        raw_pressure: f32,
        tilt_x: f32,
        tilt_y: f32,
        timestamp_ms: u64,
    ) {
        let (cx, cy) = self.viewport.screen_to_canvas(screen_x, screen_y);
        self.pointer_move_canvas(cx, cy, raw_pressure, tilt_x, tilt_y, timestamp_ms);
    }

    /// Direct canvas-space pointer move (tracks exact position under mouse/stylus tip)
    pub fn pointer_move_canvas(
        &mut self,
        cx: f32,
        cy: f32,
        raw_pressure: f32,
        tilt_x: f32,
        tilt_y: f32,
        timestamp_ms: u64,
    ) {
        if !self.is_drawing {
            return;
        }

        let (cx, cy) = if let Some(ruler) = &self.active_ruler {
            ruler.snap((cx, cy))
        } else {
            (cx, cy)
        };

        self.stylus_tilt = (tilt_x, tilt_y);
        let pressure = self.tablet_profile.pressure_curve.map(raw_pressure);

        let effective_tool = if self.active_stylus_tool == TabletTool::Eraser {
            ActiveTool::Eraser
        } else {
            self.active_tool
        };

        match effective_tool {
            ActiveTool::Brush | ActiveTool::Eraser => {
                let raw_pt = StrokePoint::with_details(cx, cy, pressure, tilt_x, tilt_y, timestamp_ms);
                if let Some(stab_pt) = self.stabilizer.push(raw_pt) {
                    self.render_stroke_point(stab_pt);
                }
            }
            ActiveTool::QuickMask => {
                let r = (self.brush_engine.params().size * 0.5).max(1.0);
                self.selection_mask.paint_brush_dab(cx, cy, r, 255);
            }
            ActiveTool::Smudge => {
                if let Some((px, py)) = self.last_canvas_pos {
                    if let Some(layer) = self.stack.active_layer_mut() {
                        self.smudge_engine.smudge_to(layer, px, py, cx, cy);
                        self.compositor.update_composite(&self.stack);
                    }
                }
            }
            ActiveTool::Lasso => {
                self.lasso_points.push((cx, cy));
            }
            _ => {}
        }

        self.last_canvas_pos = Some((cx, cy));
    }

    /// High-level tablet event dispatcher (handles proximity, pressure mapping,
    /// inverted eraser detection, and barrel buttons).
    pub fn handle_tablet_event(&mut self, event: TabletInputEvent) {
        match event {
            TabletInputEvent::ProximityIn { tool } => {
                self.is_stylus_hovering = true;
                self.active_stylus_tool = tool;
            }
            TabletInputEvent::ProximityOut { .. } => {
                self.is_stylus_hovering = false;
                self.stylus_hover_pos = None;
            }
            TabletInputEvent::Hover { x, y, tilt_x, tilt_y, tool } => {
                self.is_stylus_hovering = true;
                self.stylus_hover_pos = Some((x, y));
                self.stylus_tilt = (tilt_x, tilt_y);
                self.active_stylus_tool = tool;
            }
            TabletInputEvent::Down { x, y, pressure, tilt_x, tilt_y, tool, buttons } => {
                self.active_stylus_tool = tool;
                self.last_barrel_buttons = buttons;

                // Primary barrel button (Button 1): sample color directly under pen tip
                if buttons.button1 {
                    let (cx, cy) = self.viewport.screen_to_canvas(x, y);
                    let picked = ColorPicker::sample_stack(
                        &self.stack,
                        cx.round() as i32,
                        cy.round() as i32,
                        self.tool_options.eyedropper_radius,
                    );
                    if picked.a > 0.0 {
                        self.set_primary_color(picked);
                    }
                    return;
                }

                self.pointer_down_with_details(x, y, pressure, tilt_x, tilt_y);
            }
            TabletInputEvent::Motion { x, y, pressure, tilt_x, tilt_y, tool, buttons, timestamp_ms } => {
                self.active_stylus_tool = tool;
                self.last_barrel_buttons = buttons;
                self.pointer_move_with_details(x, y, pressure, tilt_x, tilt_y, timestamp_ms);
            }
            TabletInputEvent::Up { tool } => {
                self.active_stylus_tool = tool;
                self.pointer_up();
            }
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

                self.commit_layer_undo();
            }

            ActiveTool::Smudge => {
                self.commit_layer_undo();
            }

            ActiveTool::Line => {
                if let (Some((x0, y0)), Some((x1, y1))) = (self.stroke_start_canvas, self.last_canvas_pos) {
                    let w = self.tool_options.shape_stroke_width.unwrap_or(3.0);
                    if let Some(layer) = self.stack.active_layer_mut() {
                        ShapeDrawer::draw_line(layer, x0, y0, x1, y1, self.primary_color, w);
                    }
                    self.commit_layer_undo();
                }
            }

            ActiveTool::Rectangle => {
                if let (Some((x0, y0)), Some((x1, y1))) = (self.stroke_start_canvas, self.last_canvas_pos) {
                    let rx = x0.min(x1).round() as i32;
                    let ry = y0.min(y1).round() as i32;
                    let rw = (x1 - x0).abs().round() as u32;
                    let rh = (y1 - y0).abs().round() as u32;

                    if rw > 0 && rh > 0 {
                        if let Some(layer) = self.stack.active_layer_mut() {
                            ShapeDrawer::draw_rect(layer, rx, ry, rw, rh, self.primary_color, self.tool_options.shape_stroke_width);
                        }
                    }
                    self.commit_layer_undo();
                }
            }

            ActiveTool::Ellipse => {
                if let (Some((x0, y0)), Some((x1, y1))) = (self.stroke_start_canvas, self.last_canvas_pos) {
                    let cx = (x0 + x1) * 0.5;
                    let cy = (y0 + y1) * 0.5;
                    let rx = (x1 - x0).abs() * 0.5;
                    let ry = (y1 - y0).abs() * 0.5;

                    if rx > 0.0 && ry > 0.0 {
                        if let Some(layer) = self.stack.active_layer_mut() {
                            ShapeDrawer::draw_ellipse(layer, cx, cy, rx, ry, self.primary_color, self.tool_options.shape_stroke_width);
                        }
                    }
                    self.commit_layer_undo();
                }
            }

            ActiveTool::Star => {
                if let (Some((x0, y0)), Some((x1, y1))) = (self.stroke_start_canvas, self.last_canvas_pos) {
                    let cx = (x0 + x1) * 0.5;
                    let cy = (y0 + y1) * 0.5;
                    let outer_r = ((x1 - x0).hypot(y1 - y0)) * 0.5;
                    let inner_r = outer_r * self.tool_options.star_inner_ratio;

                    if outer_r > 2.0 {
                        let sw = self.tool_options.shape_stroke_width.unwrap_or(2.0);
                        if let Some(layer) = self.stack.active_layer_mut() {
                            ShapeDrawer::draw_star(layer, cx, cy, self.tool_options.star_points, inner_r, outer_r, self.primary_color, sw);
                        }
                    }
                    self.commit_layer_undo();
                }
            }

            ActiveTool::Lasso => {
                if self.lasso_points.len() >= 3 {
                    self.selection_mask.clear();
                    self.selection_mask.select_lasso(&self.lasso_points);
                }
                self.lasso_points.clear();
            }

            ActiveTool::QuickMask => {
                self.is_drawing = false;
            }

            ActiveTool::FrameDivider => {
                if let Some(layer) = self.stack.active_layer_mut() {
                    let panels = self.frame_divider.generate_grid(
                        self.manifest.canvas_width,
                        self.manifest.canvas_height,
                        2,
                        2,
                    );
                    self.frame_divider.draw_panels_on_layer(layer, &panels);
                }
                self.commit_layer_undo();
            }

            ActiveTool::Text => {
                if let Some(layer) = self.stack.active_layer_mut() {
                    let (x0, y0) = self.stroke_start_canvas.unwrap_or((150.0, 150.0));
                    SpeechBalloon::draw(
                        layer,
                        SpeechBalloonKind::Oval,
                        x0,
                        y0,
                        80.0,
                        50.0,
                        Color::WHITE,
                        Color::BLACK,
                        3.0,
                    );
                }
                self.commit_layer_undo();
            }

            _ => {}
        }

        self.stroke_start_canvas = None;
        self.last_canvas_pos = None;
    }

    fn commit_layer_undo(&mut self) {
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
        let pixels = self.compositor.composite_buffer();
        self.timelapser.record_stroke_snapshot(
            self.manifest.canvas_width,
            self.manifest.canvas_height,
            &pixels,
        );
    }

    fn render_stroke_point(&mut self, pt: StrokePoint) {
        let sym_points = SymmetryGuide::generate_points(&self.symmetry_mode, pt);

        for sym_pt in sym_points {
            let dabs = self.brush_engine.push_point(sym_pt);
            let is_eraser = self.active_tool == ActiveTool::Eraser
                || self.active_stylus_tool == TabletTool::Eraser;

            if let Some(layer) = self.stack.active_layer_mut() {
                for mut dab in dabs {
                    if is_eraser {
                        dab.radius = dab.radius.max((self.brush_engine.params().size * 0.5).max(3.0));
                        dab.opacity = 1.0;
                        BrushTipRasterizer::render_eraser_dab(layer, &dab);
                    } else {
                        BrushTipRasterizer::render_dab(layer, &dab);
                    }
                }
            }
        }

        self.compositor.update_composite(&self.stack);
    }

    /// Undo last action
    pub fn undo(&mut self) -> bool {
        let changed = self.history.undo(&mut self.stack);
        if changed {
            self.compositor.update_composite(&self.stack);
        }
        changed
    }

    /// Redo last undone action
    pub fn redo(&mut self) -> bool {
        let changed = self.history.redo(&mut self.stack);
        if changed {
            self.compositor.update_composite(&self.stack);
        }
        changed
    }

    /// Merge active layer down onto the layer beneath it
    pub fn merge_down(&mut self) {
        let active = self.stack.active_index;
        self.stack.merge_down(active);
        self.compositor.update_composite(&self.stack);
    }

    /// Resize canvas to new dimensions, keeping content and rescaling buffers
    pub fn resize_canvas(&mut self, new_width: u32, new_height: u32, dpi: f32) {
        if new_width == 0 || new_height == 0 {
            return;
        }

        let old_w = self.manifest.canvas_width;
        let old_h = self.manifest.canvas_height;

        self.manifest.canvas_width = new_width;
        self.manifest.canvas_height = new_height;
        self.manifest.dpi = dpi;

        self.stack.width = new_width;
        self.stack.height = new_height;

        for layer in &mut self.stack.layers {
            let mut new_pixels = vec![0u8; (new_width * new_height * 4) as usize];
            let copy_w = old_w.min(new_width) as usize;
            let copy_h = old_h.min(new_height) as usize;

            for y in 0..copy_h {
                let src_start = (y * old_w as usize) * 4;
                let src_end = src_start + copy_w * 4;
                let dst_start = (y * new_width as usize) * 4;
                let dst_end = dst_start + copy_w * 4;

                if src_end <= layer.pixels.len() && dst_end <= new_pixels.len() {
                    new_pixels[dst_start..dst_end].copy_from_slice(&layer.pixels[src_start..src_end]);
                }
            }

            layer.width = new_width;
            layer.height = new_height;
            layer.pixels = new_pixels;
        }

        self.compositor.resize(new_width, new_height);
        self.compositor.update_composite(&self.stack);

        self.selection_mask = SelectionMask::new(new_width, new_height);
        self.viewport = CanvasViewport::new(
            self.viewport.screen_width,
            self.viewport.screen_height,
            new_width as f32,
            new_height as f32,
        );
    }

    // --- High-Performance Multithreaded Filters & Tone Adjustments ---

    fn run_filter_on_active_layer<F>(&mut self, apply: F)
    where
        F: FnOnce(&mut Layer),
    {
        if let Some(layer) = self.stack.active_layer_mut() {
            if layer.locked {
                return;
            }
            let before = layer.pixels.clone();
            let layer_id = layer.id;

            apply(layer);

            let after = layer.pixels.clone();
            self.history.push(HistoryAction::LayerPixelsChanged {
                layer_id,
                before_pixels: before,
                after_pixels: after,
            });
            self.compositor.update_composite(&self.stack);
        }
    }

    pub fn apply_filter_gaussian_blur(&mut self, radius: f32) {
        self.run_filter_on_active_layer(|l| GaussianBlur::apply(l, radius));
    }

    pub fn apply_filter_motion_blur(&mut self, angle_deg: f32, distance: f32) {
        self.run_filter_on_active_layer(|l| ArtisticFilter::motion_blur(l, angle_deg, distance));
    }

    pub fn apply_filter_radial_blur(&mut self, cx: f32, cy: f32, amount: f32, samples: usize) {
        self.run_filter_on_active_layer(|l| ArtisticFilter::radial_blur(l, cx, cy, amount, samples));
    }

    pub fn apply_filter_unsharp_mask(&mut self, radius: f32, amount: f32, threshold: u8) {
        self.run_filter_on_active_layer(|l| ArtisticFilter::unsharp_mask(l, radius, amount, threshold));
    }

    pub fn apply_filter_halftone(&mut self, dot_size: f32, angle_deg: f32, invert: bool) {
        self.run_filter_on_active_layer(|l| ArtisticFilter::halftone(l, dot_size, angle_deg, invert));
    }

    pub fn apply_filter_chromatic_aberration(&mut self, offset_x: f32, offset_y: f32) {
        self.run_filter_on_active_layer(|l| ArtisticFilter::chromatic_aberration(l, offset_x, offset_y));
    }

    pub fn apply_filter_bloom(&mut self, threshold: f32, blur_radius: f32, intensity: f32) {
        self.run_filter_on_active_layer(|l| ArtisticFilter::bloom(l, threshold, blur_radius, intensity));
    }

    pub fn apply_filter_mosaic(&mut self, block_size: u32) {
        self.run_filter_on_active_layer(|l| ArtisticFilter::mosaic(l, block_size));
    }

    pub fn apply_filter_film_grain(&mut self, intensity: f32, monochrome: bool) {
        self.run_filter_on_active_layer(|l| ArtisticFilter::film_grain(l, intensity, monochrome));
    }

    pub fn apply_filter_posterize(&mut self, levels: u8) {
        self.run_filter_on_active_layer(|l| ArtisticFilter::posterize(l, levels));
    }

    pub fn apply_filter_threshold(&mut self, cutoff: u8) {
        self.run_filter_on_active_layer(|l| ArtisticFilter::threshold(l, cutoff));
    }

    pub fn apply_filter_levels(&mut self, in_black: u8, in_white: u8, gamma: f32, out_black: u8, out_white: u8) {
        self.run_filter_on_active_layer(|l| ArtisticFilter::levels(l, in_black, in_white, gamma, out_black, out_white));
    }

    pub fn apply_filter_brightness_contrast(&mut self, brightness: f32, contrast: f32) {
        self.run_filter_on_active_layer(|l| ColorAdjust::brightness_contrast(l, brightness, contrast));
    }

    pub fn apply_filter_hue_saturation(&mut self, hue_deg: f32, sat_scale: f32) {
        self.run_filter_on_active_layer(|l| ColorAdjust::hue_saturation(l, hue_deg, sat_scale));
    }

    pub fn apply_filter_sepia(&mut self, intensity: f32) {
        self.run_filter_on_active_layer(|l| ColorAdjust::sepia(l, intensity));
    }

    pub fn apply_filter_vignette(&mut self, radius: f32, softness: f32) {
        self.run_filter_on_active_layer(|l| ColorAdjust::vignette(l, radius, softness));
    }

    pub fn apply_filter_invert(&mut self) {
        self.run_filter_on_active_layer(ColorAdjust::invert);
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
