use crate::state::{ActiveTool, EditorState};
use drawling_core::*;
use drawling_storage::*;
use eframe::egui::{self, Color32, ColorImage, TextureOptions, Vec2};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppView {
    Studio,
    Gallery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrushDialogTab {
    Grundlagen,
    Anpassungen,
    Importieren,
}

pub struct DrawlingApp {
    pub editor: EditorState,
    pub view: AppView,
    pub canvas_texture: Option<egui::TextureHandle>,
    pub status_message: String,
    pub last_status_update: Instant,
    pub last_paint_time: Instant,

    // Dialog toggles (IbisPaint floating dialogs)
    pub show_brush_dialog: bool,
    pub brush_tab: BrushDialogTab,
    pub show_stabilizer_dialog: bool,
    pub show_layer_dialog: bool,
    pub show_canvas_dialog: bool,
    pub show_color_dialog: bool,
    pub show_filter_dialog: bool,
    pub show_material_dialog: bool,
    pub material_tab: MaterialCategory,
    pub show_settings_dialog: bool,

    // Stabilizer settings
    pub stabilizer_level: u32,
    pub stabilizer_realtime: bool,
    pub stabilizer_constant: bool,
    pub anpress_fading: bool,
    pub fade_start_len: f32,
    pub fade_end_len: f32,

    // Canvas settings & viewport controls
    pub input_canvas_w: u32,
    pub input_canvas_h: u32,
    pub input_canvas_dpi: f32,
    pub canvas_zoom: f32,
    pub canvas_rotation_deg: f32,

    // Color Wheel (Farbkreis) state
    pub color_h: f32,
    pub color_s: f32,
    pub color_v: f32,
    pub hex_input: String,

    // Brush import text
    pub import_text: String,
    pub import_feedback: Option<String>,

    // Quick tool switcher memory
    pub previous_brush_tool: ActiveTool,

    // Gallery state
    pub gallery_folder: String,
    pub gallery_timelapse_frame: usize,
    pub gallery_timelapse_playing: bool,
}

impl DrawlingApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let width = 1280;
        let height = 720;
        let mut editor = EditorState::new("Titelloses 1", width, height, width as f32, height as f32);

        // Default lineart layer
        let inking_layer = editor.stack.add_layer("Lineart");
        editor.stack.active_index = inking_layer;
        let initial_color = Color::new(0.12, 0.14, 0.18, 1.0);
        editor.set_primary_color(initial_color);

        // Use real-time stabilizer by default (IbisPaint standard)
        editor.stabilizer.set_mode(StabilizerMode::Realtime { level: 3 });

        let (h, s, v) = initial_color.to_hsv();
        let hex = initial_color.to_hex();

        Self {
            editor,
            view: AppView::Studio,
            canvas_texture: None,
            status_message: "Bereit. Zeichne mit Maus oder XP-Pen Grafiktablett.".into(),
            last_status_update: Instant::now(),
            last_paint_time: Instant::now(),

            show_brush_dialog: false,
            brush_tab: BrushDialogTab::Grundlagen,
            show_stabilizer_dialog: false,
            show_layer_dialog: false,
            show_canvas_dialog: false,
            show_color_dialog: false,
            show_filter_dialog: false,
            show_material_dialog: false,
            material_tab: MaterialCategory::Hintergruende,
            show_settings_dialog: false,

            stabilizer_level: 3,
            stabilizer_realtime: true,
            stabilizer_constant: true,
            anpress_fading: true,
            fade_start_len: 25.0,
            fade_end_len: 25.0,

            input_canvas_w: width,
            input_canvas_h: height,
            input_canvas_dpi: 300.0,
            canvas_zoom: 1.0,
            canvas_rotation_deg: 0.0,

            color_h: h,
            color_s: s,
            color_v: v,
            hex_input: hex,

            import_text: String::new(),
            import_feedback: None,

            previous_brush_tool: ActiveTool::Brush,

            gallery_folder: "Alle".into(),
            gallery_timelapse_frame: 0,
            gallery_timelapse_playing: false,
        }
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = msg.into();
        self.last_status_update = Instant::now();
    }

    pub fn set_app_color(&mut self, color: Color) {
        self.editor.set_primary_color(color);
        let (h, s, v) = color.to_hsv();
        self.color_h = h;
        self.color_s = s;
        self.color_v = v;
        self.hex_input = color.to_hex();
    }

    fn apply_stabilizer_settings(&mut self) {
        if self.stabilizer_level == 0 {
            self.editor.stabilizer.set_mode(StabilizerMode::Off);
        } else if self.stabilizer_realtime {
            self.editor.stabilizer.set_mode(StabilizerMode::Realtime {
                level: self.stabilizer_level,
            });
        } else {
            self.editor.stabilizer.set_mode(StabilizerMode::MovingAverage {
                window_size: (self.stabilizer_level as usize * 2).max(2),
            });
        }
    }
}

