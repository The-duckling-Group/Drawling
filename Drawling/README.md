# 🦆 Drawling for Duckling

> **A High-Performance Digital Drawing & Illustration Studio built in Rust.**  
> Designed for Linux (optimized for KDE Plasma) and Windows, inspired by industry-standard software like IbisPaint and Clip Studio Paint.

---

## 🌟 Key Architecture & Modular Workspace

`drawling` is built as an ultra-modular Rust workspace composed of 4 dedicated crates:

```
drawling/
├── crates/
│   ├── drawling-core/       # Pure high-performance math, stroke stabilization, brush engine, tools, layers, filters
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

### 2. Illustration Tools & Vector Shapes
- **Smudge & Color Blending Engine**:
  - Natural paint pickup, blending, and color dragging across canvas strokes with adjustable bleed and strength.
- **Flood Fill (Bucket Tool)**:
  - 4-way BFS flood fill with Euclidean color distance tolerance and contiguous/global modes.
  - **Lineart Halo Expansion (Dilation)**: Expands fill $N$ pixels underneath lineart borders to eliminate white halos.
- **Geometric Vector Shapes**:
  - Anti-aliased straight lines with stroke width.
  - Outlined and filled Rectangles and Ellipses.
  - Regular Stars with customizable ray counts and inner/outer radius ratios.
- **Eyedropper / Color Picker**:
  - Point ($1\times1$) and area ($3\times3$, $5\times5$) alpha-weighted sampling from active layer or composited canvas stack.

### 3. Artist Palettes & Color Management
- **Built-in Illustration Palettes**:
  - **Neon Cyberpunk**: Electric cyan, neon crimson, acid yellow, cyber violet, matrix green.
  - **Anime & Portrait Skin Tones**: Light highlights, midtones, and deep shadow skin shades.
  - **Manga Inking & Screentone**: Pure black, 10%, 20%, 30%, 50%, 70% dot tones, pure white.
  - **Pastel Anime**: Soft sakura pink, peach, mint cream, aqua, periwinkle, lavender.
  - **Classic Oil & Gouache**: Burnt sienna, ochre, ultramarine, British racing green, cadmium red.
- **Recent Colors Buffer**: Automatic FIFO ring-buffer tracking recently used swatches.
- **Hex Import/Export**: Quick import/export of color palettes from hex lists.

### 4. Advanced Layer Management
- **13 Blend Modes** implemented with mathematical precision:
  `Normal`, `Multiply`, `Screen`, `Overlay`, `Darken`, `Lighten`, `Color Dodge`, `Color Burn`, `Add (Linear Dodge)`, `Subtract`, `Difference`, `Soft Light`, `Hard Light`.
- **Clipping Masks**: Clip any layer against the alpha channel of base layers below.
- **Alpha Lock**: Paint exclusively within existing pixel boundaries without expanding transparency.
- **Hierarchical Layer Stack**: Add, duplicate, reorder, merge down, flatten, and adjust opacity.

### 5. Selection & Transformation Tools
- **Selection Tools**:
  - Rectangle and Ellipse selections.
  - **Freehand Lasso**: Polygon scanline rasterization with even-odd fill rules.
  - **Magic Wand**: Flood fill with Euclidean color-distance tolerance and edge feathering.
  - **Boolean Mask Operations**: Union, Subtract, Intersect, Expand, and Shrink.
- **Transformations**:
  - 2D Affine Transformation matrix (Translation, Rotation, Scaling, Skew) with high-quality Bilinear Resampling.
  - Mesh-Warp grid and Perspective vanishing-point guides with angular snapping.
- **Symmetry Guides**:
  - Horizontal & Vertical mirror axes, dual-axis cross reflection.
  - **N-Fold Radial Symmetry** (Kaleidoscope / Mandala drawing with optional mirror flipping).

### 6. Multi-Threaded Image Processing & Artistic Filters (Rayon)
- **Gaussian Blur**: Multi-threaded separable 1D convolution with arbitrary radius ($\sigma$).
- **Motion Blur**: Directional linear blur with configurable angle and distance.
- **Radial / Zoom Blur**: Blur radiating outward from a focus point for dynamic action effects.
- **Unsharp Mask**: Edge-enhancing high-pass filter with radius, amount, and threshold.
- **Manga Halftone (Screentone)**: Rotated dot grid pattern generator for authentic comic shading.
- **Chromatic Aberration**: Optical color fringing displacement for retro lens styling.
- **Bloom / Glow**: Luminance thresholding, Gaussian blur, and additive screen blend.
- **Mosaic / Pixelate**: Block-based color averaging for retro or stylized effects.
- **Film Grain / Noise**: Monochromatic or RGB pseudo-random photographic texture.
- **Posterize & Threshold**: Color quantization and strict binary black/white conversion.
- **Levels & Curves**: Comprehensive tone curve adjustment with black point, white point, and midtone gamma.
- **Color Adjustments**: Real-time Brightness/Contrast, Hue/Saturation, Invert, Sepia, Vignette, and Gradient Map.

### 7. File Formats & Local Gallery
- **`.drawling` Container Format**:
  - Compressed zip archive holding `project.json` (metadata, canvas dimensions, layer stack), `thumbnail.png`, and lossless PNG buffers for each layer (`layers/{layer_id}.png`).
- **Export Pipelines**: Lossless PNG, JPEG (with quality control & white-matte alpha composition), scalable SVG with embedded high-res assets.
- **Local Gallery (SQLite)**:
  - Local SQLite database (`projects.db`) storing artwork records, thumbnails, creation/modification timestamps, tags, and editing duration.
  - Auto-save journal for crash-recovery and undo history.

### 8. UI / UX Design ("Smooth & Clean")
- **Themes**:
  - **KDE Plasma Breeze Dark** (`#232629` surface, `#3daee9` Plasma blue accent).
  - **Modern Windows 11 Dark** (Mica dark slate, `#60cdff` accent).
- **Zen-Mode**: Toggle with `Tab` or `F11` to hide all UI chrome for distraction-free illustration.
- **Keyboard Shortcut Manager**:
  - Tools: Brush (`B`), Eraser (`E`), Smudge (`J`), Bucket (`G`), Eyedropper (`I`), Shape (`U`), Lasso (`L`), Magic Wand (`W`), Transform (`Ctrl+T`).
  - Filters: Invert (`Ctrl+I`), Levels (`Ctrl+L`), Hue/Sat (`Ctrl+U`), Blur (`Ctrl+Alt+B`), Motion Blur (`Ctrl+Alt+M`).
  - Standard: Undo (`Ctrl+Z`), Redo (`Ctrl+Y`), Save (`Ctrl+S`), Export (`Ctrl+E`), Zen-mode (`Tab`/`F11`).

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
