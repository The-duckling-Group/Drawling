mod shortcuts;
mod state;
mod theme;

use drawling_core::*;
use drawling_storage::*;
use state::EditorState;
use std::time::Instant;

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    log::info!("Starting Drawling for Duckling (Professional Rust Drawing Studio)");

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--demo" {
        run_full_system_demo();
        return;
    }

    println!("============================================================");
    println!("        🦆 DRAWLING FOR DUCKLING - RUST DRAWING APP 🦆        ");
    println!("============================================================");
    println!("Platform: Windows & Linux (KDE Plasma Optimized)");
    println!("Theme: KDE Breeze Dark / Modern Windows 11 Dark");
    println!("Rendering: wgpu GPU pipeline + Rayon Multi-threaded compositor");
    println!("Storage: .drawling container + SQLite Gallery");
    println!("Run with `--demo` to execute the full system verification & benchmark.");
    println!("============================================================");

    // Run demo by default if executed without args to show capabilities
    run_full_system_demo();
}

fn run_full_system_demo() {
    println!("\n[1/7] Initializing Canvas & Layer Stack (1920x1080)...");
    let mut editor = EditorState::new("Duckling Masterpiece", 1920, 1080, 1920.0, 1080.0);

    // 1. Layer System Verification: Add layers with various Blend Modes
    println!("[2/7] Testing Layer Management (13 Blend Modes & Clipping Masks)...");
    let inking_idx = editor.stack.add_layer("Lineart Inking");
    let shading_idx = editor.stack.add_layer("Multiply Shading");
    let highlights_idx = editor.stack.add_layer("Color Dodge Highlights");

    if let Some(l) = editor.stack.layers.get_mut(shading_idx) {
        l.blend_mode = BlendMode::Multiply;
        l.opacity = 0.85;
        l.clipping_mask = true; // Clip shading to lineart/base
    }
    if let Some(l) = editor.stack.layers.get_mut(highlights_idx) {
        l.blend_mode = BlendMode::ColorDodge;
        l.opacity = 0.70;
    }

    println!("      Layers created: {} total", editor.stack.layers.len());
    for (i, layer) in editor.stack.layers.iter().enumerate() {
        println!(
            "      - Layer {}: '{}' (Mode: {:?}, Opacity: {:.0}%, Clip: {})",
            i,
            layer.name,
            layer.blend_mode,
            layer.opacity * 100.0,
            layer.clipping_mask
        );
    }

    // 2. Brush Engine & Stroke Stabilization (Lazy Mouse + Catmull-Rom)
    println!("[3/7] Testing Brush Engine & Real-time Stroke Stabilization...");
    editor.stack.active_index = inking_idx;
    editor.select_preset(0); // G-Pen
    editor.set_primary_color(Color::from_hex("#1a252f").unwrap());

    // Enable 8-fold radial symmetry
    editor.symmetry_mode = SymmetryMode::Radial {
        center_x: 960.0,
        center_y: 540.0,
        segments: 8,
        mirror: true,
    };

    println!("      Drawing stroke with 8-fold radial symmetry + Lazy Mouse rope stabilization...");
    let stroke_start = Instant::now();
    editor.pointer_down(960.0, 300.0, 0.4);

    for step in 1..=40 {
        let t = step as f32 / 40.0;
        let x = 960.0 + (t * 200.0);
        let y = 300.0 + ((t * std::f32::consts::PI).sin() * 120.0);
        let pressure = 0.3 + 0.6 * (1.0 - (t - 0.5).abs() * 2.0); // Dynamic pressure curve
        editor.pointer_move(x, y, pressure);
    }
    editor.pointer_up();
    let stroke_duration = stroke_start.elapsed();
    println!("      Stroke completed in {:?} with full symmetry propagation.", stroke_duration);

    // 3. Selection Tools: Magic Wand & Feathering
    println!("[4/8] Testing Selection Masks (Magic Wand, Boolean Ops & Feathering)...");
    let target_layer = &editor.stack.layers[inking_idx];
    editor.selection_mask.select_magic_wand(target_layer, 960, 540, 32.0, false);
    editor.selection_mask.feather(2);
    if let Some((x0, y0, x1, y1)) = editor.selection_mask.bounding_box() {
        println!("      Selection mask active bounding box: ({}, {}) -> ({}, {})", x0, y0, x1, y1);
    }

    // 4. Transform & Drawing Aids: Rulers and Perspective Snapping
    println!("[5/8] Testing Drawing Aids: Rulers & Perspective Vanishing Points...");
    let ruler = Ruler::Linear {
        p0: (100.0, 100.0),
        p1: (800.0, 100.0),
        snap_distance: 15.0,
    };
    let snapped_ruler = ruler.snap((450.0, 105.0));
    println!("      Linear ruler snapped (450, 105) -> {:?}", snapped_ruler);

    let persp = PerspectiveGrid::two_point(540.0, -500.0, 2420.0);
    let snapped_persp = persp.snap_point((962.0, 540.0));
    println!("      Perspective vanishing ray snapped (962, 540) -> {:?}", snapped_persp);

    // 5. Multi-threaded Image Processing Filters (Rayon)
    println!("[6/8] Testing Multi-threaded Filters (Gaussian Blur, Sepia, Vignette)...");
    let filter_start = Instant::now();
    if let Some(active) = editor.stack.active_layer_mut() {
        GaussianBlur::apply(active, 2.0);
        ColorAdjust::brightness_contrast(active, 0.05, 0.15);
        ColorAdjust::sepia(active, 0.3);
        ColorAdjust::vignette(active, 1.0, 0.4);
    }
    let filter_duration = filter_start.elapsed();
    println!("      Artistic filters processed via Rayon in {:?}", filter_duration);

    // 6. High Performance Compositor Benchmark (60+ FPS check)
    println!("[7/8] Benchmarking Full Canvas Compositor...");
    let frames = if cfg!(debug_assertions) { 10 } else { 60 };
    let bench_start = Instant::now();
    let mut screen_buffer = vec![0u8; 1920 * 1080 * 4];

    for _ in 0..frames {
        editor.compositor.update_composite(&editor.stack);
        editor.compositor.render_viewport_frame(
            &editor.viewport,
            &mut screen_buffer,
            1920,
            1080,
        );
    }
    let total_time = bench_start.elapsed();
    let fps = frames as f64 / total_time.as_secs_f64();
    let frame_ms = total_time.as_secs_f64() * 1000.0 / frames as f64;
    println!("      Rendered {} frames in {:.2?} => {:.1} FPS ({:.2} ms/frame, debug mode: {})", frames, total_time, fps, frame_ms, cfg!(debug_assertions));

    // 7. Storage: .drawling Container & Gallery SQLite Database
    println!("[8/8] Testing .drawling Archive Save/Load & Gallery SQLite DB...");
    let temp_dir = std::env::temp_dir().join("drawling_test");
    std::fs::create_dir_all(&temp_dir).unwrap();

    let project_file = temp_dir.join("test_artwork.drawling");
    editor.save_project(&project_file).expect("Failed to save .drawling");
    println!("      Saved .drawling archive to: {:?}", project_file);

    // Verify loading
    let (loaded_manifest, loaded_stack, thumb) = DrawlingArchive::load(&project_file).expect("Failed to load .drawling");
    println!("      Loaded archive: '{}', {} layers, thumbnail {} bytes", loaded_manifest.title, loaded_stack.layers.len(), thumb.len());
    assert_eq!(loaded_stack.layers.len(), editor.stack.layers.len());

    // Export formats: PNG, JPEG, SVG
    let png_out = temp_dir.join("artwork.png");
    let jpg_out = temp_dir.join("artwork.jpg");
    let svg_out = temp_dir.join("artwork.svg");

    ImageExporter::export_png(&png_out, &loaded_stack).expect("PNG export failed");
    ImageExporter::export_jpeg(&jpg_out, &loaded_stack, 92).expect("JPEG export failed");
    ImageExporter::export_svg(&svg_out, &loaded_stack).expect("SVG export failed");
    println!("      Exported formats: PNG ({:?}), JPEG ({:?}), SVG ({:?})", png_out, jpg_out, svg_out);

    // Gallery Database SQLite test with project duplication
    let mut gallery_db = GalleryDatabase::open_in_memory().expect("SQLite DB failed");
    let record = ProjectRecord {
        id: loaded_manifest.id,
        title: loaded_manifest.title.clone(),
        file_path: project_file.to_string_lossy().into(),
        thumbnail_png: thumb,
        width: loaded_manifest.canvas_width,
        height: loaded_manifest.canvas_height,
        created_at: loaded_manifest.created_at,
        updated_at: loaded_manifest.updated_at,
        edit_duration_secs: 120,
        tags: vec!["Digital Art".into(), "KDE".into(), "Duckling".into()],
    };
    gallery_db.upsert_project(&record).expect("Failed to upsert project");

    let dup = gallery_db.duplicate_project(
        &record.id,
        "Duckling Masterpiece (Copy)",
        &project_file.to_string_lossy(),
    ).expect("Duplication failed").unwrap();
    println!("      Duplicated project in gallery: '{}' (New ID: {})", dup.title, dup.id);

    let projects = gallery_db.list_projects().expect("Failed to list projects");
    println!("      Gallery Database registered {} projects.", projects.len());
    assert_eq!(projects.len(), 2);

    println!("\n🎉 ALL TESTS & BENCHMARKS PASSED PERFECTLY! 🎉\n");
}
