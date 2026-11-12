use crate::state::{ActiveTool, EditorState};
use drawling_core::*;
use drawling_storage::*;
use eframe::egui::{self, Color32, ColorImage, TextureOptions, Vec2};
use std::path::PathBuf;
use std::time::Instant;

pub struct DrawlingApp {
    pub editor: EditorState,
    pub canvas_texture: Option<egui::TextureHandle>,
    pub status_message: String,
    pub last_status_update: Instant,
    pub selected_stabilizer: usize,
    pub last_paint_time: Instant,
}

impl DrawlingApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let width = 1280;
        let height = 720;
        let mut editor = EditorState::new("Untitled Artwork", width, height, width as f32, height as f32);
        
        // Add default lineart layer
        let inking_layer = editor.stack.add_layer("Lineart");
        editor.stack.active_index = inking_layer;
        editor.set_primary_color(Color::new(0.12, 0.14, 0.18, 1.0));

        Self {
            editor,
            canvas_texture: None,
            status_message: "Ready to draw. Use Mouse or XP-Pen Stylus with pressure.".into(),
            last_status_update: Instant::now(),
            selected_stabilizer: 1, // Lazy Mouse by default
            last_paint_time: Instant::now(),
        }
    }

    fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = msg.into();
        self.last_status_update = Instant::now();
    }
}

