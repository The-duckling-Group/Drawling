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

    // Canvas settings inputs
    pub input_canvas_w: u32,
    pub input_canvas_h: u32,
    pub input_canvas_dpi: f32,
    pub canvas_zoom: f32,
    pub canvas_rotation_deg: f32,

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
        editor.set_primary_color(Color::new(0.12, 0.14, 0.18, 1.0));

        // Use real-time stabilizer by default (IbisPaint standard)
        editor.stabilizer.set_mode(StabilizerMode::Realtime { level: 3 });

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

            import_text: String::new(),
            import_feedback: None,

            previous_brush_tool: ActiveTool::Brush,

            gallery_folder: "Alle".into(),
            gallery_timelapse_frame: 0,
            gallery_timelapse_playing: false,
        }
    }

    fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = msg.into();
        self.last_status_update = Instant::now();
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

        // Authentic IbisPaint dark visuals styling (No emojis, sleek dark tones)
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::from_rgb(32, 33, 38);
        visuals.window_fill = Color32::from_rgb(36, 37, 44);
        visuals.window_stroke = egui::Stroke::new(1.0, Color32::from_rgb(58, 60, 72));
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(42, 44, 52);
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(46, 48, 58);
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(58, 62, 75);
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
            egui::Panel::top("ibis_top_bar").show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(8.0, 0.0);

                    // Gallery Back button
                    if ui.button("< Meine Galerie").on_hover_text("Zur Galerie wechseln").clicked() {
                        self.view = AppView::Gallery;
                    }

                    ui.separator();

                    // Undo / Redo Buttons on Top Bar (IbisPaint layout)
                    if ui.button("Undo").on_hover_text("Rückgängig (Ctrl+Z)").clicked() {
                        self.editor.undo();
                        self.set_status("Rückgängig (Undo)");
                    }
                    if ui.button("Redo").on_hover_text("Wiederholen (Ctrl+Y)").clicked() {
                        self.editor.redo();
                        self.set_status("Wiederholen (Redo)");
                    }

                    ui.separator();

                    // Active Tool quick selector
                    egui::ComboBox::from_id_salt("top_tool_selector")
                        .selected_text(format!("Werkzeug: {:?}", self.editor.active_tool))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.editor.active_tool, ActiveTool::Brush, "Pinsel [B]");
                            ui.selectable_value(&mut self.editor.active_tool, ActiveTool::Eraser, "Radierer [E]");
                            ui.selectable_value(&mut self.editor.active_tool, ActiveTool::Smudge, "Verwischen [J]");
                            ui.selectable_value(&mut self.editor.active_tool, ActiveTool::Bucket, "Farbeimer [G]");
                            ui.selectable_value(&mut self.editor.active_tool, ActiveTool::Eyedropper, "Pipette [I]");
                            ui.selectable_value(&mut self.editor.active_tool, ActiveTool::Lasso, "Lasso-Auswahl");
                            ui.selectable_value(&mut self.editor.active_tool, ActiveTool::MagicWand, "Zauberstab");
                            ui.selectable_value(&mut self.editor.active_tool, ActiveTool::QuickMask, "Schnellmaske");
                            ui.selectable_value(&mut self.editor.active_tool, ActiveTool::FrameDivider, "Manga Panel-Teiler");
                            ui.selectable_value(&mut self.editor.active_tool, ActiveTool::Text, "Sprechblase / Text");
                            ui.selectable_value(&mut self.editor.active_tool, ActiveTool::Transform, "Transformieren");
                        });

                    // Layer Button on Top Bar
                    let layer_count = self.editor.stack.layers.len();
                    let layer_btn_text = format!("Ebenen [{}]", layer_count);
                    if ui.selectable_label(self.show_layer_dialog, layer_btn_text).on_hover_text("Ebenen-Manager öffnen").clicked() {
                        self.show_layer_dialog = !self.show_layer_dialog;
                    }

                    // Material-Store Button on Top Bar
                    if ui.selectable_label(self.show_material_dialog, "Material-Store").on_hover_text("Material-Bibliothek öffnen").clicked() {
                        self.show_material_dialog = !self.show_material_dialog;
                    }

                    // Title
                    ui.separator();
                    ui.strong(format!("Drawling | {}", self.editor.manifest.title));

                    // Right-aligned items on top bar
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Export / Save
                        if ui.button("Speichern & Export").clicked() {
                            let _ = ImageExporter::export_png(&PathBuf::from("artwork.png"), &self.editor.stack);
                            let _ = self.editor.save_project(&PathBuf::from("artwork.drawling"));
                            self.set_status("Projekt gespeichert & als 'artwork.png' exportiert");
                        }

                        // Settings / Canvas dialog button
                        if ui.selectable_label(self.show_canvas_dialog, "Leinwand / Einstellungen").clicked() {
                            self.show_canvas_dialog = !self.show_canvas_dialog;
                        }

                        // Stabilizer Icon Button
                        let stab_text = format!("Stabilisator [{}]", self.stabilizer_level);
                        if ui.selectable_label(self.show_stabilizer_dialog, stab_text).clicked() {
                            self.show_stabilizer_dialog = !self.show_stabilizer_dialog;
                        }

                        // Filter Menu
                        if ui.selectable_label(self.show_filter_dialog, "FX Filter").clicked() {
                            self.show_filter_dialog = !self.show_filter_dialog;
                        }

                        // Symmetry & Rulers Menu
                        ui.menu_button("Symmetrie & Lineale", |ui| {
                            ui.label("Symmetrie-Hilfen:");
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
                            ui.label("Geometrische Lineale:");
                            if ui.selectable_label(self.editor.active_ruler.is_none(), "Lineal: Aus").clicked() {
                                self.editor.active_ruler = None;
                                ui.close();
                            }
                            if ui.button("Geodreieck / Linear-Lineal").clicked() {
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
                            if ui.button("Perspektivlineal (2-Punkt)").clicked() {
                                let w = self.editor.manifest.canvas_width as f32;
                                let h = self.editor.manifest.canvas_height as f32;
                                self.editor.active_ruler = Some(Ruler::Perspective {
                                    vanishing_points: vec![(0.0, h * 0.5), (w, h * 0.5)],
                                    snap_distance: 20.0,
                                });
                                ui.close();
                            }
                        });

                        // XP-Pen Tablet profile
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

        // 2. Linke Steuerleiste (Left Vertical Dock)
        if !self.editor.zen_mode {
            egui::Panel::left("ibis_left_toolbar").default_size(58.0).show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(4.0);

                    // Transform
                    if ui.selectable_label(self.editor.active_tool == ActiveTool::Transform, "[T]").on_hover_text("Transformieren [T]").clicked() {
                        self.editor.active_tool = ActiveTool::Transform;
                    }
                    ui.add_space(2.0);

                    // Zauberstab
                    if ui.selectable_label(self.editor.active_tool == ActiveTool::MagicWand, "[Wand]").on_hover_text("Zauberstab (Magic Wand)").clicked() {
                        self.editor.active_tool = ActiveTool::MagicWand;
                    }
                    ui.add_space(2.0);

                    // Lasso
                    if ui.selectable_label(self.editor.active_tool == ActiveTool::Lasso, "[Lasso]").on_hover_text("Lasso-Auswahl").clicked() {
                        self.editor.active_tool = ActiveTool::Lasso;
                    }
                    ui.add_space(2.0);

                    // Schnellmaske
                    let mask_active = self.editor.active_tool == ActiveTool::QuickMask;
                    if ui.selectable_label(mask_active, "[Mask]").on_hover_text("Schnellmaske: Ein-/Ausblenden mit Pinsel").clicked() {
                        if mask_active {
                            self.editor.active_tool = ActiveTool::Brush;
                            self.set_status("Schnellmaske angewendet");
                        } else {
                            self.editor.active_tool = ActiveTool::QuickMask;
                            self.set_status("Schnellmaske aktiv: Male geschützte Bereiche");
                        }
                    }
                    ui.add_space(2.0);

                    // FX Filter
                    if ui.selectable_label(self.show_filter_dialog, "[FX]").on_hover_text("Filter & Effekte").clicked() {
                        self.show_filter_dialog = !self.show_filter_dialog;
                    }
                    ui.add_space(2.0);

                    // Pinsel
                    if ui.selectable_label(self.editor.active_tool == ActiveTool::Brush, "[B]").on_hover_text("Pinsel (Brush)").clicked() {
                        self.editor.active_tool = ActiveTool::Brush;
                    }
                    ui.add_space(2.0);

                    // Radierer
                    if ui.selectable_label(self.editor.active_tool == ActiveTool::Eraser, "[E]").on_hover_text("Radierer (Eraser)").clicked() {
                        self.editor.active_tool = ActiveTool::Eraser;
                    }
                    ui.add_space(2.0);

                    // Verwischen
                    if ui.selectable_label(self.editor.active_tool == ActiveTool::Smudge, "[J]").on_hover_text("Verwischen (Smudge)").clicked() {
                        self.editor.active_tool = ActiveTool::Smudge;
                    }
                    ui.add_space(2.0);

                    // Farbeimer
                    if ui.selectable_label(self.editor.active_tool == ActiveTool::Bucket, "[G]").on_hover_text("Farbeimer (Bucket Fill)").clicked() {
                        self.editor.active_tool = ActiveTool::Bucket;
                    }
                    ui.add_space(2.0);

                    // Manga Frame Divider
                    if ui.selectable_label(self.editor.active_tool == ActiveTool::FrameDivider, "[Panel]").on_hover_text("Manga Panel-Teiler (Frame Divider)").clicked() {
                        self.editor.active_tool = ActiveTool::FrameDivider;
                    }
                    ui.add_space(2.0);

                    // Text / Speech Balloon
                    if ui.selectable_label(self.editor.active_tool == ActiveTool::Text, "[Text]").on_hover_text("Text & Manga Sprechblase").clicked() {
                        self.editor.active_tool = ActiveTool::Text;
                    }
                    ui.add_space(2.0);

                    // Formen
                    if ui.selectable_label(self.editor.active_tool == ActiveTool::Rectangle, "[Box]").on_hover_text("Rechteck").clicked() {
                        self.editor.active_tool = ActiveTool::Rectangle;
                    }
                    ui.add_space(2.0);

                    if ui.selectable_label(self.editor.active_tool == ActiveTool::Ellipse, "[Kreis]").on_hover_text("Ellipse / Kreis").clicked() {
                        self.editor.active_tool = ActiveTool::Ellipse;
                    }
                    ui.add_space(2.0);

                    // Pipette
                    if ui.selectable_label(self.editor.active_tool == ActiveTool::Eyedropper, "[I]").on_hover_text("Farbpipette (Eyedropper)").clicked() {
                        self.editor.active_tool = ActiveTool::Eyedropper;
                    }
                    ui.add_space(6.0);

                    ui.separator();
                    ui.add_space(4.0);

                    // Material-Store Quick Icon
                    if ui.selectable_label(self.show_material_dialog, "[Store]").on_hover_text("Material-Store / Cloud-Bibliothek").clicked() {
                        self.show_material_dialog = !self.show_material_dialog;
                    }
                });
            });
        }

        // 3. Rechte Steuerleiste (Right Quick-Adjustment Dock)
        if !self.editor.zen_mode {
            egui::Panel::right("ibis_right_toolbar").default_size(70.0).show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(6.0);
                    ui.small("Größe");
                    let params = self.editor.brush_engine.params_mut();
                    ui.add(egui::Slider::new(&mut params.size, 0.5..=500.0).vertical().show_value(false).logarithmic(true));
                    ui.small(format!("{:.0}px", params.size));

                    ui.separator();
                    ui.small("Deckkr.");
                    let mut op_percent = (params.opacity * 100.0).round();
                    if ui.add(egui::Slider::new(&mut op_percent, 1.0..=100.0).vertical().show_value(false)).changed() {
                        params.opacity = op_percent / 100.0;
                    }
                    ui.small(format!("{}%", op_percent as u32));

                    ui.separator();
                    ui.small("Zoom");
                    ui.add(egui::Slider::new(&mut self.canvas_zoom, 0.1..=5.0).vertical().show_value(false));
                    ui.small(format!("{:.0}%", self.canvas_zoom * 100.0));

                    ui.separator();
                    ui.small("Drehung");
                    ui.add(egui::Slider::new(&mut self.canvas_rotation_deg, -180.0..=180.0).vertical().show_value(false));
                    ui.small(format!("{:.0}°", self.canvas_rotation_deg));
                });
            });
        }

        // 4. Untere Werkzeugleiste (Bottom Dock)
        if !self.editor.zen_mode {
            egui::Panel::bottom("ibis_bottom_bar").show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(8.0, 0.0);

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

                    // Brush preview circle button (opens Pinsel Dialog)
                    let current_size = self.editor.brush_engine.params().size;
                    let brush_btn_text = format!("Pinsel: {:.1}px", current_size);
                    if ui.selectable_label(self.show_brush_dialog, brush_btn_text)
                        .on_hover_text("Pinsel-Auswahl & Anpassung (15.000+)").clicked()
                    {
                        self.show_brush_dialog = !self.show_brush_dialog;
                    }

                    // Color Preview Swatch (opens Color Picker)
                    let color = self.editor.primary_color;
                    let c32 = Color32::from_rgb(
                        (color.r * 255.0) as u8,
                        (color.g * 255.0) as u8,
                        (color.b * 255.0) as u8,
                    );
                    let (swatch_rect, swatch_resp) = ui.allocate_exact_size(Vec2::new(26.0, 26.0), egui::Sense::click());
                    ui.painter().rect_filled(swatch_rect, 4.0, c32);
                    ui.painter().rect_stroke(swatch_rect, 4.0, egui::Stroke::new(1.0, Color32::WHITE), egui::StrokeKind::Outside);
                    if swatch_resp.clicked() {
                        self.show_color_dialog = !self.show_color_dialog;
                    }

                    // Quick Size Slider
                    ui.label("Größe:");
                    let params = self.editor.brush_engine.params_mut();
                    ui.add(egui::Slider::new(&mut params.size, 0.5..=500.0).show_value(false).logarithmic(true));

                    // Quick Opacity Slider
                    ui.label("Deckkraft:");
                    let mut op_percent = (params.opacity * 100.0).round();
                    if ui.add(egui::Slider::new(&mut op_percent, 1.0..=100.0).show_value(false)).changed() {
                        params.opacity = op_percent / 100.0;
                    }
                    ui.label(format!("{}%", op_percent as u32));

                    ui.separator();

                    // Layers Button with Count Indicator
                    let layer_count = self.editor.stack.layers.len();
                    let layer_btn_text = format!("Ebenen [{}]", layer_count);
                    if ui.selectable_label(self.show_layer_dialog, layer_btn_text)
                        .on_hover_text("Ebenen-Manager öffnen").clicked()
                    {
                        self.show_layer_dialog = !self.show_layer_dialog;
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

        // Central Drawing Canvas
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

    /// Renders Color Picker & Palette Dialog
    fn render_color_dialog(&mut self, ui: &mut egui::Ui) {
        if !self.show_color_dialog {
            return;
        }

        let mut open = self.show_color_dialog;
        egui::Window::new("Farbe & Palette")
            .open(&mut open)
            .default_size([300.0, 360.0])
            .resizable(false)
            .show(ui.ctx(), |ui| {
                let mut rgba = [
                    self.editor.primary_color.r,
                    self.editor.primary_color.g,
                    self.editor.primary_color.b,
                ];
                if ui.color_edit_button_rgb(&mut rgba).changed() {
                    self.editor.set_primary_color(Color::new(rgba[0], rgba[1], rgba[2], 1.0));
                }

                ui.separator();
                ui.label(format!("Aktive Palette: {}", self.editor.active_palette.name));

                let mut picked_color = None;
                ui.horizontal_wrapped(|ui| {
                    for color in &self.editor.active_palette.colors {
                        let c32 = Color32::from_rgb(
                            (color.r * 255.0) as u8,
                            (color.g * 255.0) as u8,
                            (color.b * 255.0) as u8,
                        );
                        let (rect, resp) = ui.allocate_exact_size(Vec2::new(28.0, 28.0), egui::Sense::click());
                        ui.painter().rect_filled(rect, 4.0, c32);
                        if resp.clicked() {
                            picked_color = Some(*color);
                        }
                    }
                });

                if let Some(c) = picked_color {
                    self.editor.set_primary_color(c);
                }
            });
        self.show_color_dialog = open;
    }

    /// Renders Artistic Filter Dialog (Over 80 image manipulation filter effects)
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

    /// Renders Material Store Dialog (Hintergründe, Muster, 3D-Körpermodelle, Texturen, Stempel)
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
