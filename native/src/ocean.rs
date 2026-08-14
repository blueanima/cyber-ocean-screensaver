use crate::formulas::{FillFn, SPECIES, VIEW};

pub struct Creature {
    pub ci: usize,
    pub x: f64,
    pub y: f64,
    pub scale: f64,
    pub rot: f64,
    pub t: f64,
    pub tempo: f64,
    pub vx: f64,
    pub vy: f64,
    pub wobble: f64,
    pub phase: f64,
    pub pulse: f64,
    pub fx: f64,
    pub fy: f64,
    pub ax: f32,
    pub ay: f32,
    pub radius: f32,
    pub cx: f64,
    pub cy: f64,
    pub rms: f64,
    pub ready: bool,
    pub phi: f64,
    pub amp: f64,
    pub omega: f64,
    pub bias: f64,
    pub speed: f64,
    pub bell: f64,
    pub pose_sway: f64,
    pub pose_spin: f64,
    pub wall_dir: f64,
    pub face: f64,
}

fn mulberry32(mut a: u32) -> impl FnMut() -> f64 {
    move || {
        a = a.wrapping_add(0x6D2B79F5);
        let mut tt = (a ^ (a >> 15)).wrapping_mul(1 | a);
        tt = tt.wrapping_add((tt ^ (tt >> 7)).wrapping_mul(61 | tt)) ^ tt;
        ((tt ^ (tt >> 14)) as f64) / 4_294_967_296.0
    }
}

fn shuffled(n: usize, rand: &mut impl FnMut() -> f64) -> Vec<usize> {
    let mut a: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        let j = (rand() * (i + 1) as f64) as usize;
        a.swap(i, j);
    }
    a
}

pub fn spawn(seed: u32, count: usize) -> Vec<Creature> {
    let mut rand = mulberry32(seed);
    let n_c = SPECIES.len();
    let count = count.clamp(1, n_c);
    let order = shuffled(n_c, &mut rand);
    let mut inst = Vec::with_capacity(count);
    for i in 0..count {
        let ang = rand() * std::f64::consts::TAU;
        let ci = order[i];
        let g = crate::gait::gait(ci);
        let tempo = 0.72 + rand() * 0.28;
        let omega = g.hz * std::f64::consts::TAU;
        let speed = g.cruise;
        inst.push(Creature {
            ci,
            x: 0.08 + rand() * 0.84,
            y: 0.10 + rand() * 0.80,
            scale: 0.30 + rand() * 0.10,
            rot: ang,
            t: 0.6 + rand() * 4.0,
            tempo,
            vx: speed * ang.cos(),
            vy: -speed * ang.sin(),
            wobble: 0.18 + rand() * 0.22,
            phase: rand() * std::f64::consts::PI * 2.0,
            pulse: 0.0,
            fx: 0.0,
            fy: 0.0,
            ax: 0.0,
            ay: 0.0,
            radius: 40.0,
            cx: VIEW * 0.5,
            cy: VIEW * 0.5,
            rms: 40.0,
            ready: false,
            phi: rand() * std::f64::consts::TAU,
            amp: 0.85 + rand() * 0.15,
            omega,
            bias: (rand() - 0.5) * 0.2,
            speed,
            bell: 1.0,
            pose_sway: 0.0,
            pose_spin: 0.0,
            wall_dir: 0.0,
            face: 0.0,
        });
    }
    inst
}

fn shape_stats(pts: &[[f32; 2]]) -> (f64, f64, f64) {
    if pts.len() < 8 {
        return (VIEW * 0.5, VIEW * 0.5, 40.0);
    }
    let stride = (pts.len() / 480).max(1);
    let mut sx = 0.0;
    let mut sy = 0.0;
    let mut n = 0.0;
    for p in pts.iter().step_by(stride) {
        sx += p[0] as f64;
        sy += p[1] as f64;
        n += 1.0;
    }
    if n < 4.0 {
        return (VIEW * 0.5, VIEW * 0.5, 40.0);
    }
    let cx = sx / n;
    let cy = sy / n;
    let mut acc = 0.0;
    for p in pts.iter().step_by(stride) {
        let dx = p[0] as f64 - cx;
        let dy = p[1] as f64 - cy;
        acc += dx * dx + dy * dy;
    }
    (cx, cy, (acc / n).sqrt().max(8.0))
}

