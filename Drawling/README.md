# 🦆 Drawling for Duckling

> **A High-Performance Digital Drawing & Illustration Studio built in Rust.**  
> Designed for Linux (optimized for KDE Plasma) and Windows, inspired by industry-standard software like IbisPaint and Clip Studio Paint.

---

## 🌟 Key Architecture & Modular Workspace

`drawling` is built as an ultra-modular Rust workspace composed of 4 dedicated crates:

```
drawling/
├── crates/
│   ├── drawling-core/       # Pure high-performance math, stroke stabilization, brush engine, layers, filters
│   ├── drawling-storage/    # .drawling container format, SQLite gallery database, PNG/JPEG/SVG exporters
│   ├── drawling-render/     # WGPU shader pipeline (WGSL) & Rayon-accelerated canvas compositor
│   └── drawling-app/        # Desktop application orchestration, state machine, theme & shortcut manager
├── Cargo.toml
└── README.md
```

---

## 🎨 Features & Capabilities

### 1. High-Performance Brush Engine & Real-Time Stabilization
- **Stroke Stabilization**:
  - **Lazy Mouse / Rope Mode**: Pulled-string anchor model with adjustable rope radius and damping tension.
  - **Moving Average**: Windowed smoothing filter for responsive stylus input.
  - **Catmull-Rom Splines**: Centripetal parameterization ($\alpha = 0.5$) preventing loops, sharp cusps, and self-intersections during rapid strokes.
- **Brush Dynamics**:
  - Sub-pixel dab generation with spacing, flow, scatter dispersion, and smoothstep hardness falloff.
  - Pressure and tilt sensitivity (ready for Wintab, X11, Wayland stylus tablets).
- **Brush Presets & Extensibility**:
  - Built-in library: Inking G-Pen, Soft Airbrush, Hard Round, Textured HB Pencil, Acrylic Flat.
  - Full JSON preset serialization and Adobe Photoshop (`.abr`) brush resource parsing.

### 2. Advanced Layer Management
- **13 Blend Modes** implemented with mathematical precision:
  `Normal`, `Multiply`, `Screen`, `Overlay`, `Darken`, `Lighten`, `Color Dodge`, `Color Burn`, `Add (Linear Dodge)`, `Subtract`, `Difference`, `Soft Light`, `Hard Light`.
- **Clipping Masks**: Clip any layer against the alpha channel of base layers below.
- **Alpha Lock**: Paint exclusively within existing pixel boundaries without expanding transparency.
- **Hierarchical Layer Stack**: Add, duplicate, reorder, merge, and adjust opacity.

### 3. Selection & Transformation Tools
- **Selection Tools**:
  - Rectangle and Ellipse selections.
  - **Freehand Lasso**: Polygon scanline rasterization with even-odd fill rules.
  - **Magic Wand**: Flood fill with Euclidean color-distance tolerance and edge feathering.
- **Transformations**:
  - 2D Affine Transformation matrix (Translation, Rotation, Scaling, Skew) with high-quality Bilinear Resampling.
  - Mesh-Warp grid and Perspective vanishing-point guides.
- **Symmetry Guides**:
  - Horizontal & Vertical mirror axes.
  - Dual-Axis cross reflection.
  - **N-Fold Radial Symmetry** (Kaleidoscope / Mandala drawing with optional mirror flipping).

### 4. Multi-Threaded Image Processing Filters (Rayon)
- **Gaussian Blur**: Multi-threaded separable 1D convolution with arbitrary radius ($\sigma$).
- **Color Adjustments**: Real-time Brightness/Contrast and Hue/Saturation shift in HSV space.
- **Gradient Map**: Re-maps grayscale intensity onto multi-stop color ramps using 256-level precomputed LUTs.

### 5. File Formats & Local Gallery
- **`.drawling` Container Format**:
  - Compressed zip archive holding `project.json` (metadata, canvas dimensions, layer stack), `thumbnail.png`, and lossless PNG buffers for each layer (`layers/{layer_id}.png`).
- **Export Pipelines**: Lossless PNG, JPEG (with quality control & white-matte alpha composition), scalable SVG with embedded high-res assets.
- **Local Gallery (SQLite)**:
  - Local SQLite database (`projects.db`) storing artwork records, thumbnails, creation/modification timestamps, tags, and editing duration.
  - Auto-save journal for crash-recovery and undo history.

### 6. UI / UX Design ("Smooth & Clean")
- **Themes**:
  - **KDE Plasma Breeze Dark** (`#232629` surface, `#3daee9` Plasma blue accent).
  - **Modern Windows 11 Dark** (Mica dark slate, `#60cdff` accent).
- **Zen-Mode**: Toggle with `Tab` or `F11` to hide all UI chrome for distraction-free illustration.
- **Keyboard Shortcut Manager**: Clip Studio / Photoshop-aligned shortcuts (Ctrl+Z, Ctrl+Y, B, E, I, G, L, W, Ctrl+T, Ctrl+S).

---

## 🚀 Running the Project

### Run Unit Tests
```bash
cargo test --workspace
```

### Run the Full System Verification & Benchmark
```bash
cargo run --release -p drawling-app -- --demo
```

---

## 📄 License
Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).