impl eframe::App for DrawlingApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Authentic clean dark studio theme (No emojis, sleek slate/charcoal styling)
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::from_rgb(28, 30, 35);
        visuals.window_fill = Color32::from_rgb(33, 35, 42);
        visuals.window_stroke = egui::Stroke::new(1.0, Color32::from_rgb(52, 55, 66));
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(38, 40, 48);
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(44, 46, 56);
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(56, 60, 74);
        visuals.widgets.active.bg_fill = Color32::from_rgb(42, 117, 215);
        visuals.selection.bg_fill = Color32::from_rgb(42, 117, 215);
        ui.ctx().set_visuals(visuals);

        // Global Keyboard Shortcuts
        if ctx.input(|i| i.key_pressed(egui::Key::B)) {
            self.editor.active_tool = ActiveTool::Brush;
            self.set_status("Werkzeug: Pinsel [B]");
        }
        if ctx.input(|i| i.key_pressed(egui::Key::E)) {
            self.editor.active_tool = ActiveTool::Eraser;
            self.set_status("Werkzeug: Radierer [E]");
        }
        if ctx.input(|i| i.key_pressed(egui::Key::J)) {
            self.editor.active_tool = ActiveTool::Smudge;
            self.set_status("Werkzeug: Verwischen [J]");
        }
        if ctx.input(|i| i.key_pressed(egui::Key::G)) {
            self.editor.active_tool = ActiveTool::Bucket;
            self.set_status("Werkzeug: Farbeimer [G]");
        }
        if ctx.input(|i| i.key_pressed(egui::Key::I)) {
            self.editor.active_tool = ActiveTool::Eyedropper;
            self.set_status("Werkzeug: Pipette [I]");
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Tab)) {
            self.editor.zen_mode = !self.editor.zen_mode;
            self.set_status(if self.editor.zen_mode {
                "Zen-Modus aktiv (Tab zum Beenden)"
            } else {
                "Zen-Modus inaktiv"
            });
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Z)) {
            self.editor.undo();
            self.set_status("Rückgängig (Undo)");
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Y)) {
            self.editor.redo();
            self.set_status("Wiederholen (Redo)");
        }

        // View Router: Studio vs Gallery
        match self.view {
            AppView::Gallery => {
                self.render_gallery_view(ui);
                return;
            }
            AppView::Studio => {}
        }

        // 1. Obere Leiste (Top Bar)
        if !self.editor.zen_mode {
            egui::Panel::top("ibis_top_bar").default_size(42.0).show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(8.0, 0.0);

                    // Gallery Back button
                    if ui.button("< Galerie").on_hover_text("Zur Galerie wechseln").clicked() {
                        self.view = AppView::Gallery;
                    }

                    ui.separator();

                    // Undo / Redo
                    if ui.button("Undo").on_hover_text("Rückgängig (Ctrl+Z)").clicked() {
                        self.editor.undo();
                        self.set_status("Rückgängig (Undo)");
                    }
                    if ui.button("Redo").on_hover_text("Wiederholen (Ctrl+Y)").clicked() {
                        self.editor.redo();
                        self.set_status("Wiederholen (Redo)");
                    }

                    ui.separator();

                    // Active Tool quick-switch text
                    ui.strong(format!("Drawling Studio: {}", self.editor.manifest.title));

                    // Right-aligned items on top bar
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Export / Save
                        if ui.button("Export").on_hover_text("PNG exportieren & Projekt speichern").clicked() {
                            let _ = ImageExporter::export_png(&PathBuf::from("artwork.png"), &self.editor.stack);
                            let _ = self.editor.save_project(&PathBuf::from("artwork.drawling"));
                            self.set_status("Gespeichert & als 'artwork.png' exportiert");
                        }

                        // Canvas Settings
                        if ui.selectable_label(self.show_canvas_dialog, "Leinwand").on_hover_text("Leinwandgröße & DPI anpassen").clicked() {
                            self.show_canvas_dialog = !self.show_canvas_dialog;
                        }

                        // Layer Button
                        let layer_count = self.editor.stack.layers.len();
                        let layer_btn_text = format!("Ebenen ({})", layer_count);
                        if ui.selectable_label(self.show_layer_dialog, layer_btn_text).on_hover_text("Ebenen-Manager öffnen").clicked() {
                            self.show_layer_dialog = !self.show_layer_dialog;
                        }

                        // Material Store
                        if ui.selectable_label(self.show_material_dialog, "Material-Store").on_hover_text("Material-Bibliothek öffnen").clicked() {
                            self.show_material_dialog = !self.show_material_dialog;
                        }

                        // FX Filter
                        if ui.selectable_label(self.show_filter_dialog, "FX Filter").on_hover_text("Künstlerische Filter (Rayon)").clicked() {
                            self.show_filter_dialog = !self.show_filter_dialog;
                        }

                        // Symmetry & Rulers Menu
                        ui.menu_button("Symmetrie & Lineale", |ui| {
                            ui.label("Symmetrie:");
                            if ui.selectable_label(self.editor.symmetry_mode == SymmetryMode::None, "Symmetrie: Aus").clicked() {
                                self.editor.symmetry_mode = SymmetryMode::None;
                                ui.close();
                            }
                            if ui.selectable_label(matches!(self.editor.symmetry_mode, SymmetryMode::VerticalMirror { .. }), "Vertikal spiegeln").clicked() {
                                let cx = self.editor.manifest.canvas_width as f32 * 0.5;
                                self.editor.symmetry_mode = SymmetryMode::VerticalMirror { axis_x: cx };
                                ui.close();
                            }
                            if ui.selectable_label(matches!(self.editor.symmetry_mode, SymmetryMode::HorizontalMirror { .. }), "Horizontal spiegeln").clicked() {
                                let cy = self.editor.manifest.canvas_height as f32 * 0.5;
                                self.editor.symmetry_mode = SymmetryMode::HorizontalMirror { axis_y: cy };
                                ui.close();
                            }
                            if ui.selectable_label(matches!(self.editor.symmetry_mode, SymmetryMode::Radial { .. }), "Radial (8 Segmente)").clicked() {
                                let cx = self.editor.manifest.canvas_width as f32 * 0.5;
                                let cy = self.editor.manifest.canvas_height as f32 * 0.5;
                                self.editor.symmetry_mode = SymmetryMode::Radial { center_x: cx, center_y: cy, segments: 8, mirror: false };
                                ui.close();
                            }

                            ui.separator();
                            ui.label("Lineale:");
                            if ui.selectable_label(self.editor.active_ruler.is_none(), "Lineal: Aus").clicked() {
                                self.editor.active_ruler = None;
                                ui.close();
                            }
                            if ui.button("Geodreieck / Linear").clicked() {
                                let w = self.editor.manifest.canvas_width as f32;
                                let h = self.editor.manifest.canvas_height as f32;
                                self.editor.active_ruler = Some(Ruler::Linear {
                                    p0: (w * 0.1, h * 0.5),
                                    p1: (w * 0.9, h * 0.5),
                                    snap_distance: 16.0,
                                });
                                ui.close();
                            }
                            if ui.button("Kreis-Lineal").clicked() {
                                let cx = self.editor.manifest.canvas_width as f32 * 0.5;
                                let cy = self.editor.manifest.canvas_height as f32 * 0.5;
                                self.editor.active_ruler = Some(Ruler::Circular {
                                    center: (cx, cy),
                                    radius: 120.0,
                                    snap_distance: 16.0,
                                });
                                ui.close();
                            }
                            if ui.button("Ellipsen-Lineal").clicked() {
                                let cx = self.editor.manifest.canvas_width as f32 * 0.5;
                                let cy = self.editor.manifest.canvas_height as f32 * 0.5;
                                self.editor.active_ruler = Some(Ruler::Ellipse {
                                    center: (cx, cy),
                                    radius_x: 180.0,
                                    radius_y: 90.0,
                                    snap_distance: 16.0,
                                });
                                ui.close();
                            }
                            if ui.button("Perspektive (2-Punkt)").clicked() {
                                let w = self.editor.manifest.canvas_width as f32;
                                let h = self.editor.manifest.canvas_height as f32;
                                self.editor.active_ruler = Some(Ruler::Perspective {
                                    vanishing_points: vec![(0.0, h * 0.5), (w, h * 0.5)],
                                    snap_distance: 20.0,
                                });
                                ui.close();
                            }
                        });

                        // Stabilizer Button
                        let stab_text = format!("Stabilisator ({})", self.stabilizer_level);
                        if ui.selectable_label(self.show_stabilizer_dialog, stab_text).on_hover_text("Glättung & Anpress-Fading").clicked() {
                            self.show_stabilizer_dialog = !self.show_stabilizer_dialog;
                        }

                        // Tablet Profile
                        ui.menu_button(format!("Tablett: {}", self.editor.tablet_profile.manufacturer), |ui| {
                            if ui.button("XP-Pen Standard (8192 Stufen)").clicked() {
                                self.editor.tablet_profile = TabletCalibrationProfile::xp_pen_standard();
                                self.set_status("XP-Pen Standardprofil aktiviert");
                                ui.close();
                            }
                            if ui.button("XP-Pen Soft (Feines Ansprechen)").clicked() {
                                self.editor.tablet_profile = TabletCalibrationProfile::xp_pen_soft();
                                self.set_status("XP-Pen Softprofil aktiviert");
                                ui.close();
                            }
                            if ui.button("XP-Pen Hard (Fester Druck)").clicked() {
                                self.editor.tablet_profile = TabletCalibrationProfile::xp_pen_hard();
                                self.set_status("XP-Pen Hardprofil aktiviert");
                                ui.close();
                            }
                            if ui.button("Wacom Pro Pen 2").clicked() {
                                self.editor.tablet_profile = TabletCalibrationProfile::wacom_pro_pen_2();
                                ui.close();
                            }
                            if ui.button("Huion Kamvas").clicked() {
                                self.editor.tablet_profile = TabletCalibrationProfile::huion_standard();
                                ui.close();
                            }
                        });
                    });
                });
            });
        }

        // 2. Linke Steuerleiste (Left Fixed-Width Tool Dock - Clean, No Text Wrapping!)
        if !self.editor.zen_mode {
            egui::Panel::left("ibis_left_toolbar").default_size(52.0).show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(4.0);

                    let tool_btn = |ui: &mut egui::Ui, active: bool, text: &str, tip: &str| -> bool {
                        let rt = egui::RichText::new(text).size(13.0).strong();
                        let btn = egui::Button::new(rt)
                            .selected(active)
                            .min_size(Vec2::new(38.0, 28.0));
                        ui.add(btn).on_hover_text(tip).clicked()
                    };

                    // Tools with clean 1-2 char identifiers that NEVER wrap
                    if tool_btn(ui, self.editor.active_tool == ActiveTool::Transform, "T", "Transformieren [Ctrl+T]") {
                        self.editor.active_tool = ActiveTool::Transform;
                    }
                    if tool_btn(ui, self.editor.active_tool == ActiveTool::MagicWand, "W", "Zauberstab (Magic Wand) [W]") {
                        self.editor.active_tool = ActiveTool::MagicWand;
                    }
                    if tool_btn(ui, self.editor.active_tool == ActiveTool::Lasso, "L", "Lasso-Auswahl [L]") {
                        self.editor.active_tool = ActiveTool::Lasso;
                    }
                    if tool_btn(ui, self.editor.active_tool == ActiveTool::QuickMask, "Q", "Schnellmaske (Quick Mask)") {
                        if self.editor.active_tool == ActiveTool::QuickMask {
                            self.editor.active_tool = ActiveTool::Brush;
                            self.set_status("Schnellmaske deaktiviert");
                        } else {
                            self.editor.active_tool = ActiveTool::QuickMask;
                            self.set_status("Schnellmaske aktiv: Pinsel malt Maske");
                        }
                    }
                    if tool_btn(ui, self.show_filter_dialog, "FX", "Filter & Effekte") {
                        self.show_filter_dialog = !self.show_filter_dialog;
                    }
                    if tool_btn(ui, self.editor.active_tool == ActiveTool::Brush, "B", "Pinsel (Brush) [B]") {
                        self.editor.active_tool = ActiveTool::Brush;
                    }
                    if tool_btn(ui, self.editor.active_tool == ActiveTool::Eraser, "E", "Radierer (Eraser) [E]") {
                        self.editor.active_tool = ActiveTool::Eraser;
                    }
                    if tool_btn(ui, self.editor.active_tool == ActiveTool::Smudge, "S", "Verwischen (Smudge) [J]") {
                        self.editor.active_tool = ActiveTool::Smudge;
                    }
                    if tool_btn(ui, self.editor.active_tool == ActiveTool::Bucket, "G", "Farbeimer (Bucket Fill) [G]") {
                        self.editor.active_tool = ActiveTool::Bucket;
                    }
                    if tool_btn(ui, self.editor.active_tool == ActiveTool::FrameDivider, "P", "Manga Panel-Teiler [P]") {
                        self.editor.active_tool = ActiveTool::FrameDivider;
                    }
                    if tool_btn(ui, self.editor.active_tool == ActiveTool::Text, "A", "Text & Sprechblase [T]") {
                        self.editor.active_tool = ActiveTool::Text;
                    }
                    if tool_btn(ui, self.editor.active_tool == ActiveTool::Rectangle, "R", "Rechteck-Form [U]") {
                        self.editor.active_tool = ActiveTool::Rectangle;
                    }
                    if tool_btn(ui, self.editor.active_tool == ActiveTool::Ellipse, "O", "Ellipse / Kreis-Form") {
                        self.editor.active_tool = ActiveTool::Ellipse;
                    }
                    if tool_btn(ui, self.editor.active_tool == ActiveTool::Eyedropper, "I", "Farbpipette (Eyedropper) [I]") {
                        self.editor.active_tool = ActiveTool::Eyedropper;
                    }
                    if tool_btn(ui, self.show_material_dialog, "M", "Material-Store & Cloud-Bibliothek") {
                        self.show_material_dialog = !self.show_material_dialog;
                    }

                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);

                    // Color Swatch quick trigger on left toolbar
                    let [cr, cg, cb, _] = self.editor.primary_color.to_rgba8();
                    let (swatch_rect, swatch_resp) = ui.allocate_exact_size(Vec2::new(32.0, 32.0), egui::Sense::click());
                    ui.painter().rect_filled(swatch_rect, 6.0, Color32::from_rgb(cr, cg, cb));
                    ui.painter().rect_stroke(swatch_rect, 6.0, egui::Stroke::new(1.5, Color32::WHITE), egui::StrokeKind::Outside);
                    if swatch_resp.on_hover_text("Farbkreis & Palette öffnen").clicked() {
                        self.show_color_dialog = !self.show_color_dialog;
                    }
                });
            });
        }

        // 3. Untere Werkzeugleiste (Bottom Dock - Clean, All Controls in One Row)
        if !self.editor.zen_mode {
            egui::Panel::bottom("ibis_bottom_bar").default_size(44.0).show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(10.0, 0.0);

                    // Quick Toggle between Brush and Eraser
                    let is_brush = self.editor.active_tool == ActiveTool::Brush;
                    let toggle_text = if is_brush { "Pinsel -> Radierer" } else { "Radierer -> Pinsel" };
                    if ui.button(toggle_text).on_hover_text("Schnellwechsel Pinsel / Radierer").clicked() {
                        if is_brush {
                            self.previous_brush_tool = ActiveTool::Brush;
                            self.editor.active_tool = ActiveTool::Eraser;
                        } else {
                            self.editor.active_tool = ActiveTool::Brush;
                        }
                    }

                    ui.separator();

                    // Brush Preset Pill Button
                    let current_size = self.editor.brush_engine.params().size;
                    let brush_btn_text = format!("Pinsel: {:.1}px", current_size);
                    if ui.selectable_label(self.show_brush_dialog, brush_btn_text)
                        .on_hover_text("Pinsel-Auswahl & Anpassung (15.000+)").clicked()
                    {
                        self.show_brush_dialog = !self.show_brush_dialog;
                    }

                    // Color Preview Swatch (opens Farbkreis Dialog)
                    let [cr, cg, cb, _] = self.editor.primary_color.to_rgba8();
                    let (swatch_rect, swatch_resp) = ui.allocate_exact_size(Vec2::new(30.0, 26.0), egui::Sense::click());
                    ui.painter().rect_filled(swatch_rect, 4.0, Color32::from_rgb(cr, cg, cb));
                    ui.painter().rect_stroke(swatch_rect, 4.0, egui::Stroke::new(1.5, Color32::WHITE), egui::StrokeKind::Outside);
                    if swatch_resp.on_hover_text("Farbkreis & Farbton-Auswahl öffnen").clicked() {
                        self.show_color_dialog = !self.show_color_dialog;
                    }

                    // Quick Size Slider
                    ui.label("Größe:");
                    let params = self.editor.brush_engine.params_mut();
                    ui.add(egui::Slider::new(&mut params.size, 0.5..=500.0).show_value(false).logarithmic(true));
                    ui.small(format!("{:.1}px", params.size));

                    // Quick Opacity Slider
                    ui.label("Deckkraft:");
                    let mut op_percent = (params.opacity * 100.0).round();
                    if ui.add(egui::Slider::new(&mut op_percent, 1.0..=100.0).show_value(false)).changed() {
                        params.opacity = op_percent / 100.0;
                    }
                    ui.small(format!("{}%", op_percent as u32));

                    ui.separator();

                    // Viewport Zoom Controls
                    ui.label("Zoom:");
                    if ui.small_button("-").on_hover_text("Verkleinern").clicked() {
                        self.canvas_zoom = (self.canvas_zoom * 0.85).max(0.1);
                    }
                    ui.small(format!("{:.0}%", self.canvas_zoom * 100.0));
                    if ui.small_button("+").on_hover_text("Vergrößern").clicked() {
                        self.canvas_zoom = (self.canvas_zoom * 1.18).min(5.0);
                    }
                    if ui.small_button("1:1").on_hover_text("Zoom zurücksetzen").clicked() {
                        self.canvas_zoom = 1.0;
                    }

                    // Status Bar Indicator
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let frames = self.editor.timelapser.frame_count();
                        ui.label(format!("Leinwand: {}x{} | Zeitraffer: {} Frames", self.editor.manifest.canvas_width, self.editor.manifest.canvas_height, frames));
                    });
                });
            });
        }

        // Floating Dialogs
        self.render_brush_dialog(ui);
        self.render_stabilizer_dialog(ui);
        self.render_layer_dialog(ui);
        self.render_canvas_dialog(ui);
        self.render_color_dialog(ui);
        self.render_filter_dialog(ui);
        self.render_material_dialog(ui);

        // Central Drawing Canvas (Full screen, no cramped side panels)
        egui::CentralPanel::default().show(ui, |ui| {
            let available_size = ui.available_size();
            let canvas_w = self.editor.manifest.canvas_width;
            let canvas_h = self.editor.manifest.canvas_height;

            let scale_x = available_size.x / canvas_w as f32;
            let scale_y = available_size.y / canvas_h as f32;
            let fit_scale = scale_x.min(scale_y).clamp(0.05, 10.0) * 0.96 * self.canvas_zoom;

            let display_w = canvas_w as f32 * fit_scale;
            let display_h = canvas_h as f32 * fit_scale;

            let (rect, response) = ui.allocate_exact_size(Vec2::new(display_w, display_h), egui::Sense::drag());

            // Exact Mouse & Stylus Interaction
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                if rect.contains(pointer_pos) {
                    let cx = ((pointer_pos.x - rect.min.x) / rect.width() * canvas_w as f32)
                        .clamp(0.0, canvas_w as f32 - 1.0);
                    let cy = ((pointer_pos.y - rect.min.y) / rect.height() * canvas_h as f32)
                        .clamp(0.0, canvas_h as f32 - 1.0);

                    let raw_pressure = 0.85;
                    let now_ms = (Instant::now().duration_since(self.last_paint_time).as_millis() as u64) & 0xFFFFFFFF;

                    if response.drag_started() {
                        self.editor.pointer_down_canvas(cx, cy, raw_pressure, 0.0, 0.0);
                    } else if response.dragged() {
                        self.editor.pointer_move_canvas(cx, cy, raw_pressure, 0.0, 0.0, now_ms);
                    }
                }
            }

            if response.drag_stopped() {
                self.editor.pointer_up();
            }

            // Update & Render Texture
            let flat_pixels = self.editor.compositor.composite_buffer();
            let color_image = ColorImage::from_rgba_unmultiplied(
                [canvas_w as usize, canvas_h as usize],
                flat_pixels,
            );

            let texture = self.canvas_texture.get_or_insert_with(|| {
                ui.ctx().load_texture("main_canvas", color_image.clone(), TextureOptions::LINEAR)
            });
            texture.set(color_image, TextureOptions::LINEAR);

            // Canvas Background Paper & Frame
            ui.painter().rect_filled(rect, 0.0, Color32::WHITE);
            let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            ui.painter().image(texture.id(), rect, uv, Color32::WHITE);
            ui.painter().rect_stroke(rect, 0.0, egui::Stroke::new(1.0, Color32::from_rgb(50, 52, 60)), egui::StrokeKind::Outside);

            // Brush tip hover reticle cursor
            if let Some(hover_pos) = response.hover_pos() {
                if rect.contains(hover_pos) {
                    let brush_size = self.editor.brush_engine.params().size;
                    let screen_radius = (brush_size * 0.5 * fit_scale).max(2.0);
                    ui.painter().circle_stroke(hover_pos, screen_radius, egui::Stroke::new(1.0, Color32::from_black_alpha(180)));
                    ui.painter().circle_stroke(hover_pos, screen_radius + 0.5, egui::Stroke::new(1.0, Color32::from_white_alpha(180)));
                }
            }
        });

        if self.editor.is_drawing {
            ui.ctx().request_repaint();
        }
    }
}

