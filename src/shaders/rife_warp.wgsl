// RIFE frame generation, full-resolution warp: the network (run at reduced
// resolution) gives, for each pixel of the frame at t = 0.5, an offset into the
// older frame (xy), one into the newer frame (zw) and the older frame's blend
// weight. Offsets are upsampled here and scaled to this resolution. Other t are
// approximated by scaling the t = 0.5 offsets (2t toward the older frame, 2(1-t)
// toward the newer) and re-weighting the blend.

struct Warp {
    size: vec2<f32>,  // full-resolution frame, px
    fsize: vec2<f32>, // model input, px (flow/mask texel grid starts at the same origin)
    t: f32,
    _p0: f32,
    _p1: f32,
    _p2: f32,
};

@group(0) @binding(0) var<uniform> W: Warp;
@group(0) @binding(1) var older: texture_2d<f32>;
@group(0) @binding(2) var newer: texture_2d<f32>;
@group(0) @binding(3) var flow_tex: texture_2d<f32>; // Rgba32Float, not filterable
@group(0) @binding(4) var mask_tex: texture_2d<f32>; // R32Float, not filterable
@group(0) @binding(5) var samp: sampler;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    var o: VsOut;
    let x = f32((vi << 1u) & 2u);
    let y = f32(vi & 2u);
    o.pos = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    return o;
}

// Manual bilinear (32-bit float textures aren't filterable everywhere).
fn bil(t: texture_2d<f32>, p: vec2<f32>) -> vec4<f32> {
    let hi = vec2<i32>(W.fsize) - vec2<i32>(1);
    let q = p - vec2<f32>(0.5);
    let i = vec2<i32>(floor(q));
    let f = q - floor(q);
    let i00 = clamp(i, vec2<i32>(0), hi);
    let i11 = clamp(i + vec2<i32>(1), vec2<i32>(0), hi);
    let a = textureLoad(t, vec2<i32>(i00.x, i00.y), 0);
    let b = textureLoad(t, vec2<i32>(i11.x, i00.y), 0);
    let c = textureLoad(t, vec2<i32>(i00.x, i11.y), 0);
    let d = textureLoad(t, vec2<i32>(i11.x, i11.y), 0);
    return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let x = in.pos.xy;
    let k = W.fsize / W.size;      // full px -> model px
    let fp = x * k;
    let f = bil(flow_tex, fp) / vec4<f32>(k, k);  // offsets in full px
    let m = bil(mask_tex, fp).r;
    let pa = x + f.xy * (2.0 * W.t);
    let pb = x + f.zw * (2.0 * (1.0 - W.t));
    let ca = textureSampleLevel(older, samp, pa / W.size, 0.0).rgb;
    let cb = textureSampleLevel(newer, samp, pb / W.size, 0.0).rgb;
    let w0 = m * (1.0 - W.t);
    let w1 = (1.0 - m) * W.t;
    let a = w0 / max(w0 + w1, 1e-6);
    return vec4<f32>(mix(cb, ca, a), 1.0);
}
