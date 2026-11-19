// RIFE input: the frame box-filtered down to the model's resolution,
// gamma-encoded RGB in 0..1 (what the network was trained on).

struct Prep {
    out_size: vec2<f32>,
    linear: u32,
    _p0: u32,
};

@group(0) @binding(0) var<uniform> P: Prep;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var samp: sampler;

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

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // Four bilinear taps spread over the output texel: roughly a box filter for
    // any reduction up to ~4:1.
    let px = 1.0 / P.out_size;
    let uv = in.pos.xy * px;
    var c = vec3<f32>(0.0);
    for (var i = 0; i < 4; i++) {
        let o = vec2<f32>(f32(i & 1) - 0.5, f32(i >> 1u) - 0.5) * 0.5 * px;
        c += max(textureSampleLevel(src, samp, uv + o, 0.0).rgb, vec3<f32>(0.0));
    }
    c *= 0.25;
    if (P.linear == 1u) {
        c = pow(c, vec3<f32>(1.0 / 2.2));
    }
    return vec4<f32>(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