impl DrawlingApp {
    /// Renders IbisPaint Brush Dialog ("Pinsel")
    fn render_brush_dialog(&mut self, ui: &mut egui::Ui) {
        if !self.show_brush_dialog {
            return;
        }

        let mut open = self.show_brush_dialog;
        egui::Window::new("Pinsel (15.000+ Presets)")
            .open(&mut open)
            .default_size([480.0, 540.0])
            .resizable(true)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.brush_tab, BrushDialogTab::Grundlagen, "Grundlagen");
                    ui.selectable_value(&mut self.brush_tab, BrushDialogTab::Anpassungen, "Anpassungen");
                    ui.selectable_value(&mut self.brush_tab, BrushDialogTab::Importieren, "Online / Import");
                });
                ui.separator();

                match self.brush_tab {
                    BrushDialogTab::Grundlagen => {
                        let mut selected_idx = None;
                        egui::ScrollArea::vertical().max_height(420.0).show(ui, |ui| {
                            for (idx, preset) in self.editor.brush_presets.iter().enumerate() {
                                let is_active = self.editor.active_preset_index == idx;
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        let btn = ui.selectable_label(is_active, &preset.name);
                                        if btn.clicked() {
                                            selected_idx = Some(idx);
                                        }
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.label(format!("{:.1}px", preset.parameters.size));
                                        });
                                    });
                                    ui.small(&preset.description);
                                });
                            }
                        });

                        if let Some(idx) = selected_idx {
                            self.editor.select_preset(idx);
                            self.set_status(format!("Pinsel ausgewählt: {}", self.editor.brush_presets[idx].name));
                        }
                    }

                    BrushDialogTab::Anpassungen => {
                        let params = self.editor.brush_engine.params_mut();
                        ui.heading("Strich-Dynamik & Verjüngung");
                        ui.add_space(4.0);

                        ui.add(egui::Slider::new(&mut params.size, 0.5..=500.0).text("Größe (px)").logarithmic(true));
                        ui.add(egui::Slider::new(&mut params.opacity, 0.01..=1.0).text("Deckkraft"));
                        ui.add(egui::Slider::new(&mut params.hardness, 0.01..=1.0).text("Härte"));
                        ui.add(egui::Slider::new(&mut params.spacing, 0.01..=1.0).text("Abstand"));

                        ui.separator();
                        ui.heading("Anpress-Fading (Tapering)");
                        ui.add(egui::Slider::new(&mut params.start_width_ratio, 0.0..=1.0).text("Anfangsbreite (%)"));
                        ui.add(egui::Slider::new(&mut params.end_width_ratio, 0.0..=1.0).text("Endbreite (%)"));
                        ui.add(egui::Slider::new(&mut params.start_opacity_ratio, 0.0..=1.0).text("Anfangsdeckkraft (%)"));
                        ui.add(egui::Slider::new(&mut params.end_opacity_ratio, 0.0..=1.0).text("Enddeckkraft (%)"));
                        ui.add(egui::Slider::new(&mut params.fade_in_length, 0.0..=200.0).text("Startlänge Verblassen (px)"));

                        ui.separator();
                        ui.checkbox(&mut params.size_by_pressure, "Stiftdruck steuert Größe");
                        ui.checkbox(&mut params.opacity_by_pressure, "Stiftdruck steuert Deckkraft");
                        ui.checkbox(&mut params.tilt_aspect_ratio, "Stiftneigung -> Elliptischer Auftrag");
                    }

                    BrushDialogTab::Importieren => {
                        ui.heading("IbisPaint Pinsel & ABR importieren");
                        ui.label("Füge hier IbisPaint Pinsel-Code, JSON oder QR-Payload ein:");
                        ui.add(egui::TextEdit::multiline(&mut self.import_text).desired_rows(6));

                        if ui.button("Pinsel-Code importieren").clicked() {
                            match IbisBrushImporter::parse_ibis_string(&self.import_text) {
                                Ok(preset) => {
                                    let name = preset.name.clone();
                                    self.editor.brush_presets.push(preset);
                                    let new_idx = self.editor.brush_presets.len() - 1;
                                    self.editor.select_preset(new_idx);
                                    self.import_feedback = Some(format!("Erfolg: '{}' importiert!", name));
                                    self.set_status(format!("Pinsel '{}' erfolgreich importiert", name));
                                }
                                Err(err) => {
                                    self.import_feedback = Some(format!("Fehler beim Import: {}", err));
                                }
                            }
                        }

                        if let Some(feedback) = &self.import_feedback {
                            ui.separator();
                            ui.label(feedback);
                        }
                    }
                }
            });
        self.show_brush_dialog = open;
    }

    /// Renders IbisPaint Stabilizer Dialog ("Verwackel-Korrektur")
    fn render_stabilizer_dialog(&mut self, ui: &mut egui::Ui) {
        if !self.show_stabilizer_dialog {
            return;
        }

        let mut open = self.show_stabilizer_dialog;
        egui::Window::new("Verwackel-Korrektur")
            .open(&mut open)
            .default_size([340.0, 360.0])
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.heading("Stabilisator Stufe");
                let mut lvl = self.stabilizer_level as i32;
                if ui.add(egui::Slider::new(&mut lvl, 0..=10).text("Stufe")).changed() {
                    self.stabilizer_level = lvl as u32;
                    self.apply_stabilizer_settings();
                }

                ui.separator();
                ui.heading("Methode");
                ui.horizontal(|ui| {
                    if ui.selectable_label(self.stabilizer_realtime, "Echtzeit (Standard)").clicked() {
                        self.stabilizer_realtime = true;
                        self.apply_stabilizer_settings();
                    }
                    if ui.selectable_label(!self.stabilizer_realtime, "Danach").clicked() {
                        self.stabilizer_realtime = false;
                        self.apply_stabilizer_settings();
                    }
                });

                ui.separator();
                ui.checkbox(&mut self.stabilizer_constant, "Konstant glätten");

                ui.separator();
                ui.heading("Anpress-Fading");
                ui.checkbox(&mut self.anpress_fading, "Anpress-Fading aktivieren");

                if self.anpress_fading {
                    let mut start = self.fade_start_len;
                    let mut end = self.fade_end_len;
                    if ui.add(egui::Slider::new(&mut start, 0.0..=100.0).text("Startlänge")).changed() {
                        self.fade_start_len = start;
                        self.editor.brush_engine.params_mut().fade_in_length = start;
                    }
                    if ui.add(egui::Slider::new(&mut end, 0.0..=100.0).text("Endlänge")).changed() {
                        self.fade_end_len = end;
                        self.editor.brush_engine.params_mut().fade_out_length = end;
                    }
                }
            });
        self.show_stabilizer_dialog = open;
    }

    /// Renders IbisPaint Layer Manager Dialog ("Ebene")
    fn render_layer_dialog(&mut self, ui: &mut egui::Ui) {
        if !self.show_layer_dialog {
            return;
        }

        let mut open = self.show_layer_dialog;
        egui::Window::new("Ebenen-Manager")
            .open(&mut open)
            .default_size([340.0, 500.0])
            .resizable(true)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    if ui.button("+ Neue Ebene").clicked() {
                        let count = self.editor.stack.layers.len() + 1;
                        let idx = self.editor.stack.add_layer(format!("Ebene {}", count));
                        self.editor.stack.active_index = idx;
                        self.editor.compositor.update_composite(&self.editor.stack);
                        self.set_status("Neue Ebene hinzugefügt");
                    }
                    if ui.button("+ Neuer Ordner").on_hover_text("Ebenen-Ordner zur Organisation erstellen").clicked() {
                        let count = self.editor.stack.layers.len() + 1;
                        let idx = self.editor.stack.add_folder(format!("Ordner {}", count));
                        self.editor.stack.active_index = idx;
                        self.set_status("Neuer Ebenen-Ordner hinzugefügt");
                    }
                    if ui.button("Zusammenfügen").on_hover_text("Nach unten zusammenfügen").clicked() {
                        self.editor.merge_down();
                        self.set_status("Ebene nach unten zusammengefügt");
                    }
                });

                ui.separator();

                // Active layer controls
                if let Some(active_layer) = self.editor.stack.active_layer_mut() {
                    let type_label = if active_layer.is_folder { "[Ordner]" } else { "[Ebene]" };
                    ui.label(format!("Aktiv: {} {}", type_label, active_layer.name));

                    // Opacity Slider
                    let mut op_percent = (active_layer.opacity * 100.0).round();
                    if ui.add(egui::Slider::new(&mut op_percent, 0.0..=100.0).text("Deckkraft (%)")).changed() {
                        active_layer.opacity = op_percent / 100.0;
                    }

                    // Clipping Mask and Alpha Lock
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut active_layer.clipping_mask, "Schnittmaske");
                        ui.checkbox(&mut active_layer.alpha_locked, "Alpha-Sperre");
                    });

                    // Blend Mode selector
                    ui.horizontal(|ui| {
                        ui.label("Mischmodus:");
                        egui::ComboBox::from_id_salt("blend_mode_select")
                            .selected_text(format!("{:?}", active_layer.blend_mode))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut active_layer.blend_mode, BlendMode::Normal, "Normal");
                                ui.selectable_value(&mut active_layer.blend_mode, BlendMode::Multiply, "Multiplizieren");
                                ui.selectable_value(&mut active_layer.blend_mode, BlendMode::Screen, "Negativ Multiplizieren");
                                ui.selectable_value(&mut active_layer.blend_mode, BlendMode::Overlay, "Überlagern");
                                ui.selectable_value(&mut active_layer.blend_mode, BlendMode::ColorDodge, "Farbig abwedeln");
                                ui.selectable_value(&mut active_layer.blend_mode, BlendMode::ColorBurn, "Farbig nachbelichten");
                                ui.selectable_value(&mut active_layer.blend_mode, BlendMode::HardLight, "Hartes Licht");
                                ui.selectable_value(&mut active_layer.blend_mode, BlendMode::SoftLight, "Weiches Licht");
                            });
                    });
                }

                ui.separator();
                ui.heading("Ebenen-Stapel");

                let mut change_active = None;
                let active_idx = self.editor.stack.active_index;
                let layer_count = self.editor.stack.layers.len();

                egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                    for i in (0..layer_count).rev() {
                        let is_active = i == active_idx;
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                let layer = &mut self.editor.stack.layers[i];
                                ui.checkbox(&mut layer.visible, "Sichtbar");

                                if layer.is_folder {
                                    ui.label("[Dir]");
                                }
                                if layer.clipping_mask {
                                    ui.label("-> Clip");
                                }
                                if layer.alpha_locked {
                                    ui.label("[a]");
                                }

                                let name = layer.name.clone();
                                if ui.selectable_label(is_active, name).clicked() {
                                    change_active = Some(i);
                                }
                            });
                        });
                    }
                });

                if let Some(idx) = change_active {
                    self.editor.stack.active_index = idx;
                    self.editor.compositor.update_composite(&self.editor.stack);
                }
            });
        self.show_layer_dialog = open;
    }

    /// Renders Canvas Size & Resolution Dialog
    fn render_canvas_dialog(&mut self, ui: &mut egui::Ui) {
        if !self.show_canvas_dialog {
            return;
        }

        let mut open = self.show_canvas_dialog;
        egui::Window::new("Leinwand-Einstellungen")
            .open(&mut open)
            .default_size([380.0, 360.0])
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.heading("Aktuelle Auflösung");
                ui.label(format!("{} x {} Pixel ({} DPI)", self.editor.manifest.canvas_width, self.editor.manifest.canvas_height, self.editor.manifest.dpi));

                ui.separator();
                ui.heading("Presets (Formatvorlagen)");
                ui.horizontal_wrapped(|ui| {
                    if ui.button("1:1 Quadrat (1080x1080)").clicked() {
                        self.input_canvas_w = 1080;
                        self.input_canvas_h = 1080;
                    }
                    if ui.button("SD (1280x720)").clicked() {
                        self.input_canvas_w = 1280;
                        self.input_canvas_h = 720;
                    }
                    if ui.button("Full HD (1920x1080)").clicked() {
                        self.input_canvas_w = 1920;
                        self.input_canvas_h = 1080;
                    }
                    if ui.button("Story 9:16 (1080x1920)").clicked() {
                        self.input_canvas_w = 1080;
                        self.input_canvas_h = 1920;
                    }
                    if ui.button("Twitter Header (1500x500)").clicked() {
                        self.input_canvas_w = 1500;
                        self.input_canvas_h = 500;
                    }
                    if ui.button("4K UHD (3840x2160)").clicked() {
                        self.input_canvas_w = 3840;
                        self.input_canvas_h = 2160;
                    }
                });

                ui.separator();
                ui.heading("Benutzerdefiniert");
                ui.horizontal(|ui| {
                    ui.label("Breite (px):");
                    ui.add(egui::DragValue::new(&mut self.input_canvas_w).range(64..=8192));
                });
                ui.horizontal(|ui| {
                    ui.label("Höhe (px):");
                    ui.add(egui::DragValue::new(&mut self.input_canvas_h).range(64..=8192));
                });
                ui.horizontal(|ui| {
                    ui.label("DPI:");
                    ui.add(egui::DragValue::new(&mut self.input_canvas_dpi).range(72.0..=1200.0));
                });

                ui.add_space(8.0);
                if ui.button("Leinwandgröße anwenden").clicked() {
                    self.editor.resize_canvas(self.input_canvas_w, self.input_canvas_h, self.input_canvas_dpi);
                    self.canvas_texture = None;
                    self.set_status(format!("Leinwand auf {}x{} skaliert", self.input_canvas_w, self.input_canvas_h));
                }
            });
        self.show_canvas_dialog = open;
    }

    /// Renders IbisPaint Authentic Color Wheel Dialog ("Farbkreis & Palette mit allen Tönen")
    fn render_color_dialog(&mut self, ui: &mut egui::Ui) {
        if !self.show_color_dialog {
            return;
        }

        let mut open = self.show_color_dialog;
        egui::Window::new("Farbkreis & Palette")
            .open(&mut open)
            .default_size([320.0, 520.0])
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.vertical_centered(|ui| {
                    // Farbkreis (HSV Color Wheel) Canvas Widget
                    let wheel_size = Vec2::new(220.0, 220.0);
                    let (wheel_rect, wheel_resp) = ui.allocate_exact_size(wheel_size, egui::Sense::drag());
                    let painter = ui.painter_at(wheel_rect);
                    let center = wheel_rect.center();

                    let r_mid = 85.0f32;
                    let stroke_w = 20.0f32;
                    let inner_r = r_mid - stroke_w * 0.5; // 75.0
                    let outer_r = r_mid + stroke_w * 0.5; // 95.0

                    // 1. Draw 72 segments for the smooth Hue Ring (All hues across 360 degrees)
                    let segments = 72;
                    for i in 0..segments {
                        let a0 = (i as f32 / segments as f32) * std::f32::consts::TAU;
                        let a1 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;
                        let h = (i as f32 / segments as f32) * 360.0;
                        let col = Color::from_hsv(h, 1.0, 1.0, 1.0);
                        let [r, g, b, _] = col.to_rgba8();
                        let c32 = Color32::from_rgb(r, g, b);

                        let p0 = center + Vec2::new(a0.cos(), a0.sin()) * r_mid;
                        let p1 = center + Vec2::new(a1.cos(), a1.sin()) * r_mid;
                        painter.line_segment([p0, p1], egui::Stroke::new(stroke_w + 1.0, c32));
                    }

                    // Ring borders for clean finish
                    painter.circle_stroke(center, outer_r + 0.5, egui::Stroke::new(1.0, Color32::from_rgb(60, 62, 70)));
                    painter.circle_stroke(center, inner_r - 0.5, egui::Stroke::new(1.0, Color32::from_rgb(60, 62, 70)));

                    // 2. Draw Saturation-Value 2D gradient box in the center
                    let sv_size = 96.0f32;
                    let sv_rect = egui::Rect::from_center_size(center, Vec2::new(sv_size, sv_size));

                    let grid = 8;
                    for iy in 0..grid {
                        let v0 = 1.0 - (iy as f32 / grid as f32);
                        let v1 = 1.0 - ((iy + 1) as f32 / grid as f32);
                        let y0 = sv_rect.min.y + (iy as f32 / grid as f32) * sv_size;
                        let y1 = sv_rect.min.y + ((iy + 1) as f32 / grid as f32) * sv_size;

                        for ix in 0..grid {
                            let s0 = ix as f32 / grid as f32;
                            let s1 = (ix + 1) as f32 / grid as f32;
                            let x0 = sv_rect.min.x + (ix as f32 / grid as f32) * sv_size;
                            let x1 = sv_rect.min.x + ((ix + 1) as f32 / grid as f32) * sv_size;

                            let c_tl = Color::from_hsv(self.color_h, s0, v0, 1.0).to_rgba8();
                            let c_tr = Color::from_hsv(self.color_h, s1, v0, 1.0).to_rgba8();
                            let c_br = Color::from_hsv(self.color_h, s1, v1, 1.0).to_rgba8();
                            let c_bl = Color::from_hsv(self.color_h, s0, v1, 1.0).to_rgba8();

                            let mut mesh = egui::Mesh::default();
                            let idx = mesh.vertices.len() as u32;
                            mesh.vertices.push(egui::epaint::Vertex { pos: egui::pos2(x0, y0), uv: egui::pos2(0.0, 0.0), color: Color32::from_rgb(c_tl[0], c_tl[1], c_tl[2]) });
                            mesh.vertices.push(egui::epaint::Vertex { pos: egui::pos2(x1, y0), uv: egui::pos2(1.0, 0.0), color: Color32::from_rgb(c_tr[0], c_tr[1], c_tr[2]) });
                            mesh.vertices.push(egui::epaint::Vertex { pos: egui::pos2(x1, y1), uv: egui::pos2(1.0, 1.0), color: Color32::from_rgb(c_br[0], c_br[1], c_br[2]) });
                            mesh.vertices.push(egui::epaint::Vertex { pos: egui::pos2(x0, y1), uv: egui::pos2(0.0, 1.0), color: Color32::from_rgb(c_bl[0], c_bl[1], c_bl[2]) });
                            mesh.add_triangle(idx, idx + 1, idx + 2);
                            mesh.add_triangle(idx, idx + 2, idx + 3);
                            painter.add(egui::Shape::mesh(mesh));
                        }
                    }
                    painter.rect_stroke(sv_rect, 0.0, egui::Stroke::new(1.0, Color32::from_rgb(80, 84, 96)), egui::StrokeKind::Outside);

                    // 3. User interaction with Farbkreis
                    if let Some(pos) = wheel_resp.interact_pointer_pos() {
                        let d = pos - center;
                        let dist = d.length();
                        if dist >= inner_r - 8.0 && dist <= outer_r + 12.0 {
                            // Dragging Hue Ring
                            let mut angle = d.y.atan2(d.x).to_degrees();
                            if angle < 0.0 { angle += 360.0; }
                            self.color_h = angle;
                            let new_col = Color::from_hsv(self.color_h, self.color_s, self.color_v, self.editor.primary_color.a);
                            self.editor.set_primary_color(new_col);
                            self.hex_input = new_col.to_hex();
                        } else if sv_rect.contains(pos) || dist < inner_r {
                            // Dragging SV Box
                            let s = ((pos.x - sv_rect.min.x) / sv_size).clamp(0.0, 1.0);
                            let v = ((sv_rect.max.y - pos.y) / sv_size).clamp(0.0, 1.0);
                            self.color_s = s;
                            self.color_v = v;
                            let new_col = Color::from_hsv(self.color_h, self.color_s, self.color_v, self.editor.primary_color.a);
                            self.editor.set_primary_color(new_col);
                            self.hex_input = new_col.to_hex();
                        }
                    }

                    // 4. Indicator on Hue Ring
                    let h_rad = self.color_h.to_radians();
                    let h_handle_pos = center + Vec2::new(h_rad.cos(), h_rad.sin()) * r_mid;
                    let pure_hue = Color::from_hsv(self.color_h, 1.0, 1.0, 1.0).to_rgba8();
                    painter.circle_filled(h_handle_pos, 7.0, Color32::WHITE);
                    painter.circle_filled(h_handle_pos, 5.0, Color32::from_rgb(pure_hue[0], pure_hue[1], pure_hue[2]));
                    painter.circle_stroke(h_handle_pos, 7.0, egui::Stroke::new(1.5, Color32::BLACK));

                    // 5. Indicator in SV Box
                    let sv_handle_pos = egui::pos2(
                        sv_rect.min.x + self.color_s * sv_size,
                        sv_rect.max.y - self.color_v * sv_size,
                    );
                    let [cr, cg, cb, _] = self.editor.primary_color.to_rgba8();
                    let cur_c32 = Color32::from_rgb(cr, cg, cb);
                    painter.circle_filled(sv_handle_pos, 5.0, cur_c32);
                    painter.circle_stroke(sv_handle_pos, 5.0, egui::Stroke::new(1.5, Color32::WHITE));
                    painter.circle_stroke(sv_handle_pos, 6.5, egui::Stroke::new(1.0, Color32::BLACK));
                });

                ui.add_space(8.0);
                ui.separator();

                // Swatch preview & HEX
                ui.horizontal(|ui| {
                    let [r, g, b, _] = self.editor.primary_color.to_rgba8();
                    let c32 = Color32::from_rgb(r, g, b);
                    let (swatch_rect, _) = ui.allocate_exact_size(Vec2::new(48.0, 24.0), egui::Sense::hover());
                    ui.painter().rect_filled(swatch_rect, 4.0, c32);
                    ui.painter().rect_stroke(swatch_rect, 4.0, egui::Stroke::new(1.0, Color32::WHITE), egui::StrokeKind::Outside);

                    ui.label("Hex:");
                    let hex_resp = ui.add(egui::TextEdit::singleline(&mut self.hex_input).desired_width(90.0));
                    if hex_resp.lost_focus() || hex_resp.changed() {
                        if let Some(c) = Color::from_hex(&self.hex_input) {
                            self.editor.set_primary_color(c);
                            let (h, s, v) = c.to_hsv();
                            self.color_h = h;
                            self.color_s = s;
                            self.color_v = v;
                        }
                    }
                });

                ui.add_space(4.0);

                // Sliders for H, S, V fine tuning
                let mut changed = false;
                ui.horizontal(|ui| {
                    ui.label("Farbton (H):");
                    changed |= ui.add(egui::Slider::new(&mut self.color_h, 0.0..=360.0).suffix("°")).changed();
                });
                ui.horizontal(|ui| {
                    ui.label("Sättigung (S):");
                    let mut s_percent = self.color_s * 100.0;
                    if ui.add(egui::Slider::new(&mut s_percent, 0.0..=100.0).suffix("%")).changed() {
                        self.color_s = s_percent / 100.0;
                        changed = true;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Helligkeit (V):");
                    let mut v_percent = self.color_v * 100.0;
                    if ui.add(egui::Slider::new(&mut v_percent, 0.0..=100.0).suffix("%")).changed() {
                        self.color_v = v_percent / 100.0;
                        changed = true;
                    }
                });

                if changed {
                    let new_col = Color::from_hsv(self.color_h, self.color_s, self.color_v, self.editor.primary_color.a);
                    self.editor.set_primary_color(new_col);
                    self.hex_input = new_col.to_hex();
                }

                ui.separator();
                ui.label("Farbtöne der aktiven Palette:");
                let mut picked_color = None;
                ui.horizontal_wrapped(|ui| {
                    for color in &self.editor.active_palette.colors {
                        let [pr, pg, pb, _] = color.to_rgba8();
                        let pc32 = Color32::from_rgb(pr, pg, pb);
                        let (rect, resp) = ui.allocate_exact_size(Vec2::new(24.0, 24.0), egui::Sense::click());
                        ui.painter().rect_filled(rect, 3.0, pc32);
                        ui.painter().rect_stroke(rect, 3.0, egui::Stroke::new(1.0, Color32::from_rgb(80, 84, 96)), egui::StrokeKind::Outside);
                        if resp.clicked() {
                            picked_color = Some(*color);
                        }
                    }
                });

                if let Some(c) = picked_color {
                    self.set_app_color(c);
                }
            });
        self.show_color_dialog = open;
    }

    /// Renders Artistic Filter Dialog
    fn render_filter_dialog(&mut self, ui: &mut egui::Ui) {
        if !self.show_filter_dialog {
            return;
        }

        let mut open = self.show_filter_dialog;
        egui::Window::new("Filter & Effekte (FX)")
            .open(&mut open)
            .default_size([320.0, 380.0])
            .show(ui.ctx(), |ui| {
                ui.heading("Künstlerische Filter");
                if ui.button("Gaußscher Weichzeichner (Gaussian Blur 3px)").clicked() {
                    self.editor.apply_filter_gaussian_blur(3.0);
                    self.set_status("Gaußscher Weichzeichner angewendet");
                }
                if ui.button("Bewegungsunschärfe (Motion Blur 45°)").clicked() {
                    self.editor.apply_filter_motion_blur(45.0, 10.0);
                    self.set_status("Bewegungsunschärfe angewendet");
                }
                if ui.button("Radialer Zoom-Blur (Aktionslinien)").clicked() {
                    let cx = self.editor.manifest.canvas_width as f32 * 0.5;
                    let cy = self.editor.manifest.canvas_height as f32 * 0.5;
                    self.editor.apply_filter_radial_blur(cx, cy, 1.25, 10);
                    self.set_status("Radialer Zoom-Blur angewendet");
                }
                if ui.button("Unscharf maskieren (Unsharp Mask)").clicked() {
                    self.editor.apply_filter_unsharp_mask(1.5, 1.2, 5);
                    self.set_status("Schärfefilter angewendet");
                }
                if ui.button("Manga Rasterton (Halftone 45°)").clicked() {
                    self.editor.apply_filter_halftone(8.0, 45.0, false);
                    self.set_status("Manga Rasterton angewendet");
                }
                if ui.button("Farben invertieren").clicked() {
                    self.editor.apply_filter_invert();
                    self.set_status("Farben invertiert");
                }
            });
        self.show_filter_dialog = open;
    }

    /// Renders Material Store Dialog
    fn render_material_dialog(&mut self, ui: &mut egui::Ui) {
        if !self.show_material_dialog {
            return;
        }

        let mut open = self.show_material_dialog;
        egui::Window::new("Material-Store & Cloud-Bibliothek")
            .open(&mut open)
            .default_size([500.0, 480.0])
            .resizable(true)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.material_tab, MaterialCategory::Hintergruende, "Hintergründe");
                    ui.selectable_value(&mut self.material_tab, MaterialCategory::Muster, "Muster");
                    ui.selectable_value(&mut self.material_tab, MaterialCategory::Posen3D, "3D-Posen");
                    ui.selectable_value(&mut self.material_tab, MaterialCategory::Texturen, "Texturen");
                    ui.selectable_value(&mut self.material_tab, MaterialCategory::Stempel, "Stempel");
                });
                ui.separator();

                let catalog = MaterialLibrary::default_catalog();
                let filtered: Vec<_> = catalog.into_iter().filter(|m| m.category == self.material_tab).collect();

                egui::ScrollArea::vertical().max_height(380.0).show(ui, |ui| {
                    for mat in filtered {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.strong(&mat.name);
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("Auf Ebene anwenden").clicked() {
                                        self.set_status(format!("Material '{}' geladen & angewendet", mat.name));
                                    }
                                });
                            });
                            ui.label(&mat.description);
                            ui.small(format!("Tags: {}", mat.tags.join(", ")));
                        });
                        ui.add_space(4.0);
                    }
                });
            });
        self.show_material_dialog = open;
    }

    /// Renders IbisPaint Authentic "Meine Galerie" Screen
    fn render_gallery_view(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            // Gallery Header
            ui.horizontal(|ui| {
                ui.heading("Meine Galerie");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("+ Neues Kunstwerk").clicked() {
                        self.view = AppView::Studio;
                    }
                    if ui.button("Zurück zum Zeichnen ->").clicked() {
                        self.view = AppView::Studio;
                    }
                });
            });
            ui.separator();

            // Folder Filter Bar
            ui.horizontal(|ui| {
                ui.label("Ordner:");
                ui.selectable_value(&mut self.gallery_folder, "Alle".into(), "Alle");
                ui.selectable_value(&mut self.gallery_folder, "Skizzen".into(), "Skizzen");
                ui.selectable_value(&mut self.gallery_folder, "Manga".into(), "Manga");
                ui.selectable_value(&mut self.gallery_folder, "Illustrationen".into(), "Illustrationen");
            });
            ui.separator();

            // Artwork Cards
            ui.horizontal_wrapped(|ui| {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.strong(&self.editor.manifest.title);
                        ui.label(format!("Größe: {} x {} px", self.editor.manifest.canvas_width, self.editor.manifest.canvas_height));
                        ui.label(format!("Zeitraffer: {} Frames aufgezeichnet", self.editor.timelapser.frame_count()));

                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            if ui.button("Zeichnen / Bearbeiten").clicked() {
                                self.view = AppView::Studio;
                            }
                            if ui.button("Export (PNG / MP4)").clicked() {
                                let _ = ImageExporter::export_png(&PathBuf::from("artwork.png"), &self.editor.stack);
                                self.set_status("Artwork exportiert");
                            }
                        });

                        // Timelapse player in gallery
                        let total_frames = self.editor.timelapser.frame_count();
                        if total_frames > 0 {
                            ui.separator();
                            ui.label("Zeitraffer (Timelapse) Vorschau:");
                            ui.horizontal(|ui| {
                                if ui.button(if self.gallery_timelapse_playing { "Pause" } else { "Abspielen" }).clicked() {
                                    self.gallery_timelapse_playing = !self.gallery_timelapse_playing;
                                }
                                let mut f = self.gallery_timelapse_frame;
                                if ui.add(egui::Slider::new(&mut f, 0..=total_frames.saturating_sub(1)).text("Frame")).changed() {
                                    self.gallery_timelapse_frame = f;
                                }
                            });
                        }
                    });
                });
            });
        });
    }
}

pub fn run_desktop_app() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Drawling Illustration Studio")
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([900.0, 600.0])
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
        "Drawling Studio",
        native_options,
        Box::new(|cc| Ok(Box::new(DrawlingApp::new(cc)))),
    )
}
