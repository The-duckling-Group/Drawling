use drawling_core::*;
use drawling_storage::*;
use uuid::Uuid;

#[test]
fn test_end_to_end_drawing_and_export_workflow() {
    let width = 64;
    let height = 64;

    // 1. Create layer stack
    let mut stack = LayerStack::new(width, height);
    let inking_layer = stack.add_layer("Lineart");
    stack.active_index = inking_layer;

    // 2. Setup brush engine and stabilizer
    let mut params = BrushParameters::default();
    params.size = 8.0;
    params.color = Color::new(0.1, 0.2, 0.8, 1.0);
    params.size_by_pressure = true;

    let mut engine = BrushEngine::new(params);
    let mut stabilizer = StrokeStabilizer::new(StabilizerMode::LazyMouse {
        rope_length: 5.0,
        tension: 0.8,
    });

    // 3. Draw a stabilized curve
    let p0 = StrokePoint::new(10.0, 10.0, 0.2);
    if let Some(pt) = stabilizer.push(p0) {
        for dab in engine.push_point(pt) {
            BrushTipRasterizer::render_dab(&mut stack.layers[inking_layer], &dab);
        }
    }

    let p1 = StrokePoint::new(30.0, 40.0, 0.9);
    if let Some(pt) = stabilizer.push(p1) {
        for dab in engine.push_point(pt) {
            BrushTipRasterizer::render_dab(&mut stack.layers[inking_layer], &dab);
        }
    }

    if let Some(pt) = stabilizer.finish() {
        for dab in engine.push_point(pt) {
            BrushTipRasterizer::render_dab(&mut stack.layers[inking_layer], &dab);
        }
    }

    // Verify drawing produced non-transparent pixels
    let painted_pixels = stack.layers[inking_layer]
        .pixels
        .chunks_exact(4)
        .filter(|px| px[3] > 0)
        .count();
    assert!(painted_pixels > 0, "Stroke should paint pixels on the layer");

    // 4. Test Multi-threaded Filter: Brightness & Contrast
    ColorAdjust::brightness_contrast(&mut stack.layers[inking_layer], 0.1, 0.2);

    // 5. Test Composite
    let mut composite = vec![0u8; (width * height * 4) as usize];
    stack.compose_to_buffer(&mut composite);
    assert!(composite.iter().any(|&v| v > 0));

    // 6. Test .drawling Archive Save and Load
    let manifest = DrawlingManifest {
        id: Uuid::new_v4(),
        title: "Integration Test Masterpiece".into(),
        author: "Duckling QA".into(),
        version: 1,
        canvas_width: width,
        canvas_height: height,
        dpi: 300.0,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        edit_duration_secs: 60,
        tags: vec!["IntegrationTest".into()],
        layers: stack
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
            .collect(),
        active_layer_index: stack.active_index,
    };

    let temp_archive = std::env::temp_dir().join(format!("integration_test_{}.drawling", Uuid::new_v4()));
    DrawlingArchive::save(&temp_archive, &manifest, &stack, None).expect("Save archive failed");

    let (loaded_man, loaded_stack, thumb) = DrawlingArchive::load(&temp_archive).expect("Load archive failed");
    assert_eq!(loaded_man.title, "Integration Test Masterpiece");
    assert_eq!(loaded_stack.layers.len(), stack.layers.len());
    assert!(!thumb.is_empty());

    // 7. Test Export
    let temp_png = std::env::temp_dir().join(format!("test_export_{}.png", Uuid::new_v4()));
    ImageExporter::export_png(&temp_png, &loaded_stack).expect("PNG export failed");
    assert!(std::fs::metadata(&temp_png).unwrap().len() > 0);

    // 8. Test Gallery Indexing
    let mut db = GalleryDatabase::open_in_memory().expect("Open gallery db failed");
    let record = ProjectRecord {
        id: loaded_man.id,
        title: loaded_man.title.clone(),
        file_path: temp_archive.to_string_lossy().into(),
        thumbnail_png: thumb,
        width: loaded_man.canvas_width,
        height: loaded_man.canvas_height,
        created_at: loaded_man.created_at,
        updated_at: loaded_man.updated_at,
        edit_duration_secs: 60,
        tags: loaded_man.tags.clone(),
    };
    db.upsert_project(&record).expect("Upsert failed");
    let searched = db.search_by_tag("integrationtest").expect("Search failed");
    assert_eq!(searched.len(), 1);

    // Clean up
    let _ = std::fs::remove_file(temp_archive);
    let _ = std::fs::remove_file(temp_png);
}