fn wrap_pi(a: f64) -> f64 {
    (a + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI
}

fn shape_face(pts: &[[f32; 2]], cx: f64, cy: f64, prev: f64, dt: f64) -> f64 {
    if pts.len() < 24 {
        return prev;
    }
    let stride = (pts.len() / 360).max(1);
    let mut xx = 0.0;
    let mut xy = 0.0;
    let mut yy = 0.0;
    let mut n = 0.0;
    for p in pts.iter().step_by(stride) {
        let dx = p[0] as f64 - cx;
        let dy = p[1] as f64 - cy;
        xx += dx * dx;
        xy += dx * dy;
        yy += dy * dy;
        n += 1.0;
    }
    if n < 10.0 {
        return prev;
    }
    xx /= n;
    xy /= n;
    yy /= n;
    let tr = xx + yy;
    let det = xx * yy - xy * xy;
    let disc = (tr * tr - 4.0 * det).max(0.0).sqrt();
    let l1 = 0.5 * (tr + disc);
    let l2 = 0.5 * (tr - disc);
    if l1 < 1.8 * l2.max(4.0) {
        return prev;
    }
    let (mut ex, mut ey) = if xy.abs() > 1e-8 || (l1 - xx).abs() > 1e-8 {
        (xy, l1 - xx)
    } else {
        (1.0, 0.0)
    };
    let el = (ex * ex + ey * ey).sqrt().max(1e-9);
    ex /= el;
    ey /= el;
    let mut mpos: f64 = 0.0;
    let mut mneg: f64 = 0.0;
    for p in pts.iter().step_by(stride) {
        let s = (p[0] as f64 - cx) * ex + (p[1] as f64 - cy) * ey;
        if s > 0.0 {
            mpos = mpos.max(s);
        } else {
            mneg = mneg.max(-s);
        }
    }
    if mpos > mneg * 1.12 {
        ex = -ex;
        ey = -ey;
    }
    let mut ang = ey.atan2(ex);
    let mut d = wrap_pi(ang - prev);
    if d.abs() > std::f64::consts::FRAC_PI_2 {
        ang += std::f64::consts::PI;
        d = wrap_pi(ang - prev);
    }
    let max_step = 0.28 * dt;
    prev + d.clamp(-max_step, max_step)
}

pub fn advance_morph(inst: &mut [Creature], dt: f64) {
    for c in inst.iter_mut() {
        let spec = &SPECIES[c.ci];
        let g = crate::gait::gait(c.ci);
        let mu = 1.0 + 0.35 * c.pulse;
        let r = c.amp.max(0.12);
        c.amp += dt * 2.4 * (mu - r * r) * r;
        c.amp = c.amp.clamp(0.12, 1.35);
        c.omega = g.hz
            * std::f64::consts::TAU
            * (0.88 + 0.18 * c.amp)
            * (0.94 + 0.12 * c.wobble)
            * (0.92 + 0.16 * c.tempo);
        if c.pulse > 0.0 {
            c.omega *= 1.0 + 0.45 * c.pulse;
            c.pulse = (c.pulse - dt * 0.9).max(0.0);
        }
        c.phi += c.omega * dt;
        let d = crate::gait::drive(g.kind, g, c.phi, c.amp, c.bias, c.speed, dt);
        c.t += spec.dt * dt * (7.0 + c.omega * 8.0) * c.amp * d.morph;
    }
}

pub fn integrate(
    inst: &mut [Creature],
    scratches: &[Vec<[f32; 2]>],
    ocean_t: f64,
    dt: f64,
    pointer: (f64, f64),
    size: (f32, f32),
    saver: bool,
) {
    let (w, h) = (size.0 as f64, size.1 as f64);
    let top_g = if saver { 12.0 } else { 64.0 };
    let bot_g = if saver { 12.0 } else { 92.0 };
    let side_g = 8.0;
    let inner_w = (w - 2.0 * side_g).max(40.0);
    let inner_h = (h - top_g - bot_g).max(40.0);
    let world = inner_w.min(inner_h);
    let cam_x = (pointer.0 - 0.5) * 40.0;
    let cam_y = (pointer.1 - 0.5) * 22.0;
    let mx = pointer.0 * w;
    let my = pointer.1 * h;
    let dt = dt.max(1.0 / 240.0);
    let n = inst.len();

    for c in inst.iter_mut() {
        c.fx = 0.0;
        c.fy = 0.0;
    }
    for a in 0..n {
        for b in (a + 1)..n {
            let ddx = inst[a].x - inst[b].x;
            let ddy = inst[a].y - inst[b].y;
            let dd = (ddx * ddx + ddy * ddy).sqrt();
            let sep = (inst[a].scale + inst[b].scale) * 0.34;
            if dd < 1e-4 || dd >= sep {
                continue;
            }
            let nx = ddx / dd;
            let ny = ddy / dd;
            let push = (1.0 - dd / sep).powi(2);
            inst[a].fx += nx * push;
            inst[a].fy += ny * push;
            inst[b].fx -= nx * push;
            inst[b].fy -= ny * push;
        }
    }

    for (i, c) in inst.iter_mut().enumerate() {
        let pts = scratches.get(i).map(|s| s.as_slice()).unwrap_or(&[]);
        let g = crate::gait::gait(c.ci);
        let (raw_cx, raw_cy, raw_rms) = shape_stats(pts);
        let (cx, cy, rms) = if g.kind == crate::gait::GaitKind::SpinDrift {
            (VIEW * 0.5, VIEW * 0.5, raw_rms)
        } else {
            (raw_cx, raw_cy, raw_rms)
        };
        let sc = c.scale * world / VIEW;
        let mut stroke = 0.0;
        if c.ready {
            let lx = (cx - c.cx).clamp(-48.0, 48.0);
            let ly = (cy - c.cy).clamp(-48.0, 48.0);
            stroke = (lx * lx + ly * ly).sqrt() + (c.rms - rms).abs() * 0.45;
            stroke += (c.rms - rms).max(0.0);
            c.cx += (cx - c.cx) * 0.08;
            c.cy += (cy - c.cy) * 0.08;
            c.rms += (rms - c.rms) * 0.08;
        } else {
            c.cx = cx;
            c.cy = cy;
            c.rms = rms;
        }
        c.ready = true;
        let elongated = matches!(
            g.kind,
            crate::gait::GaitKind::Undulate
                | crate::gait::GaitKind::Metachronal
                | crate::gait::GaitKind::Flap
        );
        if elongated {
            c.face = shape_face(pts, c.cx, c.cy, c.face, dt);
        }

        let kick = (c.rms - rms).max(0.0) * 0.004 * sc + stroke * 0.00001 * sc;

        let mut fwd_x = c.rot.cos();
        let mut fwd_y = -c.rot.sin();

        let half = (c.scale * 0.28).clamp(0.035, 0.08);
        let x0 = half;
        let x1 = 1.0 - half;
        let y0 = half * 0.85;
        let y1 = 1.0 - half * 0.85;
        let margin = 0.16;
        let mut wall_x = 0.0;
        let mut wall_y = 0.0;
        if c.x < x0 + margin {
            wall_x += (x0 + margin - c.x) / margin;
        }
        if c.x > x1 - margin {
            wall_x -= (c.x - (x1 - margin)) / margin;
        }
        if c.y < y0 + margin {
            wall_y += (y0 + margin - c.y) / margin;
        }
        if c.y > y1 - margin {
            wall_y -= (c.y - (y1 - margin)) / margin;
        }

        let mut bdes = match g.kind {
            crate::gait::GaitKind::Jet | crate::gait::GaitKind::Hover => {
                0.05 * (ocean_t * 0.020 + c.phase).sin()
            }
            crate::gait::GaitKind::SpinDrift => 0.04 * (ocean_t * 0.018 + c.phase).sin(),
            _ => 0.12 * (ocean_t * 0.048 + c.phase).sin(),
        };
        bdes += (fwd_x * c.fy - fwd_y * c.fx) * 0.25;

        let px = side_g + c.x * inner_w + cam_x;
        let py = top_g + c.y * inner_h + cam_y;
        let rdx = px - mx;
        let rdy = py - my;
        let rd2 = rdx * rdx + rdy * rdy;
        let r_min = 0.16 * w.min(h);
        if rd2 < r_min * r_min && rd2 > 16.0 {
            let rd = rd2.sqrt();
            let nx = rdx / rd;
            let ny = rdy / rd;
            bdes += (fwd_x * ny - fwd_y * nx) * (1.0 - rd / r_min) * 0.35;
        }

        c.bias += dt * 0.55 * (bdes.clamp(-0.7, 0.7) - c.bias);
        let d = crate::gait::drive(g.kind, g, c.phi, c.amp, c.bias, c.speed, dt);
        c.speed = (d.speed + kick * dt).clamp(0.0, 0.048);
        c.rot += d.d_rot;
        c.pose_spin += d.spin_vis;
        c.bell = d.bell;
        c.pose_sway = d.sway * 0.22;
        fwd_x = c.rot.cos();
        fwd_y = -c.rot.sin();

        let wn = (wall_x * wall_x + wall_y * wall_y).sqrt();
        if wn > 0.06 {
            let wx = wall_x / wn;
            let wy = wall_y / wn;
            let inward = fwd_x * wx + fwd_y * wy;
            if inward < 0.30 {
                if c.wall_dir.abs() < 0.5 {
                    let cross = fwd_x * wy - fwd_y * wx;
                    c.wall_dir = if cross.abs() > 0.05 {
                        cross.signum()
                    } else if c.phase > std::f64::consts::PI {
                        1.0
                    } else {
                        -1.0
                    };
                }
                let need = (0.30 - inward).clamp(0.0, 1.0);
                let yaw = 0.42 * need * (0.45 + 0.55 * wn.min(1.0));
                c.rot += c.wall_dir * yaw * dt;
                fwd_x = c.rot.cos();
                fwd_y = -c.rot.sin();
            } else {
                c.wall_dir = 0.0;
            }
            let into = (-fwd_x * wall_x - fwd_y * wall_y).max(0.0);
            if into > 0.0 {
                c.speed *= (1.0 - 0.35 * (into / wn.max(1e-6)).min(1.0)).max(0.55);
            }
        } else {
            c.wall_dir = 0.0;
        }

        let drift = 0.0014 * (ocean_t * 0.07 + c.phase).sin();
        c.vx = c.speed * fwd_x + c.speed * d.slip * fwd_y + drift * 0.18;
        c.vy = c.speed * fwd_y - c.speed * d.slip * fwd_x + g.rise + drift * 0.10;
        if wn > 0.06 {
            let wx = wall_x / wn;
            let wy = wall_y / wn;
            let out = (-c.vx * wx - c.vy * wy).max(0.0);
            c.vx += wx * out;
            c.vy += wy * out;
        }
        c.x += c.vx * dt + c.fx * dt * 0.06;
        c.y += c.vy * dt + c.fy * dt * 0.06;

        if c.x < x0 {
            c.x = x0;
        } else if c.x > x1 {
            c.x = x1;
        }
        if c.y < y0 {
            c.y = y0;
        } else if c.y > y1 {
            c.y = y1;
        }
        c.x = c.x.clamp(x0, x1);
        c.y = c.y.clamp(y0, y1);
        c.ax = (side_g + c.x * inner_w + cam_x) as f32;
        c.ay = (top_g + c.y * inner_h + cam_y) as f32;
        c.radius = (rms * sc).max(34.0) as f32;
    }
}

#[derive(Clone, Copy)]
pub struct LegendGeom {
    pub x0: f32,
    pub y0: f32,
    pub box_w: f32,
    pub box_h: f32,
    pub row_h: f32,
    pub head: f32,
    pub scale: f32,
}

pub fn layout_legend(w: f32, h: f32, n: usize) -> LegendGeom {
    let s = (w.min(h) / 1080.0).clamp(0.82, 1.0) * 1.2;
    let n = n.max(1) as f32;
    let x0 = 10.0 * s;
    let y0 = 10.0 * s;
    let max_h = (h * 0.72 - y0).max(160.0);
    let head = 42.0 * s;
    let row_h = ((max_h - head - 8.0 * s) / n).clamp(20.0 * s, 28.0 * s);
    let box_h = head + n * row_h + 8.0 * s;
    let box_w = (w * 0.18).clamp(220.0 * s, 300.0 * s);
    LegendGeom {
        x0,
        y0,
        box_w,
        box_h,
        row_h,
        head,
        scale: s,
    }
}

pub fn fill_creatures(inst: &[Creature], step: usize, scratches: &mut [Vec<[f32; 2]>]) {
    let n = inst.len().min(scratches.len());
    if n == 0 {
        return;
    }
    let workers = std::thread::available_parallelism()
        .map(|p| p.get().clamp(1, 8))
        .unwrap_or(1)
        .min(n);
    if workers <= 1 || n <= 2 {
        for i in 0..n {
            scratches[i].clear();
            (SPECIES[inst[i].ci].fill as FillFn)(inst[i].t, step, &mut scratches[i]);
        }
        return;
    }
    let chunk = (n + workers - 1) / workers;
    std::thread::scope(|scope| {
        let mut rest_i = &inst[..n];
        let mut rest_s = &mut scratches[..n];
        while !rest_i.is_empty() {
            let take = chunk.min(rest_i.len());
            let (hi, ti) = rest_i.split_at(take);
            let (hs, ts) = rest_s.split_at_mut(take);
            rest_i = ti;
            rest_s = ts;
            scope.spawn(move || {
                for (c, scratch) in hi.iter().zip(hs.iter_mut()) {
                    scratch.clear();
                    (SPECIES[c.ci].fill as FillFn)(c.t, step, scratch);
                }
            });
        }
    });
}

pub struct PointParams {
    pub highlight: usize,
    pub ocean_ps: f32,
    pub legend_ps: f32,
    pub legend: bool,
    pub legend_stride: usize,
}

pub fn build_points(
    inst: &[Creature],
    scratches: &[Vec<[f32; 2]>],
    _ocean_t: f64,
    size: (f32, f32),
    params: PointParams,
    out: &mut Vec<crate::gpu::Instance>,
) {
    out.clear();
    let (w, h) = size;
    let top_g = 12.0f32;
    let bot_g = 12.0f32;
    let side_g = 8.0f32;
    let inner_w = (w - 2.0 * side_g).max(40.0);
    let inner_h = (h - top_g - bot_g).max(40.0);
    let world = inner_w.min(inner_h);
    let layout = layout_legend(w, h, inst.len());
    let stride = params.legend_stride.max(1);
    let msc = (layout.row_h * 0.82) / VIEW as f32;
    let legend_x = layout.x0 + 26.0 * layout.scale;

    for (k, c) in inst.iter().enumerate() {
        let scratch = scratches.get(k).map(|s| s.as_slice()).unwrap_or(&[]);
        let ang = c.rot - c.face + c.pose_sway + c.pose_spin;
        let (ca, sa) = (ang.cos() as f32, ang.sin() as f32);
        let mut breathe = c.bell * (1.0 + 0.04 * c.pulse);
        if k == params.highlight {
            breathe += 0.04;
        }
        let sc = (c.scale * breathe) as f32 * world / VIEW as f32;
        let px = side_g + c.x as f32 * inner_w;
        let py = top_g + c.y as f32 * inner_h;
        let alpha = if k == params.highlight { 0.46 } else { 0.30 };
        let rgb = if k == params.highlight {
            [1.0, 1.0, 1.0]
        } else {
            [0.96, 0.98, 1.0]
        };
        let cx = c.cx as f32;
        let cy = c.cy as f32;
        for p in scratch.iter() {
            let x = p[0];
            let y = p[1];
            if x < -40.0 || x > VIEW as f32 + 40.0 || y < -40.0 || y > VIEW as f32 + 40.0 {
                continue;
            }
            let dx = x - cx;
            let dy = y - cy;
            out.push(crate::gpu::Instance {
                pos: [px + (dx * ca - dy * sa) * sc, py - (dx * sa + dy * ca) * sc],
                rgba: [rgb[0], rgb[1], rgb[2], alpha],
                size: params.ocean_ps,
                _pad: 0.0,
            });
        }
        if !params.legend {
            continue;
        }
        let ly = layout.y0 + layout.head + (k as f32 + 0.5) * layout.row_h;
        let la = if k == params.highlight { 0.95 } else { 0.62 };
        for (i, p) in scratch.iter().enumerate() {
            if i % stride != 0 {
                continue;
            }
            let x = p[0];
            let y = p[1];
            if x < -40.0 || x > VIEW as f32 + 40.0 || y < -40.0 || y > VIEW as f32 + 40.0 {
                continue;
            }
            let dx = x - cx;
            let dy = y - cy;
            out.push(crate::gpu::Instance {
                pos: [legend_x + (dx * ca - dy * sa) * msc, ly - (dx * sa + dy * ca) * msc],
                rgba: [1.0, 1.0, 1.0, la],
                size: params.legend_ps,
                _pad: 0.0,
            });
        }
    }
}
