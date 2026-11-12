// Drawling for Duckling - WGPU Shader Pipeline for High Performance Canvas & Layer Compositing

struct Uniforms {
    view_matrix: mat4x4<f32>,
    layer_opacity: f32,
    blend_mode: u32,
    alpha_locked: u32,
    clipping_mask: u32,
};

@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var t_dest: texture_2d<f32>;
@group(0) @binding(2) var s_dest: sampler;
@group(0) @binding(3) var t_src: texture_2d<f32>;
@group(0) @binding(4) var s_src: sampler;
@group(0) @binding(5) var t_clip: texture_2d<f32>;
@group(0) @binding(6) var s_clip: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = u.view_matrix * vec4<f32>(in.position, 0.0, 1.0);
    out.uv = in.uv;
    return out;
}

// 13 Blend modes implemented directly in WGSL for GPU speed
fn blend_channel(mode: u32, cb: f32, cs: f32) -> f32 {
    switch (mode) {
        case 0u: { // Normal
            return cs;
        }
        case 1u: { // Multiply
            return cb * cs;
        }
        case 2u: { // Screen
            return cb + cs - (cb * cs);
        }
        case 3u: { // Overlay
            if (cb <= 0.5) {
                return 2.0 * cb * cs;
            } else {
                return 1.0 - 2.0 * (1.0 - cb) * (1.0 - cs);
            }
        }
        case 4u: { // Darken
            return min(cb, cs);
        }
        case 5u: { // Lighten
            return max(cb, cs);
        }
        case 6u: { // ColorDodge
            if (cb <= 0.0) { return 0.0; }
            if (cs >= 1.0) { return 1.0; }
            return min(1.0, cb / (1.0 - cs));
        }
        case 7u: { // ColorBurn
            if (cb >= 1.0) { return 1.0; }
            if (cs <= 0.0) { return 0.0; }
            return 1.0 - min(1.0, (1.0 - cb) / cs);
        }
        case 8u: { // Add
            return min(1.0, cb + cs);
        }
        case 9u: { // Subtract
            return max(0.0, cb - cs);
        }
        case 10u: { // Difference
            return abs(cb - cs);
        }
        case 11u: { // SoftLight
            if (cs <= 0.5) {
                return cb - (1.0 - 2.0 * cs) * cb * (1.0 - cb);
            } else {
                var d: f32;
                if (cb <= 0.25) {
                    d = ((16.0 * cb - 12.0) * cb + 4.0) * cb;
                } else {
                    d = sqrt(cb);
                }
                return cb + (2.0 * cs - 1.0) * (d - cb);
            }
        }
        case 12u: { // HardLight
            if (cs <= 0.5) {
                return 2.0 * cb * cs;
            } else {
                return 1.0 - 2.0 * (1.0 - cb) * (1.0 - cs);
            }
        }
        default: {
            return cs;
        }
    }
}

@fragment
fn fs_composite(in: VertexOutput) -> @location(0) vec4<f32> {
    let dst = textureSample(t_dest, s_dest, in.uv);
    var src = textureSample(t_src, s_src, in.uv);

    // Clipping mask check
    if (u.clipping_mask != 0u) {
        let clip_px = textureSample(t_clip, s_clip, in.uv);
        src.a = src.a * clip_px.a;
    }

    let sa = src.a * clamp(u.layer_opacity, 0.0, 1.0);
    if (sa <= 0.0) {
        return dst;
    }

    let da = dst.a;
    if (da <= 0.0 && u.alpha_locked == 0u) {
        return vec4<f32>(src.rgb, sa);
    }
    if (u.alpha_locked != 0u && da <= 0.0) {
        return dst;
    }

    let out_a = select(sa + da * (1.0 - sa), da, u.alpha_locked != 0u);
    if (out_a <= 0.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let br = blend_channel(u.blend_mode, dst.r, src.r);
    let bg = blend_channel(u.blend_mode, dst.g, src.g);
    let bb = blend_channel(u.blend_mode, dst.b, src.b);

    let out_r = (sa * (1.0 - da) * src.r + sa * da * br + (1.0 - sa) * da * dst.r) / out_a;
    let out_g = (sa * (1.0 - da) * src.g + sa * da * bg + (1.0 - sa) * da * dst.g) / out_a;
    let out_b = (sa * (1.0 - da) * src.b + sa * da * bb + (1.0 - sa) * da * dst.b) / out_a;

    return vec4<f32>(clamp(out_r, 0.0, 1.0), clamp(out_g, 0.0, 1.0), clamp(out_b, 0.0, 1.0), clamp(out_a, 0.0, 1.0));
}