#[test]
fn test_advanced_tools_and_artistic_filters_workflow() {
    let width = 64;
    let height = 64;
    let mut stack = LayerStack::new(width, height);
    let layer_idx = stack.add_layer("Artwork");
    stack.active_index = layer_idx;
    let layer = &mut stack.layers[layer_idx];

    // 1. Draw shapes
    ShapeDrawer::draw_rect(layer, 5, 5, 20, 20, Color::RED, None);
    ShapeDrawer::draw_star(layer, 45.0, 45.0, 5, 6.0, 14.0, Color::YELLOW, 2.0);

    // 2. Flood fill with expansion inside a region
    FloodFill::fill(layer, 10, 10, Color::BLUE, 0.1, 1, true);
    assert_eq!(layer.get_color(10, 10), Some(Color::BLUE));

    // 3. Color Picker sampling
    let sampled = ColorPicker::sample_layer(layer, 10, 10, 1);
    assert!(sampled.b > 0.5);

    // 4. Smudge tool stroke
    let mut smudge = SmudgeEngine::new(6.0, 0.8, 0.5);
    smudge.begin_stroke(layer, 10.0, 10.0);
    smudge.smudge_to(layer, 10.0, 10.0, 25.0, 10.0);

    // 5. Advanced artistic filters pipeline
    ArtisticFilter::bloom(layer, 0.5, 2.0, 0.7);
    ArtisticFilter::chromatic_aberration(layer, 2.0, 0.0);
    ArtisticFilter::motion_blur(layer, 90.0, 3.0);
    ArtisticFilter::levels(layer, 5, 250, 1.0, 0, 255);
    ArtisticFilter::unsharp_mask(layer, 1.0, 1.5, 5);

    // Verify non-empty pixels after filters
    let active_count = layer.pixels.chunks_exact(4).filter(|px| px[3] > 0).count();
    assert!(active_count > 50, "Layer should contain painted pixels after artistic pipeline");
}

#[test]
fn test_xp_pen_stylus_pressure_tilt_and_eraser_workflow() {
    let mut editor = drawling_app::EditorState::new("XP-Pen Tablet Test", 128, 128, 128.0, 128.0);
    editor.viewport.zoom = 1.0;
    editor.viewport.pan_x = 0.0;
    editor.viewport.pan_y = 0.0;
    let art_layer = editor.stack.add_layer("Artwork Layer");
    editor.stack.active_index = art_layer;

    // 1. Verify default tablet profile is XP-Pen with 8192 levels
    assert_eq!(editor.tablet_profile.manufacturer, "XP-Pen");
    assert_eq!(editor.tablet_profile.max_pressure_levels, 8192);
    assert!(editor.tablet_profile.supports_tilt);

    // 2. Simulate Stylus Hover (Proximity)
    editor.handle_tablet_event(TabletInputEvent::Hover {
        x: 64.0,
        y: 64.0,
        tilt_x: 30.0,
        tilt_y: 10.0,
        tool: TabletTool::Pen,
    });
    assert!(editor.is_stylus_hovering);
    assert_eq!(editor.active_stylus_tool, TabletTool::Pen);

    // 3. Stylus Pen Down with 8192-level pressure and tilt
    editor.set_primary_color(Color::RED);
    editor.handle_tablet_event(TabletInputEvent::Down {
        x: 20.0,
        y: 64.0,
        pressure: 0.75, // 75% force on XP-Pen stylus
        tilt_x: 45.0,
        tilt_y: 0.0,
        tool: TabletTool::Pen,
        buttons: StylusButtons {
            tip: true,
            button1: false,
            button2: false,
            button3: false,
        },
    });

    // 4. Stylus Motion along line
    for step in 1..=10 {
        let x = 20.0 + (step as f32 * 5.0);
        editor.handle_tablet_event(TabletInputEvent::Motion {
            x,
            y: 64.0,
            pressure: 0.75,
            tilt_x: 45.0,
            tilt_y: 0.0,
            tool: TabletTool::Pen,
            buttons: StylusButtons {
                tip: true,
                button1: false,
                button2: false,
                button3: false,
            },
            timestamp_ms: step * 10,
        });
    }

    // 5. Stylus Up
    editor.handle_tablet_event(TabletInputEvent::Up {
        tool: TabletTool::Pen,
    });

    // Verify drawn red pixels
    let painted_count = editor.stack.layers[art_layer]
        .pixels
        .chunks_exact(4)
        .filter(|px| px[0] > 200)
        .count();
    assert!(painted_count > 10, "XP-Pen stylus stroke should paint pixels");

    // 6. Test Barrel Button 1 (Eyedropper quick sample)
    editor.handle_tablet_event(TabletInputEvent::Down {
        x: 30.0,
        y: 64.0,
        pressure: 0.5,
        tilt_x: 0.0,
        tilt_y: 0.0,
        tool: TabletTool::Pen,
        buttons: StylusButtons {
            tip: false,
            button1: true, // Eyedropper sampled under cursor
            button2: false,
            button3: false,
        },
    });
    assert_eq!(editor.primary_color.r, 1.0, "Barrel button 1 should sample red color under stylus");

    // 7. Test Inverted Stylus Physical Eraser (flipping the pen)
    editor.handle_tablet_event(TabletInputEvent::Down {
        x: 30.0,
        y: 64.0,
        pressure: 0.9,
        tilt_x: 0.0,
        tilt_y: 0.0,
        tool: TabletTool::Eraser, // Pen inverted
        buttons: StylusButtons {
            tip: true,
            button1: false,
            button2: false,
            button3: false,
        },
    });

    editor.handle_tablet_event(TabletInputEvent::Up {
        tool: TabletTool::Eraser,
    });

    // Erased pixel should now be erased
    let erased = editor.stack.layers[art_layer].get_color(30, 64).unwrap();
    assert_eq!(erased.a, 0.0, "Physical eraser tip should erase underlying paint");
}
