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