impl eframe::App for DrawlingApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Keyboard Shortcuts
        if ctx.input(|i| i.key_pressed(egui::Key::B)) {
            self.editor.active_tool = ActiveTool::Brush;
            self.set_status("Tool: Brush (B)");
        }
        if ctx.input(|i| i.key_pressed(egui::Key::E)) {
            self.editor.active_tool = ActiveTool::Eraser;
            self.set_status("Tool: Eraser (E)");
        }
        if ctx.input(|i| i.key_pressed(egui::Key::J)) {
            self.editor.active_tool = ActiveTool::Smudge;
            self.set_status("Tool: Smudge (J)");
        }
        if ctx.input(|i| i.key_pressed(egui::Key::G)) {
            self.editor.active_tool = ActiveTool::Bucket;
            self.set_status("Tool: Flood Fill Bucket (G)");
        }
        if ctx.input(|i| i.key_pressed(egui::Key::I)) {
            self.editor.active_tool = ActiveTool::Eyedropper;
            self.set_status("Tool: Eyedropper (I)");
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Tab) || i.key_pressed(egui::Key::F11)) {
            self.editor.zen_mode = !self.editor.zen_mode;
            self.set_status(if self.editor.zen_mode { "Zen Mode Active (Press Tab to exit)" } else { "Zen Mode Inactive" });
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Z)) {
            self.editor.undo();
            self.set_status("Undo performed");
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Y)) {
            self.editor.redo();
            self.set_status("Redo performed");
        }

        // Top Menu Bar
        if !self.editor.zen_mode {
            egui::Panel::top("top_menu").show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Save .drawling Project").clicked() {
                            let path = PathBuf::from("artwork.drawling");
                            if let Err(e) = self.editor.save_project(&path) {
                                self.set_status(format!("Save error: {}", e));
                            } else {
                                self.set_status("Project saved as 'artwork.drawling'");
                            }
                            ui.close();
                        }
                        if ui.button("Export Lossless PNG").clicked() {
                            let path = PathBuf::from("export_artwork.png");
                            if let Err(e) = ImageExporter::export_png(&path, &self.editor.stack) {
                                self.set_status(format!("Export error: {}", e));
                            } else {
                                self.set_status("Exported as 'export_artwork.png'");
                            }
                            ui.close();
                        }
                        if ui.button("Export JPEG").clicked() {
                            let path = PathBuf::from("export_artwork.jpg");
                            if let Err(e) = ImageExporter::export_jpeg(&path, &self.editor.stack, 92) {
                                self.set_status(format!("Export error: {}", e));
                            } else {
                                self.set_status("Exported as 'export_artwork.jpg'");
                            }
                            ui.close();
                        }
                    });

                    ui.menu_button("Edit", |ui| {
                        if ui.button("Undo (Ctrl+Z)").clicked() {
                            self.editor.undo();
                            ui.close();
                        }
                        if ui.button("Redo (Ctrl+Y)").clicked() {
                            self.editor.redo();
                            ui.close();
                        }
                        if ui.button("Clear Active Layer").clicked() {
                            if let Some(layer) = self.editor.stack.active_layer_mut() {
                                layer.clear();
                                self.editor.compositor.update_composite(&self.editor.stack);
                                self.set_status("Active layer cleared");
                            }
                            ui.close();
                        }
                    });

                    ui.menu_button("Filters (Rayon)", |ui| {
                        if ui.button("Gaussian Blur (3.0px)").clicked() {
                            self.editor.apply_filter_gaussian_blur(3.0);
                            self.set_status("Gaussian Blur applied");
                            ui.close();
                        }
                        if ui.button("Motion Blur (45°, 10px)").clicked() {
                            self.editor.apply_filter_motion_blur(45.0, 10.0);
                            self.set_status("Motion Blur applied");
                            ui.close();
                        }
                        if ui.button("Radial / Zoom Blur").clicked() {
                            let cx = self.editor.manifest.canvas_width as f32 * 0.5;
                            let cy = self.editor.manifest.canvas_height as f32 * 0.5;
                            self.editor.apply_filter_radial_blur(cx, cy, 1.25, 10);
                            self.set_status("Radial Blur applied");
                            ui.close();
                        }
                        if ui.button("Unsharp Mask (Sharpen)").clicked() {
                            self.editor.apply_filter_unsharp_mask(1.5, 1.2, 5);
                            self.set_status("Unsharp Mask applied");
                            ui.close();
                        }
                        if ui.button("Manga Halftone Screentone").clicked() {
                            self.editor.apply_filter_halftone(8.0, 45.0, false);
                            self.set_status("Manga Screentone applied");
                            ui.close();
                        }
                        if ui.button("Invert Colors (Ctrl+I)").clicked() {
                            self.editor.apply_filter_invert();
                            self.set_status("Colors inverted");
                            ui.close();
                        }
                        if ui.button("Bloom Glow").clicked() {
                            self.editor.apply_filter_bloom(0.65, 4.0, 0.6);
                            self.set_status("Bloom Glow applied");
                            ui.close();
                        }
                    });

                    ui.menu_button("Tablet Profile (XP-Pen)", |ui| {
                        if ui.button("XP-Pen Standard (Linear)").clicked() {
                            self.editor.tablet_profile = TabletCalibrationProfile::xp_pen_standard();
                            self.set_status("Tablet profile: XP-Pen Standard (8192 levels)");
                            ui.close();
                        }
                        if ui.button("XP-Pen Soft (Light Touch)").clicked() {
                            self.editor.tablet_profile = TabletCalibrationProfile::xp_pen_soft();
                            self.set_status("Tablet profile: XP-Pen Soft (Gamma 0.65)");
                            ui.close();
                        }
                        if ui.button("XP-Pen Hard (Firm Press)").clicked() {
                            self.editor.tablet_profile = TabletCalibrationProfile::xp_pen_hard();
                            self.set_status("Tablet profile: XP-Pen Hard (Gamma 1.6)");
                            ui.close();
                        }
                        if ui.button("Wacom Pro Pen 2").clicked() {
                            self.editor.tablet_profile = TabletCalibrationProfile::wacom_pro_pen_2();
                            self.set_status("Tablet profile: Wacom Pro Pen 2");
                            ui.close();
                        }
                        if ui.button("Huion Standard").clicked() {
                            self.editor.tablet_profile = TabletCalibrationProfile::huion_standard();
                            self.set_status("Tablet profile: Huion Standard");
                            ui.close();
                        }
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Zen Mode (Tab)").clicked() {
                            self.editor.zen_mode = true;
                        }
                    });
                });
            });
        }

        // Bottom Status Bar
        egui::Panel::bottom("bottom_status").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("🦆 {}", self.status_message));
                ui.separator();
                let tool_name = format!("{:?}", self.editor.active_tool);
                ui.label(format!("Tool: {}", tool_name));
                ui.separator();
                ui.label(format!("Canvas: {}x{}", self.editor.manifest.canvas_width, self.editor.manifest.canvas_height));
                ui.separator();
                ui.label(format!("Tablet: {} ({} levels)", self.editor.tablet_profile.manufacturer, self.editor.tablet_profile.max_pressure_levels));
            });
        });

        // Left Tools Panel
        if !self.editor.zen_mode {
            egui::Panel::left("left_tools").default_size(220.0).show(ui, |ui| {
                ui.heading("🛠 Tools");
                ui.separator();

                ui.horizontal_wrapped(|ui| {
                    if ui.selectable_label(self.editor.active_tool == ActiveTool::Brush, "🖌 Brush (B)").clicked() {
                        self.editor.active_tool = ActiveTool::Brush;
                    }
                    if ui.selectable_label(self.editor.active_tool == ActiveTool::Eraser, "🧹 Eraser (E)").clicked() {
                        self.editor.active_tool = ActiveTool::Eraser;
                    }
                    if ui.selectable_label(self.editor.active_tool == ActiveTool::Smudge, "💧 Smudge (J)").clicked() {
                        self.editor.active_tool = ActiveTool::Smudge;
                    }
                    if ui.selectable_label(self.editor.active_tool == ActiveTool::Bucket, "🪣 Fill (G)").clicked() {
                        self.editor.active_tool = ActiveTool::Bucket;
                    }
                    if ui.selectable_label(self.editor.active_tool == ActiveTool::Eyedropper, "💉 Pick (I)").clicked() {
                        self.editor.active_tool = ActiveTool::Eyedropper;
                    }
                    if ui.selectable_label(self.editor.active_tool == ActiveTool::Rectangle, "🔲 Rect").clicked() {
                        self.editor.active_tool = ActiveTool::Rectangle;
                    }
                    if ui.selectable_label(self.editor.active_tool == ActiveTool::Ellipse, "⭕ Circle").clicked() {
                        self.editor.active_tool = ActiveTool::Ellipse;
                    }
                    if ui.selectable_label(self.editor.active_tool == ActiveTool::Star, "⭐ Star").clicked() {
                        self.editor.active_tool = ActiveTool::Star;
                    }
                });

                ui.add_space(8.0);
                ui.heading("Brush Dynamics");
                ui.separator();

                let params = self.editor.brush_engine.params_mut();
                ui.add(egui::Slider::new(&mut params.size, 1.0..=120.0).text("Size (px)"));
                ui.add(egui::Slider::new(&mut params.opacity, 0.05..=1.0).text("Opacity"));
                ui.add(egui::Slider::new(&mut params.hardness, 0.01..=1.0).text("Hardness"));
                ui.checkbox(&mut params.size_by_pressure, "Pressure -> Size");
                ui.checkbox(&mut params.opacity_by_pressure, "Pressure -> Opacity");
                ui.checkbox(&mut params.tilt_affects_size, "Tilt -> Dynamic Size");
                ui.checkbox(&mut params.tilt_aspect_ratio, "Tilt -> Elliptical Dab");

                ui.add_space(8.0);
                ui.heading("Presets");
                ui.separator();
                let mut selected_preset = None;
                for (idx, preset) in self.editor.brush_presets.iter().enumerate() {
                    let is_active = self.editor.active_preset_index == idx;
                    if ui.selectable_label(is_active, &preset.name).clicked() {
                        selected_preset = Some((idx, preset.name.clone()));
                    }
                }
                if let Some((idx, name)) = selected_preset {
                    self.editor.select_preset(idx);
                    self.set_status(format!("Preset selected: {}", name));
                }

                ui.add_space(8.0);
                ui.heading("Stabilization");
                ui.separator();
                ui.radio_value(&mut self.selected_stabilizer, 0, "Off");
                ui.radio_value(&mut self.selected_stabilizer, 1, "Lazy Mouse (Rope)");
                ui.radio_value(&mut self.selected_stabilizer, 2, "Moving Average");

                match self.selected_stabilizer {
                    0 => self.editor.stabilizer.set_mode(StabilizerMode::Off),
                    1 => self.editor.stabilizer.set_mode(StabilizerMode::LazyMouse { rope_length: 12.0, tension: 0.8 }),
                    2 => self.editor.stabilizer.set_mode(StabilizerMode::MovingAverage { window_size: 6 }),
                    _ => {}
                }
            });
        }

        // Right Layers & Color Palette Panel
        if !self.editor.zen_mode {
            egui::Panel::right("right_layers").default_size(260.0).show(ui, |ui| {
                ui.heading("🎨 Color & Palette");
                ui.separator();

                let mut rgba = [
                    self.editor.primary_color.r,
                    self.editor.primary_color.g,
                    self.editor.primary_color.b,
                ];
                if ui.color_edit_button_rgb(&mut rgba).changed() {
                    self.editor.set_primary_color(Color::new(rgba[0], rgba[1], rgba[2], 1.0));
                }

                ui.add_space(4.0);
                ui.label(format!("Active Palette: {}", self.editor.active_palette.name));
                let mut picked_color = None;
                ui.horizontal_wrapped(|ui| {
                    for color in &self.editor.active_palette.colors {
                        let c32 = Color32::from_rgb(
                            (color.r * 255.0) as u8,
                            (color.g * 255.0) as u8,
                            (color.b * 255.0) as u8,
                        );
                        let (rect, resp) = ui.allocate_exact_size(Vec2::new(24.0, 24.0), egui::Sense::click());
                        ui.painter().rect_filled(rect, 4.0, c32);
                        if resp.clicked() {
                            picked_color = Some(*color);
                        }
                    }
                });
                if let Some(c) = picked_color {
                    self.editor.set_primary_color(c);
                }

                ui.add_space(12.0);
                ui.heading("📑 Layers");
                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("+ New Layer").clicked() {
                        let count = self.editor.stack.layers.len();
                        let idx = self.editor.stack.add_layer(format!("Layer {}", count));
                        self.editor.stack.active_index = idx;
                        self.editor.compositor.update_composite(&self.editor.stack);
                        self.set_status("New layer added");
                    }
                    if ui.button("⬇ Merge").clicked() {
                        self.editor.merge_down();
                        self.set_status("Merged layer down");
                    }
                });

                ui.add_space(4.0);
                let active_idx = self.editor.stack.active_index;
                let layer_count = self.editor.stack.layers.len();
                let mut new_active_layer = None;

                for i in (0..layer_count).rev() {
                    let is_active = i == active_idx;
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            let layer = &mut self.editor.stack.layers[i];
                            ui.checkbox(&mut layer.visible, "");
                            let name = layer.name.clone();
                            if ui.selectable_label(is_active, name).clicked() {
                                new_active_layer = Some(i);
                            }
                        });
                    });
                }
                if let Some(idx) = new_active_layer {
                    self.editor.stack.active_index = idx;
                }
            });
        }

        // Central Canvas Area
        egui::CentralPanel::default().show(ui, |ui| {
            let available_size = ui.available_size();
            let canvas_w = self.editor.manifest.canvas_width;
            let canvas_h = self.editor.manifest.canvas_height;

            // Fit canvas inside available space
            let scale_x = available_size.x / canvas_w as f32;
            let scale_y = available_size.y / canvas_h as f32;
            let fit_scale = scale_x.min(scale_y).clamp(0.1, 5.0) * 0.95;

            let display_w = canvas_w as f32 * fit_scale;
            let display_h = canvas_h as f32 * fit_scale;

            let (rect, response) = ui.allocate_exact_size(Vec2::new(display_w, display_h), egui::Sense::drag());

            // Handle Stylus / Mouse Painting Interactions
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                if rect.contains(pointer_pos) {
                    let rel_x = (pointer_pos.x - rect.min.x) / rect.width() * canvas_w as f32;
                    let rel_y = (pointer_pos.y - rect.min.y) / rect.height() * canvas_h as f32;
                    
                    let raw_pressure = 0.85;
                    let now_ms = (Instant::now().duration_since(self.last_paint_time).as_millis() as u64) & 0xFFFFFFFF;

                    if response.drag_started() {
                        self.editor.pointer_down_with_details(rel_x, rel_y, raw_pressure, 0.0, 0.0);
                    } else if response.dragged() {
                        self.editor.pointer_move_with_details(rel_x, rel_y, raw_pressure, 0.0, 0.0, now_ms);
                    }
                }
            }

            if response.drag_stopped() {
                self.editor.pointer_up();
            }

            // Update & Render Composited Canvas Texture
            let flat_pixels = self.editor.compositor.composite_buffer();
            let color_image = ColorImage::from_rgba_unmultiplied(
                [canvas_w as usize, canvas_h as usize],
                flat_pixels,
            );

            let texture = self.canvas_texture.get_or_insert_with(|| {
                ui.ctx().load_texture("main_canvas", color_image.clone(), TextureOptions::LINEAR)
            });
            texture.set(color_image, TextureOptions::LINEAR);

            // Draw Canvas Frame
            ui.painter().rect_filled(rect, 0.0, Color32::WHITE);
            let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            ui.painter().image(texture.id(), rect, uv, Color32::WHITE);
            ui.painter().rect_stroke(rect, 0.0, egui::Stroke::new(1.0, Color32::from_rgb(60, 65, 75)), egui::StrokeKind::Outside);
        });

        // Request continuous rendering while drawing for 60+ FPS responsiveness
        if self.editor.is_drawing {
            ui.ctx().request_repaint();
        }
    }
}

pub fn run_desktop_app() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("🦆 Drawling for Duckling - Digital Illustration Studio")
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([900.0, 600.0])
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
        "Drawling for Duckling",
        native_options,
        Box::new(|cc| Ok(Box::new(DrawlingApp::new(cc)))),
    )
}
