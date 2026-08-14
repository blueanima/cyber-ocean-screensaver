struct Uniforms {
    res: vec2<f32>,
    point_size: f32,
    time: f32,
};

@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var atlas: texture_2d<f32>;
@group(0) @binding(2) var atlas_samp: sampler;

struct VsIn {
    @location(0) pos: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) uv0: vec2<f32>,
    @location(3) uv1: vec2<f32>,
    @location(4) rgba: vec4<f32>,
    @location(5) extra: vec2<f32>,
};

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) rgba: vec4<f32>,
    @location(3) extra: vec2<f32>,
    @location(4) half_size: vec2<f32>,
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
    let ang = in.extra.y;
    let ca = cos(ang);
    let sa = sin(ang);
    let local = c * in.size * 0.5;
    let rot = vec2<f32>(local.x * ca - local.y * sa, local.x * sa + local.y * ca);
    let pixel = in.pos + rot;
    let clip = vec2<f32>(
        pixel.x / u.res.x * 2.0 - 1.0,
        1.0 - pixel.y / u.res.y * 2.0,
    );
    var out: VsOut;
    out.clip = vec4<f32>(clip, 0.0, 1.0);
    out.local = local;
    out.uv = mix(in.uv0, in.uv1, c * 0.5 + vec2<f32>(0.5, 0.5));
    out.rgba = in.rgba;
    out.extra = in.extra;
    out.half_size = in.size * 0.5;
    return out;
}

fn sd_round_box(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - b + vec2<f32>(r, r);
    return length(max(q, vec2<f32>(0.0, 0.0))) + min(max(q.x, q.y), 0.0) - r;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    if (in.extra.x < -0.5) {
        let a = textureSample(atlas, atlas_samp, in.uv).r;
        if (a < 0.02) {
            discard;
        }
        return vec4<f32>(in.rgba.rgb, in.rgba.a * a);
    }
    let radius = in.extra.x;
    if (radius > 0.5) {
        let d = sd_round_box(in.local, in.half_size, radius);
        let a = in.rgba.a * (1.0 - smoothstep(-0.8, 0.8, d));
        if (a < 0.01) {
            discard;
        }
        let edge = 1.0 - smoothstep(-1.2, 0.4, abs(d + 0.4));
        let rgb = mix(in.rgba.rgb, vec3<f32>(0.85, 0.92, 1.0), edge * 0.22);
        return vec4<f32>(rgb, a);
    }
    return in.rgba;
}
