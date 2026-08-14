struct Uniforms {
    res: vec2<f32>,
    point_size: f32,
    time: f32,
};

@group(0) @binding(0) var<uniform> u: Uniforms;

struct VsIn {
    @location(0) pos: vec2<f32>,
    @location(1) rgba: vec4<f32>,
    @location(2) size: f32,
};

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) rgba: vec4<f32>,
};

@vertex
fn vs_main(in: VsIn, @builtin(vertex_index) vid: u32) -> VsOut {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0),
    );
    let c = corners[vid];
    let sz = max(in.size, 0.80);
    let pixel = in.pos + c * sz;
    let clip = vec2<f32>(
        pixel.x / u.res.x * 2.0 - 1.0,
        1.0 - pixel.y / u.res.y * 2.0,
    );
    var out: VsOut;
    out.clip = vec4<f32>(clip, 0.0, 1.0);
    out.uv = c;
    out.rgba = in.rgba;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let dist = length(in.uv);
    let core = 1.0 - smoothstep(0.0, 0.22, dist);
    let glow = 1.0 - smoothstep(0.12, 1.0, dist);
    let a = in.rgba.a * (core * 1.15 + glow * 0.45);
    if (a < 0.02) {
        discard;
    }
    let rgb = in.rgba.rgb;
    return vec4<f32>(rgb, min(a, 1.0));
}
