struct Uniforms {
    res: vec2<f32>,
    point_size: f32,
    time: f32,
};

@group(0) @binding(0) var<uniform> u: Uniforms;

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> @builtin(position) vec4<f32> {
    var p = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    return vec4<f32>(p[vid], 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let res = max(u.res, vec2<f32>(1.0, 1.0));
    let uv = pos.xy / res;
    let top = vec3<f32>(0.020, 0.055, 0.130);
    let mid = vec3<f32>(0.008, 0.028, 0.085);
    let bot = vec3<f32>(0.004, 0.012, 0.045);
    var col = mix(top, mid, clamp(uv.y * 1.30, 0.0, 1.0));
    col = mix(col, bot, clamp((uv.y - 0.45) * 1.6, 0.0, 1.0));

    let gx = abs(fract(pos.x / 48.0) - 0.5);
    let gy = abs(fract(pos.y / 48.0) - 0.5);
    let grid = (1.0 - smoothstep(0.0, 0.03, gx)) + (1.0 - smoothstep(0.0, 0.03, gy));
    col += vec3<f32>(0.12, 0.28, 0.55) * grid * 0.22;

    if (uv.y > 0.62) {
        let py = (uv.y - 0.62) / 0.38;
        let persp = 0.16 / max(py, 0.04);
        let hx = abs(fract((uv.x - 0.5) * persp * 6.0 + 0.5) - 0.5);
        let hy = abs(fract(pow(py, 0.65) * 9.0) - 0.5);
        let floorg = (1.0 - smoothstep(0.0, 0.04, hx)) * 0.40
            + (1.0 - smoothstep(0.0, 0.06, hy)) * 0.28;
        col += vec3<f32>(0.18, 0.40, 0.78) * floorg * py;
    }

    let horizon = 1.0 - smoothstep(0.0, 0.014, abs(uv.y - 0.62));
    col += vec3<f32>(0.55, 0.75, 1.0) * horizon * 0.28;

    let scan = 0.025 * step(0.70, fract(pos.y * 0.5));
    col *= 1.0 - scan;
    let sweep = fract(u.time * 0.035);
    let band = 1.0 - smoothstep(0.0, 0.02, abs(uv.y - sweep));
    col += vec3<f32>(0.35, 0.55, 1.0) * band * 0.08;

    let edge = min(min(uv.x, uv.y), min(1.0 - uv.x, 1.0 - uv.y));
    col += vec3<f32>(0.35, 0.55, 0.95) * (1.0 - smoothstep(0.0, 0.03, edge)) * 0.55;
    let vig = smoothstep(0.15, 0.95, distance(uv, vec2<f32>(0.50, 0.40)));
    col *= 1.0 - 0.40 * vig;
    return vec4<f32>(col, 1.0);
}
