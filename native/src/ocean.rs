use std::sync::atomic::{AtomicUsize, Ordering};

use crate::formulas::{FillFn, SPECIES, VIEW};
use crate::life::{species_life, LifeParams, LIFE};

fn cpu_workers() -> usize {
    std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(1)
        .max(1)
}

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
    pub evade_dir: f64,
    pub evade_t: f64,
}

fn shuffled(n: usize, rand: &mut impl FnMut() -> f64) -> Vec<usize> {
    let mut a: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        let j = (rand() * (i + 1) as f64) as usize;
        a.swap(i, j);
    }
    a
}

fn even_slots(count: usize, rand: &mut impl FnMut() -> f64) -> Vec<(f64, f64)> {
    let count = count.max(1);
    let cols = ((count as f64 * 1.05).sqrt().ceil() as usize).max(1);
    let rows = (count + cols - 1) / cols;
    let mut cells: Vec<(usize, usize)> = (0..rows)
        .flat_map(|r| (0..cols).map(move |c| (c, r)))
        .collect();
    while cells.len() > count {
        let j = (rand() * cells.len() as f64) as usize % cells.len();
        cells.swap_remove(j);
    }
    let mut slots: Vec<(f64, f64)> = Vec::with_capacity(count);
    for (gx, gy) in cells {
        let mut best = (0.5, 0.5);
        let mut best_d = -1.0;
        for _ in 0..8 {
            let jx = (rand() - 0.5) * 0.14;
            let jy = (rand() - 0.5) * 0.14;
            let x = ((gx as f64 + 0.5 + jx) / cols as f64 * 0.80 + 0.10).clamp(0.08, 0.92);
            let y = ((gy as f64 + 0.5 + jy) / rows as f64 * 0.76 + 0.12).clamp(0.10, 0.90);
            let d = if slots.is_empty() {
                1.0
            } else {
                slots
                    .iter()
                    .map(|p| (p.0 - x).hypot(p.1 - y))
                    .fold(f64::MAX, f64::min)
            };
            if d > best_d {
                best_d = d;
                best = (x, y);
            }
        }
        slots.push(best);
    }
    slots
}

fn gyre_heading(x: f64, y: f64) -> f64 {
    let gx = y - 0.5;
    let gy = 0.5 - x;
    (-gy).atan2(gx)
}

pub fn spawn(seed: u32, count: usize) -> Vec<Creature> {
    spawn_with(seed, count)
}

pub fn spawn_with(seed: u32, count: usize) -> Vec<Creature> {
    let mut rand = crate::life::mulberry32(seed);
    let n_c = SPECIES.len();
    let count = count.clamp(1, n_c);
    let order = shuffled(n_c, &mut rand);
    let slots = even_slots(count, &mut rand);
    let mut inst = Vec::with_capacity(count);
    for i in 0..count {
        let (x, y) = slots[i];
        let ang = gyre_heading(x, y) + (rand() - 0.5) * 1.05;
        let ci = order[i];
        let g = crate::gait::gait(ci);
        let tempo = 0.72 + rand() * 0.28;
        let omega = g.hz * std::f64::consts::TAU;
        let speed = g.cruise;
        inst.push(Creature {
            ci,
            x,
            y,
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
            face: species_face(ci, 0.0),
            evade_dir: 0.0,
            evade_t: 0.0,
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

/// 绘制用边距。指示圈必须用同一套，否则窗口模式下圈和身体会对不齐。
fn draw_frame(w: f32, h: f32) -> (f32, f32, f32, f32, f32) {
    let side = 8.0;
    let top = 12.0;
    let bot = 12.0;
    let inner_w = (w - 2.0 * side).max(40.0);
    let inner_h = (h - top - bot).max(40.0);
    (side, top, inner_w, inner_h, inner_w.min(inner_h))
}

fn vis_angle(c: &Creature) -> f64 {
    let kind = crate::gait::gait(c.ci).kind;
    let spin = if matches!(
        kind,
        crate::gait::GaitKind::SpinDrift | crate::gait::GaitKind::Helix
    ) {
        c.pose_spin
    } else {
        0.0
    };
    let sway = if matches!(
        kind,
        crate::gait::GaitKind::Undulate
            | crate::gait::GaitKind::Metachronal
            | crate::gait::GaitKind::Flap
            | crate::gait::GaitKind::Jet
    ) {
        c.pose_sway
    } else {
        0.0
    };
    c.rot - c.face + sway + spin
}

/// 屏幕上点云的包围盒中心与外接半径，给指示圈用。
fn visual_ring(
    pts: &[[f32; 2]],
    cx: f64,
    cy: f64,
    ang: f64,
    sc: f64,
    px: f64,
    py: f64,
) -> (f32, f32, f32) {
    if pts.len() < 8 {
        return (px as f32, py as f32, 16.0);
    }
    let (ca, sa) = (ang.cos(), ang.sin());
    let stride = (pts.len() / 480).max(1);
    let mut minx = f64::MAX;
    let mut maxx = f64::MIN;
    let mut miny = f64::MAX;
    let mut maxy = f64::MIN;
    let mut n = 0u32;
    for p in pts.iter().step_by(stride) {
        let x = p[0] as f64;
        let y = p[1] as f64;
        if x < -40.0 || x > VIEW + 40.0 || y < -40.0 || y > VIEW + 40.0 {
            continue;
        }
        let dx = x - cx;
        let dy = y - cy;
        let sx = px + (dx * ca - dy * sa) * sc;
        let sy = py - (dx * sa + dy * ca) * sc;
        minx = minx.min(sx);
        maxx = maxx.max(sx);
        miny = miny.min(sy);
        maxy = maxy.max(sy);
        n += 1;
    }
    if n < 4 {
        return (px as f32, py as f32, 16.0);
    }
    let ax = (minx + maxx) * 0.5;
    let ay = (miny + maxy) * 0.5;
    let r = ((maxx - minx) * 0.5).hypot((maxy - miny) * 0.5) + 8.0;
    (ax as f32, ay as f32, r.max(10.0) as f32)
}

/// 各物种公式空间里的推进轴（弧度，t=0）。必须让 `HeadingKind` 的头朝向航向。
/// 水母=伞盖朝前，栉水母=口端朝前。加新种时先定 heading，再标定 face。
const SPECIES_FACE: [f64; 17] = [
    -1.583, // fucan
    2.221, // youyan
    1.540, // jichong
    0.125, // jelly  伞盖=中线最下方，锁到航向
    1.563, // nebula
    2.473, // lantern  伞盖随公式相位转
    1.977, // feather
    1.962, // tentacle
    0.000, // flower6
    0.000, // wheel
    2.287, // spiral
    1.305, // comb  口端=中线最下方，锁到航向
    1.783, // saweel
    0.000, // star8
    2.390, // shrimp
    1.885, // vortex
    1.666, // angel  身体尖端朝前，不是翅膀
];

/// 公式整体自旋（rad / 公式 t）。花水母的伞盖会随 t 转，固定 face 会周期性倒游。
const SPECIES_FACE_RATE: [f64; 17] = [
    0.0, 0.0, 0.0, 0.0, 0.0, 0.114, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
];

const _: () = assert!(SPECIES_FACE.len() == 17);
const _: () = assert!(SPECIES_FACE_RATE.len() == 17);

fn species_face(ci: usize, t: f64) -> f64 {
    wrap_pi(SPECIES_FACE[ci] + SPECIES_FACE_RATE[ci] * t)
}

fn paced_gait(g: crate::gait::Gait, pace: f64) -> crate::gait::Gait {
    let mut g = g;
    let p = pace.clamp(0.70, 2.80);
    g.cruise *= p;
    g.pulse *= p;
    g
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
    pointer: Option<(f64, f64)>,
    size: (f32, f32),
    saver: bool,
) {
    integrate_with(inst, scratches, ocean_t, dt, pointer, size, saver, &LIFE);
}

pub fn integrate_with(
    inst: &mut [Creature],
    scratches: &[Vec<[f32; 2]>],
    ocean_t: f64,
    dt: f64,
    pointer: Option<(f64, f64)>,
    size: (f32, f32),
    saver: bool,
    life: &LifeParams,
) {
    let (w, h) = (size.0 as f64, size.1 as f64);
    let top_g = if saver { 12.0 } else { 64.0 };
    let bot_g = if saver { 12.0 } else { 92.0 };
    let side_g = 8.0;
    let inner_w = (w - 2.0 * side_g).max(40.0);
    let inner_h = (h - top_g - bot_g).max(40.0);
    let world = inner_w.min(inner_h);
    let (cam_x, cam_y, mx, my, pointer_on) = if let Some((px, py)) = pointer {
        ((px - 0.5) * 40.0, (py - 0.5) * 22.0, px * w, py * h, true)
    } else {
        (0.0, 0.0, 0.0, 0.0, false)
    };
    let dt = dt.max(1.0 / 240.0);
    let n = inst.len();
    let (ds, dtop, diw, dih, dworld) = draw_frame(size.0, size.1);

    for c in inst.iter_mut() {
        c.fx = 0.0;
        c.fy = 0.0;
    }
    let mut nd = vec![f64::MAX; n];
    let mut nnx = vec![0.0; n];
    let mut nny = vec![0.0; n];
    let mut same_nd = vec![f64::MAX; n];
    let mut orient_hx = vec![0.0; n];
    let mut orient_hy = vec![0.0; n];
    let mut orient_n = vec![0.0; n];
    let mut attract_dx = vec![0.0; n];
    let mut attract_dy = vec![0.0; n];
    let mut attract_n = vec![0.0; n];
    let mut in_repulse = vec![false; n];
    let mut align_mean = vec![f64::NAN; n];
    for a in 0..n {
        let ka = species_life(inst[a].ci, life);
        let ra = inst[a].scale * life.body * ka.space;
        for b in (a + 1)..n {
            let ddx = inst[a].x - inst[b].x;
            let ddy = inst[a].y - inst[b].y;
            let dd = (ddx * ddx + ddy * ddy).sqrt();
            if dd < 1e-5 {
                inst[a].x += 0.003;
                inst[b].x -= 0.003;
                continue;
            }
            let kb = species_life(inst[b].ci, life);
            let rb = inst[b].scale * life.body * kb.space;
            let contact = (ra + rb) * life.near;
            let far = contact * life.far;
            let nx = ddx / dd;
            let ny = ddy / dd;
            if dd < far {
                let u = (1.0 - dd / far).clamp(0.0, 1.0);
                let shy = 0.5 * (ka.shy + kb.shy);
                let w = life.far_w * u * u * shy;
                inst[a].fx += nx * w;
                inst[a].fy += ny * w;
                inst[b].fx -= nx * w;
                inst[b].fy -= ny * w;
            }
            if dd < contact {
                let u = (1.0 - dd / contact).clamp(0.0, 1.0);
                let w = life.push * u * u;
                inst[a].fx += nx * w;
                inst[a].fy += ny * w;
                inst[b].fx -= nx * w;
                inst[b].fy -= ny * w;
                // 重叠必须把身体推开；只拧航向会穿模、看起来像横移叠在一起。
                let sep = (contact - dd) * 0.42;
                inst[a].x += nx * sep;
                inst[a].y += ny * sep;
                inst[b].x -= nx * sep;
                inst[b].y -= ny * sep;
            }
            if dd < nd[a] {
                nd[a] = dd;
                nnx[a] = nx;
                nny[a] = ny;
            }
            if dd < nd[b] {
                nd[b] = dd;
                nnx[b] = -nx;
                nny[b] = -ny;
            }
            if inst[a].ci == inst[b].ci {
                if dd < same_nd[a] {
                    same_nd[a] = dd;
                }
                if dd < same_nd[b] {
                    same_nd[b] = dd;
                }
                if dd < contact {
                    in_repulse[a] = true;
                    in_repulse[b] = true;
                } else {
                    let za = ka.zone.clamp(1.60, 2.80);
                    let zb = kb.zone.clamp(1.60, 2.80);
                    let oh_a = contact * za;
                    let oh_b = contact * zb;
                    let ah_a = contact * (za + 1.15);
                    let ah_b = contact * (zb + 1.15);
                    let fwd_ax = inst[a].rot.cos();
                    let fwd_ay = -inst[a].rot.sin();
                    let fwd_bx = inst[b].rot.cos();
                    let fwd_by = -inst[b].rot.sin();
                    // 盲区：背后的邻居不进定向平均，避免环流里对向互相抵消。
                    let see_b = fwd_ax * (-nx) + fwd_ay * (-ny) > -0.55;
                    let see_a = fwd_bx * nx + fwd_by * ny > -0.55;
                    if dd < oh_a {
                        if see_b {
                            let w = (1.0 - dd / oh_a).clamp(0.0, 1.0).powi(2);
                            orient_hx[a] += inst[b].rot.cos() * w;
                            orient_hy[a] += inst[b].rot.sin() * w;
                            orient_n[a] += w;
                        }
                    } else if dd < ah_a {
                        attract_dx[a] -= nx;
                        attract_dy[a] -= ny;
                        attract_n[a] += 1.0;
                    }
                    if dd < oh_b {
                        if see_a {
                            let w = (1.0 - dd / oh_b).clamp(0.0, 1.0).powi(2);
                            orient_hx[b] += inst[a].rot.cos() * w;
                            orient_hy[b] += inst[a].rot.sin() * w;
                            orient_n[b] += w;
                        }
                    } else if dd < ah_b {
                        attract_dx[b] += nx;
                        attract_dy[b] += ny;
                        attract_n[b] += 1.0;
                    }
                }
            }
        }
    }

    for (i, c) in inst.iter_mut().enumerate() {
        let pts = scratches.get(i).map(|s| s.as_slice()).unwrap_or(&[]);
        let g = crate::gait::gait(c.ci);
        let (raw_cx, raw_cy, raw_rms) = shape_stats(pts);
        let (cx, cy, rms) = if matches!(
            g.kind,
            crate::gait::GaitKind::Jet | crate::gait::GaitKind::SpinDrift
        ) {
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
        c.face = species_face(c.ci, c.t);

        let kick = (c.rms - rms).max(0.0) * 0.004 * sc + stroke * 0.00001 * sc;

        let mut fwd_x = c.rot.cos();
        let mut fwd_y = -c.rot.sin();

        let half = (c.scale * 0.28).clamp(0.035, 0.08);
        let x0 = half;
        let x1 = 1.0 - half;
        let y0 = half * 0.85;
        let y1 = 1.0 - half * 0.85;
        let margin = 0.22;
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

        let kl = species_life(c.ci, life);
        let wfreq = match g.kind {
            crate::gait::GaitKind::Jet | crate::gait::GaitKind::Hover => 0.020,
            crate::gait::GaitKind::SpinDrift => 0.018,
            _ => 0.048,
        };
        let rot0 = c.rot;
        let mut bdes = kl.wander * (ocean_t * wfreq + c.phase).sin();
        let steer_f = match g.kind {
            crate::gait::GaitKind::SpinDrift | crate::gait::GaitKind::Helix => 0.05,
            crate::gait::GaitKind::Hover => 0.18,
            crate::gait::GaitKind::Jet => 0.32,
            _ => 0.20,
        };
        bdes += (fwd_x * c.fy - fwd_y * c.fx) * steer_f;
        let gl = ((c.x - 0.5).hypot(c.y - 0.5)).max(0.08);
        let gnx = (c.y - 0.5) / gl;
        let gny = (0.5 - c.x) / gl;
        let nnd_hint = nd.get(i).copied().unwrap_or(f64::MAX);
        let g_w = if nnd_hint < 0.16 { 0.22 } else { 1.0 };
        let g_scale = match g.kind {
            crate::gait::GaitKind::SpinDrift => 0.12,
            crate::gait::GaitKind::Hover | crate::gait::GaitKind::Jet => 0.18,
            crate::gait::GaitKind::Ciliary => 0.28,
            crate::gait::GaitKind::Helix => 0.30,
            _ => 1.0,
        };
        let flocking = orient_n[i] >= 1.0;
        if !flocking {
            bdes += (fwd_x * gny - fwd_y * gnx) * life.gyre * g_w * g_scale;
            // 环流直接拧航向：同种定向时关掉，否则平均航向被环流抵消。
            c.rot += (fwd_x * gny - fwd_y * gnx) * life.gyre * 8.0 * g_scale * dt;
        }

        if pointer_on {
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
        }

        let my_r = c.scale * life.body * kl.space;
        // Couzin：互斥优先；否则定向圈内跟所有同种邻居航向平均；再否则圈外弱吸引。
        // slip = 对齐增益，yaw 只在后面让路。
        if crate::life::heading_is_trainable(c.ci) && !in_repulse[i] {
            // slip 只调对齐强弱；增益封顶，避免顶满后振荡、把学校甩散。
            let gain = (0.20 + 0.62 * (kl.slip / 1.10).clamp(0.0, 1.0)).min(0.82);
            if orient_n[i] >= 1e-3 {
                let mean = orient_hy[i].atan2(orient_hx[i]);
                align_mean[i] = mean;
                let drot = wrap_pi(mean - rot0);
                c.rot += drot * gain * dt;
            } else if attract_n[i] >= 1.0 {
                let ax = attract_dx[i] / attract_n[i];
                let ay = attract_dy[i] / attract_n[i];
                let an = ax.hypot(ay).max(1e-9);
                let cross = fwd_x * (ay / an) - fwd_y * (ax / an);
                c.rot += cross * 0.22 * dt;
            }
        }

        c.bias += dt * 0.55 * (bdes.clamp(-0.7, 0.7) - c.bias);
        let d = crate::gait::drive(
            g.kind,
            paced_gait(g, kl.pace),
            c.phi,
            c.amp,
            c.bias,
            c.speed,
            dt,
        );
        let cap = (0.048 * kl.pace).clamp(0.048, 0.10);
        c.speed = (d.speed + kick * dt).clamp(0.0, cap);
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
            // 只在这里选定离开方向；真正转角放在 max_yaw 夹紧之后，避免被游荡吃掉。
            if inward < 0.78 {
                if c.wall_dir.abs() < 0.5 {
                    // d(inward)/d(rot)：朝开阔处的短弧，不要先拧进墙再绕一圈。
                    let din = fwd_y * wx - fwd_x * wy;
                    c.wall_dir = if din.abs() > 0.05 {
                        din.signum()
                    } else if c.phase > std::f64::consts::PI {
                        1.0
                    } else {
                        -1.0
                    };
                }
            } else {
                c.wall_dir = 0.0;
            }
            let into = (-fwd_x * wall_x - fwd_y * wall_y).max(0.0);
            if into > 0.0 {
                let frac = (into / wn.max(1e-6)).min(1.0);
                c.speed *= (1.0 - 1.8 * frac * dt).max(0.0);
            }
        } else {
            c.wall_dir = 0.0;
        }

        let sense = match g.kind {
            crate::gait::GaitKind::Jet | crate::gait::GaitKind::Hover => {
                (my_r * 3.60).clamp(0.085, 0.18)
            }
            crate::gait::GaitKind::SpinDrift => (my_r * 2.30).clamp(0.060, 0.12),
            _ => (my_r * (2.35 + 1.05 * kl.shy)).clamp(0.070, 0.16),
        };
        let nnd = nd[i];
        let contact_est = my_r * life.near;
        let overlapping = nnd.is_finite() && nnd < contact_est;
        let mut on_course = overlapping;
        if nnd < 0.24 {
            let nx = nnx[i];
            let ny = nny[i];
            let to_x = -nx;
            let to_y = -ny;
            let closing = fwd_x * to_x + fwd_y * to_y;
            let miss = (fwd_x * to_y - fwd_y * to_x).abs();
            let impact = miss * nnd;
            let hit_r = my_r * (0.68 + 0.42 * kl.space);
            let loom = closing > 0.42 && impact < hit_r * 1.2;
            let range = if loom {
                (sense * 1.9).clamp(0.10, 0.22)
            } else {
                sense
            };
            let prox = (1.0 - nnd / range.max(1e-6)).clamp(0.0, 1.0);
            on_course = overlapping
                || (nnd < range && closing > 0.12 && impact < hit_r * 1.35)
                || (loom && nnd < range);
            if on_course {
                if c.evade_t <= 0.0 || c.evade_dir.abs() < 0.5 {
                    let cross = fwd_x * ny - fwd_y * nx;
                    c.evade_dir = if cross.abs() > 0.06 {
                        cross.signum()
                    } else if c.phase > std::f64::consts::PI {
                        1.0
                    } else {
                        -1.0
                    };
                }
                let urgency = if overlapping {
                    (0.55 + 0.45 * prox).max(0.40)
                } else {
                    (1.0 - (impact / hit_r.max(1e-6)).clamp(0.0, 1.0))
                        * closing.clamp(0.0, 1.0)
                        * (0.45 + 0.55 * prox)
                };
                c.evade_t = (0.18 + 0.28 * urgency).max(c.evade_t);
                if overlapping || impact < hit_r * 0.88 {
                    let yaw = kl.yaw * urgency.max(if overlapping { 0.40 } else { 0.12 });
                    c.rot += c.evade_dir * yaw * dt;
                }
                c.speed *= (1.0 - kl.brake * urgency * 0.35).max(0.62);
            } else if closing > 0.14 && nnd < sense * 0.80 && c.evade_dir.abs() < 0.5 {
                let cross = fwd_x * ny - fwd_y * nx;
                c.evade_dir = if cross.abs() > 0.04 {
                    cross.signum()
                } else {
                    1.0
                };
            }
        }
        c.evade_t = (c.evade_t - dt).max(0.0);
        if c.evade_t <= 0.0 {
            c.evade_dir = 0.0;
        }

        let mut max_yaw: f64 = match g.kind {
            crate::gait::GaitKind::SpinDrift => 0.18,
            crate::gait::GaitKind::Hover => 0.32,
            crate::gait::GaitKind::Jet => 0.55,
            crate::gait::GaitKind::Helix => 0.55,
            crate::gait::GaitKind::Ciliary => 0.62,
            _ => 0.85,
        };
        if on_course {
            max_yaw = max_yaw.max(1.60);
        }
        if align_mean[i].is_finite() {
            max_yaw = max_yaw.max(1.45);
        }
        // 贴边时允许更快转弯，避免撞墙后沿轴滑出直角折。
        if wn > 0.10 {
            max_yaw = max_yaw.max(0.88).min(1.20);
        }
        let dheading = wrap_pi(c.rot - rot0);
        c.rot = rot0 + dheading.clamp(-max_yaw * dt, max_yaw * dt);
        if align_mean[i].is_finite() && !in_repulse[i] && c.evade_t <= 0.0 && wn < 0.08 {
            let d = wrap_pi(align_mean[i] - c.rot);
            c.rot += d.clamp(-0.95 * dt, 0.95 * dt);
        }
        fwd_x = c.rot.cos();
        fwd_y = -c.rot.sin();
        if wn > 0.06 && c.wall_dir.abs() > 0.5 {
            let wx = wall_x / wn;
            let wy = wall_y / wn;
            let inward = fwd_x * wx + fwd_y * wy;
            if inward < 0.78 {
                c.rot += c.wall_dir * (0.82 - inward).clamp(0.0, 1.0) * 0.85 * dt;
                fwd_x = c.rot.cos();
                fwd_y = -c.rot.sin();
            } else {
                c.wall_dir = 0.0;
            }
        }
        if g.kind == crate::gait::GaitKind::Jet {
            // 触手只允许很小的随流滞后；大滞后会看起来像伞盖朝前、身体横着挪。
            let stream = (c.speed / (g.cruise * kl.pace).max(1e-6)).clamp(0.45, 1.6);
            let tau = 0.12 / stream;
            let old_vis = rot0 + c.pose_sway;
            let a = 1.0 - (-dt / tau).exp();
            let new_vis = old_vis + wrap_pi(c.rot - old_vis) * a;
            c.pose_sway = wrap_pi(new_vis - c.rot).clamp(-0.08, 0.08);
        }

        if on_course {
            c.speed *= 0.92;
        }
        let drift = 0.0014 * (ocean_t * 0.07 + c.phase).sin();
        // 前进只沿航向：让路靠转向，不靠横移。重叠时另加径向推开。
        c.vx = c.speed * fwd_x;
        c.vy = c.speed * fwd_y;
        if g.kind == crate::gait::GaitKind::Hover {
            c.vy += g.rise;
        }
        if matches!(
            g.kind,
            crate::gait::GaitKind::Hover | crate::gait::GaitKind::SpinDrift
        ) {
            c.vx += drift * 0.18;
            c.vy += drift * 0.10;
        }
        if wn > 0.06 {
            let wx = wall_x / wn;
            let wy = wall_y / wn;
            let out = (-c.vx * wx - c.vy * wy).max(0.0);
            c.vx += wx * out;
            c.vy += wy * out;
        } else if !matches!(
            g.kind,
            crate::gait::GaitKind::SpinDrift | crate::gait::GaitKind::Hover
        ) {
            // 贴边时不要再投影回航向：外向被消掉后投影会把速度归零，变成沿墙停/滑。
            let along = (c.vx * fwd_x + c.vy * fwd_y).max(0.0);
            c.vx = along * fwd_x;
            c.vy = along * fwd_y;
        }
        c.x += c.vx * dt;
        c.y += c.vy * dt;

        let hit_x = c.x < x0 || c.x > x1;
        let hit_y = c.y < y0 || c.y > y1;
        c.x = c.x.clamp(x0, x1);
        c.y = c.y.clamp(y0, y1);
        if hit_x || hit_y {
            let out_x: f64 = if c.x <= x0 + 1e-12 {
                1.0
            } else if c.x >= x1 - 1e-12 {
                -1.0
            } else {
                0.0
            };
            let out_y: f64 = if c.y <= y0 + 1e-12 {
                1.0
            } else if c.y >= y1 - 1e-12 {
                -1.0
            } else {
                0.0
            };
            let on = (out_x * out_x + out_y * out_y).sqrt().max(1e-9_f64);
            let ox = out_x / on;
            let oy = out_y / on;
            // 撞上就朝开阔处转，不要改贴边巡游（那会画出直角折）。
            let target = (-oy).atan2(ox);
            let d = wrap_pi(target - c.rot);
            c.rot += d.clamp(-0.10, 0.10);
            c.wall_dir = 0.0;
            c.x += ox * 0.006;
            c.y += oy * 0.006;
            c.x = c.x.clamp(x0, x1);
            c.y = c.y.clamp(y0, y1);
            c.speed = (c.speed * (1.0 - 1.2 * dt)).max(0.010);
            let (ca, sa) = (c.rot.cos(), c.rot.sin());
            c.vx = c.speed * ca;
            c.vy = -c.speed * sa;
            let out_comp = c.vx * ox + c.vy * oy;
            if out_comp < 0.006 {
                c.vx += ox * (0.010 - out_comp);
                c.vy += oy * (0.010 - out_comp);
            }
        }
        let breathe = c.bell * (1.0 + 0.04 * c.pulse);
        let dsc = c.scale * breathe * dworld as f64 / VIEW;
        let px = ds as f64 + c.x * diw as f64;
        let py = dtop as f64 + c.y * dih as f64;
        let (ax, ay, r) = visual_ring(pts, c.cx, c.cy, vis_angle(c), dsc, px, py);
        c.ax = ax;
        c.ay = ay;
        c.radius = r;
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
    let workers = cpu_workers().min(n);
    if workers <= 1 {
        for i in 0..n {
            scratches[i].clear();
            (SPECIES[inst[i].ci].fill as FillFn)(inst[i].t, step, &mut scratches[i]);
        }
        return;
    }
    // 按完成一个领一个：浮蚕/星云比轮虫重几倍，切块会把重活堆在同一核。
    let next = AtomicUsize::new(0);
    let base = scratches.as_mut_ptr() as usize;
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= n {
                    break;
                }
                // Safety: i 由原子计数唯一分配，每个 scratch 只被一个线程写。
                let scratch = unsafe { &mut *(base as *mut Vec<[f32; 2]>).add(i) };
                scratch.clear();
                (SPECIES[inst[i].ci].fill as FillFn)(inst[i].t, step, scratch);
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
    let (side_g, top_g, inner_w, inner_h, world) = draw_frame(w, h);
    let layout = layout_legend(w, h, inst.len());
    let stride = params.legend_stride.max(1);
    let msc = (layout.row_h * 0.82) / VIEW as f32;
    let legend_x = layout.x0 + 26.0 * layout.scale;
    let n = inst.len();
    let workers = cpu_workers().min(n.max(1));
    if workers <= 1 || n <= 2 {
        for k in 0..n {
            emit_creature_points(
                k,
                &inst[k],
                scratches.get(k).map(|s| s.as_slice()).unwrap_or(&[]),
                world,
                inner_w,
                inner_h,
                side_g,
                top_g,
                legend_x,
                msc,
                stride,
                &layout,
                &params,
                out,
            );
        }
        return;
    }
    let next = AtomicUsize::new(0);
    let parts: Vec<Vec<crate::gpu::Instance>> = std::thread::scope(|scope| {
        let mut hs = Vec::with_capacity(workers);
        for _ in 0..workers {
            hs.push(scope.spawn(|| {
                let mut local = Vec::new();
                loop {
                    let k = next.fetch_add(1, Ordering::Relaxed);
                    if k >= n {
                        break;
                    }
                    emit_creature_points(
                        k,
                        &inst[k],
                        scratches.get(k).map(|s| s.as_slice()).unwrap_or(&[]),
                        world,
                        inner_w,
                        inner_h,
                        side_g,
                        top_g,
                        legend_x,
                        msc,
                        stride,
                        &layout,
                        &params,
                        &mut local,
                    );
                }
                local
            }));
        }
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    for part in parts {
        out.extend(part);
    }
}

fn emit_creature_points(
    k: usize,
    c: &Creature,
    scratch: &[[f32; 2]],
    world: f32,
    inner_w: f32,
    inner_h: f32,
    side_g: f32,
    top_g: f32,
    legend_x: f32,
    msc: f32,
    stride: usize,
    layout: &LegendGeom,
    params: &PointParams,
    out: &mut Vec<crate::gpu::Instance>,
) {
    let ang = vis_angle(c);
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
        return;
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

#[cfg(test)]
pub fn simulate_school(seed: u32, count: usize, seconds: f64, life: &LifeParams) -> crate::life::SchoolStats {
    simulate_school_as(seed, count, seconds, life, None)
}

/// 同种学校：8 只同一数字生物，对照该种自己的指标。
#[cfg(test)]
pub fn simulate_species(
    seed: u32,
    ci: usize,
    count: usize,
    seconds: f64,
    life: &LifeParams,
) -> crate::life::SchoolStats {
    simulate_school_as(seed, count, seconds, life, Some(ci.min(SPECIES.len() - 1)))
}

#[cfg(test)]
fn simulate_school_as(
    seed: u32,
    count: usize,
    seconds: f64,
    life: &LifeParams,
    as_ci: Option<usize>,
) -> crate::life::SchoolStats {
    let mut inst = spawn_with(seed, count);
    if let Some(ci) = as_ci {
        let g = crate::gait::gait(ci);
        let pace = species_life(ci, life).pace;
        let omega = g.hz * std::f64::consts::TAU;
        let speed = g.cruise * pace;
        let face = species_face(ci, 0.0);
        for c in &mut inst {
            c.ci = ci;
            c.omega = omega;
            c.speed = speed;
            let ang = c.rot;
            c.vx = speed * ang.cos();
            c.vy = -speed * ang.sin();
            c.face = face;
        }
    }
    let scratches = vec![Vec::new(); inst.len()];
    let dt = 1.0 / 60.0;
    let nsteps = (seconds / dt) as usize;
    // 同种学校：前 60% 只用来进稳态，极化/间距只报后 40%。
    let warmup = if as_ci.is_some() {
        let settle = ((seconds * 0.50) / dt) as usize;
        let keep = ((3.0 / dt) as usize).max(1);
        settle.min(nsteps.saturating_sub(keep))
    } else {
        ((1.2 / dt) as usize).min(nsteps / 4)
    };
    let n = inst.len();
    let mut nn_acc = 0.0;
    let mut min_nn = f64::MAX;
    let mut closest = f64::MAX;
    let mut overlap_n = 0.0;
    let mut samples = 0.0;
    let mut align_acc = 0.0;
    let mut flips = 0.0;
    let mut prev_yaw: Vec<f64> = vec![0.0; n];
    let mut prev_rot: Vec<f64> = inst.iter().map(|c| c.rot).collect();
    let mut cells = [0u32; 16];
    let mut corner = 0.0;
    let mut speed_acc = 0.0;
    let mut graze_n = 0.0;
    let mut gyre_acc = 0.0;
    let mut cruise_acc = 0.0;
    let mut evade_n = 0.0;
    let mut nnd_bl_acc = 0.0;
    let mut min_nnd_bl = f64::MAX;
    let mut yaw_samples: Vec<f64> = Vec::new();
    let mut sharp_n = 0.0;
    let mut sharp_d = 0.0;
    let mut polar_acc = 0.0;
    let mut polar_n = 0.0;
    let mut prev_xy: Vec<(f64, f64)> = inst.iter().map(|c| (c.x, c.y)).collect();
    let mut prev_head = vec![0.0; n];
    let mut have_head = vec![false; n];
    let mut kind_nnd = [0.0; 17];
    let mut kind_nnd_n = [0.0; 17];
    let mut kind_spd = [0.0; 17];
    let mut kind_spd_n = [0.0; 17];
    let mut kind_yaw: [Vec<f64>; 17] = std::array::from_fn(|_| Vec::new());
    let mut kind_polar = [0.0; 17];
    let mut kind_polar_n = [0.0; 17];
    let mut speed_bl_acc = 0.0;
    let mut speed_bl_n = 0.0;
    const YAW_STRIDE: usize = 12; // 5 Hz @ 60 fps
    for step in 0..nsteps {
        advance_morph(&mut inst, dt);
        integrate_with(
            &mut inst,
            &scratches,
            step as f64 * dt,
            dt,
            None,
            (1920.0, 1080.0),
            true,
            life,
        );
        if step < warmup {
            for (i, c) in inst.iter().enumerate() {
                prev_rot[i] = c.rot;
                prev_xy[i] = (c.x, c.y);
                have_head[i] = false;
            }
            continue;
        }
        samples += 1.0;
        let mut frame_min = f64::MAX;
        let mut frame_min_bl = f64::MAX;
        let stride_frame = (step - warmup) % YAW_STRIDE == 0;
        let mut pu_x = 0.0;
        let mut pu_y = 0.0;
        let mut pu_n = 0.0;
        let mut kux = [0.0; 17];
        let mut kuy = [0.0; 17];
        let mut kun = [0.0; 17];
        for i in 0..n {
            let mut nn = f64::MAX;
            let mut nn_bl = f64::MAX;
            let bli = (inst[i].scale * life.body * 2.0).clamp(0.018, 0.16);
            for j in 0..n {
                if i == j {
                    continue;
                }
                let d = (inst[i].x - inst[j].x).hypot(inst[i].y - inst[j].y);
                nn = nn.min(d);
                let blj = (inst[j].scale * life.body * 2.0).clamp(0.018, 0.16);
                nn_bl = nn_bl.min(d / ((bli + blj) * 0.5));
                if j > i {
                    closest = closest.min(d);
                    let ka = species_life(inst[i].ci, life);
                    let kb = species_life(inst[j].ci, life);
                    let contact = (inst[i].scale * life.body * ka.space
                        + inst[j].scale * life.body * kb.space)
                        * 0.85;
                    if d < contact {
                        overlap_n += 1.0;
                    }
                    if d >= 0.055 && d <= 0.125 {
                        graze_n += 1.0;
                    }
                }
            }
            frame_min = frame_min.min(nn);
            frame_min_bl = frame_min_bl.min(nn_bl);
            nn_acc += nn;
            nnd_bl_acc += nn_bl;
            let ki = inst[i].ci.min(16);
            kind_nnd[ki] += nn_bl;
            kind_nnd_n[ki] += 1.0;
            let fx = inst[i].rot.cos();
            let fy = -inst[i].rot.sin();
            let v = (inst[i].vx * inst[i].vx + inst[i].vy * inst[i].vy).sqrt();
            if v > 1e-6 {
                align_acc += (fx * inst[i].vx + fy * inst[i].vy) / v;
            }
            let yaw = wrap_pi(inst[i].rot - prev_rot[i]) / dt;
            if stride_frame {
                let (px, py) = prev_xy[i];
                let dx = inst[i].x - px;
                let dy = inst[i].y - py;
                let dist = (dx * dx + dy * dy).sqrt();
                let dt_s = YAW_STRIDE as f64 * dt;
                let vbl = if bli > 1e-6 {
                    (dist / dt_s) / bli
                } else {
                    0.0
                };
                prev_xy[i] = (inst[i].x, inst[i].y);
                // 只要在动就记转向/极化；文献 speed_floor 只在打分里罚游速项。
                const MOVE_EPS: f64 = 0.008;
                if vbl >= MOVE_EPS {
                    kind_spd[ki] += vbl;
                    kind_spd_n[ki] += 1.0;
                    speed_bl_acc += vbl;
                    speed_bl_n += 1.0;
                    let h = dy.atan2(dx);
                    pu_x += h.cos();
                    pu_y += h.sin();
                    pu_n += 1.0;
                    kux[ki] += h.cos();
                    kuy[ki] += h.sin();
                    kun[ki] += 1.0;
                    if have_head[i] {
                        let dyaw = wrap_pi(h - prev_head[i]).abs();
                        let rate = dyaw / dt_s;
                        yaw_samples.push(rate);
                        kind_yaw[ki].push(rate);
                        sharp_d += 1.0;
                        if dyaw > 75.0_f64.to_radians() {
                            sharp_n += 1.0;
                        }
                    }
                    prev_head[i] = h;
                    have_head[i] = true;
                } else {
                    have_head[i] = false;
                }
            }
            if step > warmup + 8
                && prev_yaw[i].signum() != 0.0
                && yaw.signum() != 0.0
                && prev_yaw[i].signum() != yaw.signum()
                && yaw.abs() > 0.10
            {
                flips += 1.0;
            }
            prev_yaw[i] = yaw;
            prev_rot[i] = inst[i].rot;
            speed_acc += inst[i].speed;
            let cruise = (crate::gait::gait(inst[i].ci).cruise
                * crate::life::species_life(inst[i].ci, life).pace)
                .max(1e-6);
            cruise_acc += inst[i].speed / cruise;
            if inst[i].evade_t > 0.0 {
                evade_n += 1.0;
            }
            let gl = ((inst[i].x - 0.5).hypot(inst[i].y - 0.5)).max(0.08);
            let gnx = (inst[i].y - 0.5) / gl;
            let gny = (0.5 - inst[i].x) / gl;
            gyre_acc += (fx * gnx + fy * gny).clamp(-1.0, 1.0);
            let gx = ((inst[i].x - 0.08) / 0.84 * 4.0).floor().clamp(0.0, 3.0) as usize;
            let gy = ((inst[i].y - 0.10) / 0.80 * 4.0).floor().clamp(0.0, 3.0) as usize;
            cells[gy * 4 + gx] += 1;
            if (inst[i].x < 0.18 || inst[i].x > 0.82) && (inst[i].y < 0.20 || inst[i].y > 0.80) {
                corner += 1.0;
            }
        }
        min_nn = min_nn.min(frame_min);
        min_nnd_bl = min_nnd_bl.min(frame_min_bl);
        if stride_frame && pu_n >= 2.0 {
            polar_acc += (pu_x / pu_n).hypot(pu_y / pu_n);
            polar_n += 1.0;
        }
        if stride_frame {
            for k in 0..17 {
                if kun[k] >= 2.0 {
                    kind_polar[k] += (kux[k] / kun[k]).hypot(kuy[k] / kun[k]);
                    kind_polar_n[k] += 1.0;
                }
            }
        }
    }
    let nf = n as f64;
    let pair_frames = samples * nf * (nf - 1.0) / 2.0;
    let mut ent = 0.0;
    let tot = cells.iter().sum::<u32>() as f64;
    if tot > 0.0 {
        for c in cells {
            if c == 0 {
                continue;
            }
            let p = c as f64 / tot;
            ent -= p * p.ln();
        }
    }
    let seconds = samples / 60.0;
    crate::life::SchoolStats {
        mean_nn: if samples > 0.0 { nn_acc / (samples * nf) } else { 0.0 },
        min_nn: if min_nn.is_finite() { min_nn } else { 0.0 },
        overlap_frac: if pair_frames > 0.0 {
            overlap_n / pair_frames
        } else {
            0.0
        },
        align: if samples > 0.0 {
            align_acc / (samples * nf)
        } else {
            0.0
        },
        yaw_flips: if seconds > 0.0 && nf > 0.0 {
            flips / (seconds * nf)
        } else {
            0.0
        },
        cell_entropy: ent,
        corner_frac: if samples > 0.0 {
            corner / (samples * nf)
        } else {
            0.0
        },
        mean_speed: if samples > 0.0 {
            speed_acc / (samples * nf)
        } else {
            0.0
        },
        closest: if closest.is_finite() { closest } else { 0.0 },
        graze_frac: if pair_frames > 0.0 {
            graze_n / pair_frames
        } else {
            0.0
        },
        gyre_align: if samples > 0.0 {
            gyre_acc / (samples * nf)
        } else {
            0.0
        },
        cruise_ratio: if samples > 0.0 {
            cruise_acc / (samples * nf)
        } else {
            0.0
        },
        evade_frac: if samples > 0.0 {
            evade_n / (samples * nf)
        } else {
            0.0
        },
        mean_nnd_bl: if samples > 0.0 {
            nnd_bl_acc / (samples * nf)
        } else {
            0.0
        },
        min_nnd_bl: if min_nnd_bl.is_finite() { min_nnd_bl } else { 0.0 },
        mean_abs_yaw: median_f64(&mut yaw_samples),
        sharp_frac: if sharp_d > 0.0 { sharp_n / sharp_d } else { 0.0 },
        polar: if polar_n > 0.0 { polar_acc / polar_n } else { 0.0 },
        mean_speed_bl: if speed_bl_n > 0.0 {
            speed_bl_acc / speed_bl_n
        } else {
            0.0
        },
        kinds: {
            let mut kinds = [crate::life::KindBio::default(); 17];
            for k in 0..17 {
                kinds[k] = crate::life::KindBio {
                    n: kind_nnd_n[k],
                    nnd_bl: if kind_nnd_n[k] > 0.0 {
                        kind_nnd[k] / kind_nnd_n[k]
                    } else {
                        0.0
                    },
                    yaw: median_f64(&mut kind_yaw[k]),
                    polar: if kind_polar_n[k] > 0.0 {
                        kind_polar[k] / kind_polar_n[k]
                    } else {
                        0.0
                    },
                    have_polar: kind_polar_n[k] > 0.0,
                    speed_bl: if kind_spd_n[k] > 0.0 {
                        kind_spd[k] / kind_spd_n[k]
                    } else {
                        0.0
                    },
                };
            }
            kinds
        },
    }
}

fn median_f64(xs: &mut [f64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    xs[xs.len() / 2]
}

#[cfg(test)]
pub fn simulate_headon(life: &LifeParams) -> (f64, f64, f64) {
    let mut inst = spawn_with(11, 2);
    inst[0].ci = 2;
    inst[1].ci = 12;
    inst[0].x = 0.37;
    inst[0].y = 0.50;
    inst[0].rot = 0.0;
    inst[0].speed = 0.018;
    inst[0].vx = 0.018;
    inst[0].vy = 0.0;
    inst[1].x = 0.63;
    inst[1].y = 0.50;
    inst[1].rot = std::f64::consts::PI;
    inst[1].speed = 0.018;
    inst[1].vx = -0.018;
    inst[1].vy = 0.0;
    let r0 = inst[0].rot;
    let r1 = inst[1].rot;
    let scratches = vec![Vec::new(); 2];
    let dt = 1.0 / 60.0;
    let mut closest = (inst[0].x - inst[1].x).hypot(inst[0].y - inst[1].y);
    for step in 0..240 {
        advance_morph(&mut inst, dt);
        integrate_with(
            &mut inst,
            &scratches,
            step as f64 * dt,
            dt,
            None,
            (1920.0, 1080.0),
            true,
            life,
        );
        closest = closest.min((inst[0].x - inst[1].x).hypot(inst[0].y - inst[1].y));
    }
    let final_d = (inst[0].x - inst[1].x).hypot(inst[0].y - inst[1].y);
    let turned = wrap_pi(inst[0].rot - r0).abs() + wrap_pi(inst[1].rot - r1).abs();
    (closest, final_d, turned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::life::{
        evolve, evolve_species_cfg, score, score_species, species_space_cap, LifeParams,
        SpeciesCma, SpeciesSearch, CMA_SIGMA0, LIFE, SPECIES_BIO,
    };
    use std::io::Write;

    fn transform_pts(pts: &[[f32; 2]], cx: f64, cy: f64, face: f64) -> Vec<(f64, f64)> {
        let ang = -face;
        let (ca, sa) = (ang.cos(), ang.sin());
        pts.iter()
            .step_by((pts.len() / 800).max(1))
            .map(|p| {
                let dx = p[0] as f64 - cx;
                let dy = p[1] as f64 - cy;
                (dx * ca - dy * sa, dx * sa + dy * ca)
            })
            .collect()
    }

    fn end_widths(xy: &[(f64, f64)]) -> (f64, f64, f64, f64) {
        let mut minx = f64::MAX;
        let mut maxx = f64::MIN;
        for (x, _) in xy {
            minx = minx.min(*x);
            maxx = maxx.max(*x);
        }
        let span = (maxx - minx).max(1.0);
        let mut fw = (f64::MAX, f64::MIN);
        let mut aw = (f64::MAX, f64::MIN);
        let mut nf = 0.0;
        let mut na = 0.0;
        for (x, y) in xy {
            if *x > maxx - span * 0.32 {
                fw.0 = fw.0.min(*y);
                fw.1 = fw.1.max(*y);
                nf += 1.0;
            } else if *x < minx + span * 0.32 {
                aw.0 = aw.0.min(*y);
                aw.1 = aw.1.max(*y);
                na += 1.0;
            }
        }
        (
            if nf > 6.0 { fw.1 - fw.0 } else { 0.0 },
            if na > 6.0 { aw.1 - aw.0 } else { 0.0 },
            nf,
            na,
        )
    }

    fn mean_nn_of(inst: &[Creature]) -> (f64, f64) {
        let n = inst.len();
        let mut acc = 0.0;
        let mut min = f64::MAX;
        for i in 0..n {
            let mut nn = f64::MAX;
            for j in 0..n {
                if i == j {
                    continue;
                }
                nn = nn.min((inst[i].x - inst[j].x).hypot(inst[i].y - inst[j].y));
            }
            acc += nn;
            min = min.min(nn);
        }
        (acc / n as f64, min)
    }

    fn axis_profile(pts: &[[f32; 2]], cx: f64, cy: f64, face: f64) -> (f64, f64, f64, f64, usize, [u32; 12]) {
        let xy = transform_pts(pts, cx, cy, face);
        let (fw, aw, nf, na) = end_widths(&xy);
        let mut minx = f64::MAX;
        let mut maxx = f64::MIN;
        for (x, _) in &xy {
            minx = minx.min(*x);
            maxx = maxx.max(*x);
        }
        let span = (maxx - minx).max(1.0);
        let mut bins = [0u32; 12];
        for (x, _) in &xy {
            let i = (((*x - minx) / span) * 12.0).floor() as usize;
            bins[i.min(11)] += 1;
        }
        let peak = bins
            .iter()
            .enumerate()
            .max_by_key(|(_, c)| *c)
            .map(|(i, _)| i)
            .unwrap_or(0);
        (fw, aw, nf, na, peak, bins)
    }

    #[test]
    fn audit_heading_polarity() {
        println!("id         kind         face°  fwd_w  aft_w  r_w   nf    na   peak  note");
        for ci in 0..SPECIES.len() {
            let kind = crate::gait::gait(ci).kind;
            let mut pts = Vec::new();
            (SPECIES[ci].fill as crate::formulas::FillFn)(1.2, 3, &mut pts);
            let (cx, cy, _) = super::shape_stats(&pts);
            let face = super::species_face(ci, 1.2);
            let (fw, aw, nf, na, peak, bins) = axis_profile(&pts, cx, cy, face);
            let rw = fw / aw.max(1.0);
            let note = match kind {
                crate::gait::GaitKind::SpinDrift => "radial",
                crate::gait::GaitKind::Jet if SPECIES[ci].id == "nebula" => {
                    if rw < 0.85 { "ok-narrow-bell" } else { "FLIP?" }
                }
                crate::gait::GaitKind::Jet | crate::gait::GaitKind::Ciliary => {
                    if rw > 1.15 { "ok-wide-front" } else { "FLIP?" }
                }
                crate::gait::GaitKind::Hover | crate::gait::GaitKind::Helix => "axis",
                _ => {
                    if peak >= 6 { "ok-dense-front" } else { "FLIP?" }
                }
            };
            println!(
                "{:<10} {:<12} {:>6.1} {fw:>6.0} {aw:>6.0} {rw:>5.2} {nf:>5.0} {na:>5.0} {peak:>5}  {note}  {bins:?}",
                SPECIES[ci].id,
                format!("{kind:?}"),
                face.to_degrees()
            );
        }
    }

    #[test]
    fn fucan_head_leads() {
        let ci = SPECIES.iter().position(|s| s.id == "fucan").unwrap();
        let mut pts = Vec::new();
        (SPECIES[ci].fill as crate::formulas::FillFn)(1.2, 3, &mut pts);
        let (cx, cy, _) = super::shape_stats(&pts);
        let face = super::species_face(ci, 1.2);
        let xy = transform_pts(&pts, cx, cy, face);
        let mut minx = f64::MAX;
        let mut maxx = f64::MIN;
        for (x, _) in &xy {
            minx = minx.min(*x);
            maxx = maxx.max(*x);
        }
        let span = (maxx - minx).max(1.0);
        let mut bins = [0u32; 12];
        for (x, _) in &xy {
            let i = (((*x - minx) / span) * 12.0).floor() as usize;
            bins[i.min(11)] += 1;
        }
        let peak = bins
            .iter()
            .enumerate()
            .max_by_key(|(_, c)| *c)
            .map(|(i, _)| i)
            .unwrap_or(0);
        println!(
            "fucan face={:.1}° densest_bin={peak} bins={bins:?}",
            face.to_degrees()
        );
        assert!(
            peak >= 6,
            "fucan head (dense end) not forward, densest bin={peak}"
        );
    }

    #[test]
    fn jellyfish_bell_leads_tentacles() {
        use crate::formulas::HeadingKind;
        for ci in 0..SPECIES.len() {
            let kind = crate::gait::gait(ci).kind;
            if !matches!(kind, crate::gait::GaitKind::Jet | crate::gait::GaitKind::Ciliary) {
                continue;
            }
            assert_eq!(
                SPECIES[ci].heading,
                HeadingKind::Bell,
                "{} jet/ciliary must be Bell heading",
                SPECIES[ci].id
            );
        }
    }

    #[test]
    fn lantern_bell_stays_forward() {
        let ci = SPECIES.iter().position(|s| s.id == "lantern").unwrap();
        let face0 = super::SPECIES_FACE[ci];
        let rate = super::SPECIES_FACE_RATE[ci];
        assert!(rate > 0.05, "lantern formula spins, face must track t");
        for k in 0..14 {
            let t = k as f64 * 0.45;
            let face = super::species_face(ci, t);
            let expect = super::wrap_pi(face0 + rate * t);
            let err = super::wrap_pi(face - expect).abs().to_degrees();
            println!(
                "lantern t={t:.2} face={:.1}° expect={:.1}° err={err:.2}",
                face.to_degrees(),
                expect.to_degrees()
            );
            assert!(err < 0.5, "lantern face(t) drifted {err:.1}° at t={t}");
        }
    }

    #[test]
    fn jet_bell_leads_and_face_stays_put() {
        let mut inst = spawn(7, 17);
        inst.retain(|c| matches!(crate::gait::gait(c.ci).kind, crate::gait::GaitKind::Jet));
        assert!(!inst.is_empty());
        let mut scratches = vec![Vec::new(); inst.len()];
        let dt = 1.0 / 60.0;
        let mut prev_vis: Vec<f64> = inst.iter().map(|c| c.rot - c.face).collect();
        let mut vis_j = 0.0;
        let mut n = 0.0;
        for step in 0..240 {
            advance_morph(&mut inst, dt);
            fill_creatures(&inst, 2, &mut scratches);
            integrate(
                &mut inst,
                &scratches,
                step as f64 * dt,
                dt,
                None,
                (1920.0, 1080.0),
                true,
            );
            if step < 24 {
                for (i, c) in inst.iter().enumerate() {
                    prev_vis[i] = c.rot - c.face + c.pose_sway;
                }
                continue;
            }
            for (i, c) in inst.iter().enumerate() {
                let expect = super::species_face(c.ci, c.t);
                assert!(
                    (c.face - expect).abs() < 1e-6,
                    "{} face drifted from jet axis",
                    SPECIES[c.ci].id
                );
                let vis = c.rot - c.face + c.pose_sway;
                vis_j += (wrap_pi(vis - prev_vis[i]) / dt).powi(2);
                prev_vis[i] = vis;
                n += 1.0;
                let v = (c.vx * c.vx + c.vy * c.vy).sqrt().max(1e-9);
                let align = (c.rot.cos() * c.vx - c.rot.sin() * c.vy) / v;
                assert!(
                    align > 0.72,
                    "{} not jetting along bell axis align={align:.2}",
                    SPECIES[c.ci].id
                );
            }
        }
        let rms_vis = (vis_j / n).sqrt().to_degrees();
        println!("jet vis rms={rms_vis:.1}°/s");
        assert!(rms_vis < 16.0, "jet visual heading shaking {rms_vis:.1}°/s");
    }

    #[test]
    fn jelly_track_monitor() {
        let mut inst = spawn(7, 17);
        inst.retain(|c| SPECIES[c.ci].id == "jelly");
        assert_eq!(inst.len(), 1);
        let mut scratches = vec![Vec::new(); 1];
        let dt = 1.0 / 60.0;
        let mut prev_vis = inst[0].rot - inst[0].face;
        let mut prev_x = inst[0].x;
        let mut prev_y = inst[0].y;
        let mut vis_j = 0.0;
        let mut n = 0.0;
        println!("s     face°   rot°   wake°  vis°   dvis°  speed");
        for step in 0..240 {
            advance_morph(&mut inst, dt);
            fill_creatures(&inst, 2, &mut scratches);
            integrate(
                &mut inst,
                &scratches,
                step as f64 * dt,
                dt,
                None,
                (1920.0, 1080.0),
                true,
            );
            let c = &inst[0];
            let vis = c.rot - c.face + c.pose_sway;
            if step < 24 {
                prev_vis = vis;
                prev_x = c.x;
                prev_y = c.y;
                continue;
            }
            let dvis = wrap_pi(vis - prev_vis).abs() / dt;
            vis_j += dvis * dvis;
            n += 1.0;
            if step % 30 == 0 {
                println!(
                    "{:>4.2} {:>6.1} {:>6.1} {:>6.1} {:>6.1} {:>6.1} {:>6.4}",
                    step as f64 * dt,
                    c.face.to_degrees(),
                    c.rot.to_degrees(),
                    c.pose_sway.to_degrees(),
                    vis.to_degrees(),
                    dvis.to_degrees(),
                    c.speed
                );
            }
            prev_vis = vis;
            prev_x = c.x;
            prev_y = c.y;
        }
        let rms_vis = (vis_j / n).sqrt().to_degrees();
        let disp = (inst[0].x - prev_x).abs() + (inst[0].y - prev_y).abs();
        let _ = disp;
        println!("jelly vis rms={rms_vis:.1}°/s face={:.1}°", inst[0].face.to_degrees());
        assert!(
            (inst[0].face - super::species_face(inst[0].ci, inst[0].t)).abs() < 1e-6,
            "jelly must keep bell-forward jet axis"
        );
        assert!(
            inst[0].pose_sway.abs() < 0.10,
            "jelly wake lag still looks like crab-walk sway={:.1}°",
            inst[0].pose_sway.to_degrees()
        );
        assert!(rms_vis < 14.0, "jelly still shaking {rms_vis:.1}°/s");
    }

    #[test]
    fn indicator_ring_centers_on_body() {
        let pts: Vec<[f32; 2]> = (0..400)
            .map(|i| [90.0 + (i % 20) as f32, 40.0 + (i / 20) as f32])
            .collect();
        let (ax, ay, r) = super::visual_ring(&pts, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        assert!(
            (ax - 99.5).abs() < 2.0,
            "ring x={ax} should sit on blob, not origin"
        );
        assert!(
            (ay + 49.5).abs() < 2.0,
            "ring y={ay} should follow flipped screen y"
        );
        let small: Vec<[f32; 2]> = (0..80)
            .map(|i| [90.0 + (i % 8) as f32, 40.0 + (i / 8) as f32])
            .collect();
        let (_, _, rs) = super::visual_ring(&small, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        assert!(r > rs + 4.0, "larger body must get a larger ring ({r} vs {rs})");
    }

    #[test]
    fn every_head_locks_to_heading() {
        fn vis_err(pts: &[[f32; 2]], cx: f64, cy: f64, face: f64) -> f64 {
            if pts.len() < 24 {
                return 0.0;
            }
            let ang = -face;
            let (ca, sa) = (ang.cos(), ang.sin());
            let mut xx = 0.0;
            let mut xy = 0.0;
            let mut yy = 0.0;
            let mut m = 0.0;
            for p in pts.iter().step_by((pts.len() / 360).max(1)) {
                let dx = p[0] as f64 - cx;
                let dy = p[1] as f64 - cy;
                let x = dx * ca - dy * sa;
                let y = dx * sa + dy * ca;
                xx += x * x;
                xy += x * y;
                yy += y * y;
                m += 1.0;
            }
            if m < 8.0 {
                return 0.0;
            }
            xx /= m;
            xy /= m;
            yy /= m;
            let tr = xx + yy;
            let disc = (tr * tr - 4.0 * (xx * yy - xy * xy)).max(0.0).sqrt();
            let l1 = 0.5 * (tr + disc);
            let (ex, ey) = if xy.abs() > 1e-8 || (l1 - xx).abs() > 1e-8 {
                (xy, l1 - xx)
            } else {
                (1.0, 0.0)
            };
            let el = (ex * ex + ey * ey).sqrt().max(1e-9);
            let axis = (ey / el).atan2(ex / el);
            axis.abs()
                .min(wrap_pi(axis + std::f64::consts::PI).abs())
                .to_degrees()
        }
        println!("id         kind         face°   err°");
        assert_eq!(SPECIES_FACE.len(), SPECIES.len());
        assert_eq!(super::SPECIES_FACE_RATE.len(), SPECIES.len());
        let mut worst: f64 = 0.0;
        for ci in 0..SPECIES.len() {
            let kind = crate::gait::gait(ci).kind;
            let mut pts = Vec::new();
            (SPECIES[ci].fill as crate::formulas::FillFn)(1.2, 4, &mut pts);
            let (cx, cy, _) = super::shape_stats(&pts);
            let face = super::species_face(ci, 1.2);
            let err = vis_err(&pts, cx, cy, face);
            if !matches!(kind, crate::gait::GaitKind::SpinDrift) {
                worst = worst.max(err);
            }
            println!(
                "{:<10} {:<12} {:>6.1} {err:>6.1}",
                SPECIES[ci].id,
                format!("{kind:?}"),
                face.to_degrees()
            );
            if !matches!(
                kind,
                crate::gait::GaitKind::SpinDrift
                    | crate::gait::GaitKind::Hover
                    | crate::gait::GaitKind::Jet
                    | crate::gait::GaitKind::Ciliary
            )                 && SPECIES[ci].id != "vortex"
                && SPECIES[ci].id != "tentacle"
                && SPECIES[ci].id != "shrimp"
            {
                assert!(
                    err < 20.0,
                    "{} head not locked, visual axis off {err:.1}°",
                    SPECIES[ci].id
                );
            }
        }
        println!("worst vis_err={worst:.1}°");
    }

    #[test]
    fn spawn_is_evenly_spread() {
        for seed in [1u32, 7, 99, 2026] {
            let inst = spawn(seed, 17);
            let (mean, min) = mean_nn_of(&inst);
            assert!(
                min > 0.09,
                "seed {seed} stacked at spawn min_nn={min:.3} mean={mean:.3}"
            );
            assert!(mean > 0.15, "seed {seed} clumped mean_nn={mean:.3}");
        }
    }

    #[test]
    fn school_stays_spread() {
        let s = simulate_school(7, 17, 10.0, &LIFE);
        println!(
            "school mean_nn={:.3} nnd_bl={:.2} min_bl={:.2} closest={:.3} graze={:.3} overlap={:.3} align={:.2} yaw={:.2} sharp={:.3} flips={:.2} H={:.2} gyre={:.2} cruise={:.2} evade={:.2} v={:.4} score={:.2}",
            s.mean_nn,
            s.mean_nnd_bl,
            s.min_nnd_bl,
            s.closest,
            s.graze_frac,
            s.overlap_frac,
            s.align,
            s.mean_abs_yaw,
            s.sharp_frac,
            s.yaw_flips,
            s.cell_entropy,
            s.gyre_align,
            s.cruise_ratio,
            s.evade_frac,
            s.mean_speed,
            score(&s, &LIFE)
        );
        assert!(s.mean_nn > 0.11, "mean nn too small {}", s.mean_nn);
        assert!(s.mean_nn < 0.24, "too shy mean {}", s.mean_nn);
        assert!(s.min_nn > 0.035, "piled together min {}", s.min_nn);
        assert!(s.closest < 0.16, "never graze closest {}", s.closest);
        assert!(s.overlap_frac < 0.14, "overlap {}", s.overlap_frac);
        assert!(s.align > 0.70, "align {}", s.align);
        assert!(s.cell_entropy > 1.6, "coverage {}", s.cell_entropy);
        assert!(s.cruise_ratio > 0.45, "school stalled {}", s.cruise_ratio);
        assert!(s.align > 0.90, "heading not locked to travel {}", s.align);
    }

    #[test]
    fn nobody_swims_in_circles() {
        let mut inst = spawn_with(7, 17);
        let scratches = vec![Vec::new(); inst.len()];
        let dt = 1.0 / 60.0;
        let nsteps = (12.0 / dt) as usize;
        let warmup = (1.5 / dt) as usize;
        let n = inst.len();
        let mut net = vec![0.0; n];
        let mut abs_h = vec![0.0; n];
        let mut prev = inst.iter().map(|c| c.rot).collect::<Vec<_>>();
        let mut wind = vec![0.0; n];
        let mut prev_v = vec![(0.0, 0.0); n];
        for step in 0..nsteps {
            advance_morph(&mut inst, dt);
            integrate_with(
                &mut inst,
                &scratches,
                step as f64 * dt,
                dt,
                None,
                (1920.0, 1080.0),
                true,
                &LIFE,
            );
            if step < warmup {
                for i in 0..n {
                    prev[i] = inst[i].rot;
                    prev_v[i] = (inst[i].vx, inst[i].vy);
                }
                continue;
            }
            for i in 0..n {
                let dh = wrap_pi(inst[i].rot - prev[i]);
                net[i] += dh;
                abs_h[i] += dh.abs();
                prev[i] = inst[i].rot;
                let (px, py) = prev_v[i];
                let (vx, vy) = (inst[i].vx, inst[i].vy);
                if px * px + py * py > 1e-10 && vx * vx + vy * vy > 1e-10 {
                    wind[i] += wrap_pi(vy.atan2(vx) - py.atan2(px));
                }
                prev_v[i] = (vx, vy);
            }
        }
        let tau = std::f64::consts::TAU;
        for i in 0..n {
            let id = SPECIES[inst[i].ci].id;
            let kind = crate::gait::gait(inst[i].ci).kind;
            let turns = net[i].abs() / tau;
            let winding = wind[i].abs() / tau;
            println!(
                "{id:<10} net={turns:.2} abs={:.2} wind={winding:.2}",
                abs_h[i] / tau
            );
            if kind != crate::gait::GaitKind::SpinDrift {
                assert!(
                    turns < 0.85,
                    "{id} heading circled {turns:.2} turns"
                );
                assert!(
                    winding < 1.15,
                    "{id} path wound {winding:.2} turns"
                );
            } else {
                assert!(
                    turns < 1.10,
                    "{id} heading circled {turns:.2} turns"
                );
            }
        }
    }

    #[test]
    fn wall_paths_curve_instead_of_rail() {
        let mut inst = spawn_with(3, 1);
        inst[0].ci = 12; // saweel：细长游动，用来测贴边而不是悬浮
        inst[0].x = 0.12;
        inst[0].y = 0.35;
        inst[0].rot = -std::f64::consts::FRAC_PI_2;
        inst[0].speed = 0.030;
        let scratches = vec![Vec::new(); 1];
        let dt = 1.0 / 60.0;
        let mut pts = Vec::new();
        for step in 0..(8.0 / dt) as usize {
            advance_morph(&mut inst, dt);
            integrate_with(
                &mut inst,
                &scratches,
                step as f64 * dt,
                dt,
                None,
                (1920.0, 1080.0),
                true,
                &LIFE,
            );
            if step % 4 == 0 {
                pts.push((inst[0].x, inst[0].y));
            }
        }
        let mut sharp = 0usize;
        for i in 2..pts.len() {
            let (x0, y0) = pts[i - 2];
            let (x1, y1) = pts[i - 1];
            let (x2, y2) = pts[i];
            let ax = x1 - x0;
            let ay = y1 - y0;
            let bx = x2 - x1;
            let by = y2 - y1;
            let na = (ax * ax + ay * ay).sqrt();
            let nb = (bx * bx + by * by).sqrt();
            if na < 1.2e-3 || nb < 1.2e-3 {
                continue;
            }
            let dot = (ax * bx + ay * by) / (na * nb);
            if dot < 0.20 {
                sharp += 1;
            }
        }
        assert!(
            sharp < 3,
            "path has {sharp} near-right-angle corners (unbiological wall slide)"
        );
        let x_min = pts.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
        let x_max = pts.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
        assert!(
            x_max - x_min > 0.06,
            "stayed on a vertical rail x=[{x_min:.3},{x_max:.3}]"
        );
    }

    #[test]
    fn headon_yields_and_turns() {
        let (closest, final_d, turned) = simulate_headon(&LIFE);
        println!("headon closest={closest:.3} final={final_d:.3} turned={turned:.2}");
        assert!(closest > 0.045, "they passed through each other {closest}");
        assert!(closest < 0.20, "too shy to approach {closest}");
        assert!(final_d > 0.07, "still stuck together {final_d}");
        assert!(turned > 0.22, "did not turn away {turned}");
        assert!(turned < 4.2, "spun out {turned}");
    }

    #[test]
    fn overlap_pushes_apart() {
        let mut inst = spawn_with(3, 2);
        inst[0].ci = 3;
        inst[1].ci = 3;
        let g = crate::gait::gait(3);
        for c in &mut inst {
            c.omega = g.hz * std::f64::consts::TAU;
            c.speed = g.cruise;
            c.face = super::species_face(3, 0.0);
        }
        inst[0].x = 0.50;
        inst[0].y = 0.50;
        inst[0].rot = 0.0;
        inst[0].vx = g.cruise;
        inst[0].vy = 0.0;
        inst[1].x = 0.508;
        inst[1].y = 0.50;
        inst[1].rot = std::f64::consts::PI;
        inst[1].vx = -g.cruise;
        inst[1].vy = 0.0;
        let d0 = (inst[0].x - inst[1].x).hypot(inst[0].y - inst[1].y);
        let scratches = vec![Vec::new(); 2];
        let dt = 1.0 / 60.0;
        for step in 0..90 {
            advance_morph(&mut inst, dt);
            integrate_with(
                &mut inst,
                &scratches,
                step as f64 * dt,
                dt,
                None,
                (1920.0, 1080.0),
                true,
                &LIFE,
            );
        }
        let d1 = (inst[0].x - inst[1].x).hypot(inst[0].y - inst[1].y);
        println!("overlap d0={d0:.4} d1={d1:.4}");
        assert!(d0 < 0.02, "fixture not overlapping d0={d0}");
        assert!(d1 > 0.055, "jellies stayed stacked d1={d1}");
    }

    #[test]
    fn life_score_snapshot() {
        let mix = simulate_school(7, 17, 8.0, &LIFE);
        let mix_sc = score(&mix, &LIFE);
        println!(
            "mix score={mix_sc:.3} nnd={:.2}BL min={:.2} overlap={:.3} yaw={:.2} polar={:.2} v={:.2}BL/s",
            mix.mean_nnd_bl,
            mix.min_nnd_bl,
            mix.overlap_frac,
            mix.mean_abs_yaw,
            mix.polar,
            mix.mean_speed_bl
        );
        println!("id\tspec\tnnd\tyaw\tpolar\tv\tlive");
        let blind = [0usize, 1, 2, 7, 12, 14, 15];
        for ci in 0..17 {
            let s = simulate_species(7, ci, 8, 6.0, &LIFE);
            let sc = score_species(&s, ci, &LIFE);
            let live = s.mean_speed_bl > 0.04 && (s.mean_abs_yaw > 1e-4 || !LifeParams::heading_trainable(ci));
            println!(
                "{}\t{:.3}\t{:.2}\t{:.2}\t{:.2}\t{:.2}\t{}",
                SPECIES[ci].id,
                sc,
                s.mean_nnd_bl,
                s.mean_abs_yaw,
                s.polar,
                s.mean_speed_bl,
                if live { "y" } else { "n" }
            );
            if blind.contains(&ci) {
                assert!(
                    s.mean_speed_bl > 0.04,
                    "{} still speed-blind {:.3}",
                    SPECIES[ci].id,
                    s.mean_speed_bl
                );
                assert!(
                    s.mean_abs_yaw > 1e-4,
                    "{} still yaw-blind {:.4}",
                    SPECIES[ci].id,
                    s.mean_abs_yaw
                );
            }
        }
        let seed = 20260817u32;
        let eval = |p: &LifeParams| score_species(&simulate_species(seed, 14, 8, 5.0, p), 14, p);
        let parent_sc = eval(&LIFE);
        let search = SpeciesSearch {
            lock_space: true,
            align_only: true,
        };
        let (cand, _, _) = evolve_species_cfg(LIFE, 14, 6, seed, 0.20, search, eval);
        let cand_sc = score_species(&simulate_species(seed.wrapping_add(1), 14, 8, 6.0, &cand), 14, &cand);
        println!(
            "shrimp CMA parent={parent_sc:.3} cand={cand_sc:.3} slip {:.3}->{:.3} zone {:.3}->{:.3} space {:.3}->{:.3} yaw {:.3}->{:.3}",
            LIFE.kinds[14].slip,
            cand.kinds[14].slip,
            LIFE.kinds[14].zone,
            cand.kinds[14].zone,
            LIFE.kinds[14].space,
            cand.kinds[14].space,
            LIFE.kinds[14].yaw,
            cand.kinds[14].yaw
        );
        assert!(
            (cand.kinds[14].space - LIFE.kinds[14].space).abs() < 1e-9,
            "shrimp align-only moved space"
        );
        assert!(
            (cand.kinds[14].yaw - LIFE.kinds[14].yaw).abs() < 1e-9,
            "shrimp align-only moved yaw"
        );
        assert!(
            (cand.kinds[14].pace - LIFE.kinds[14].pace).abs() < 1e-9,
            "shrimp align-only moved pace"
        );
        assert!(
            mix.mean_abs_yaw > 1e-4 && mix.mean_speed_bl > 0.04,
            "mixed school kinematics still dropped"
        );
        assert!(mix_sc > 4.5, "mixed school lifeless {mix_sc:.3}");
    }

    #[test]
    fn swimmer_kinematics_are_recorded() {
        for ci in [0usize, 12, 14] {
            let s = simulate_species(7, ci, 8, 6.0, &LIFE);
            let id = SPECIES[ci].id;
            assert!(
                s.mean_abs_yaw > 1e-4,
                "{id} yaw dropped ({:.4})",
                s.mean_abs_yaw
            );
            assert!(
                s.mean_speed_bl > 0.04,
                "{id} speed_bl=0 ({:.3})",
                s.mean_speed_bl
            );
            assert!(
                s.kinds[ci].speed_bl > 0.04,
                "{id} kind speed_bl={:.3}",
                s.kinds[ci].speed_bl
            );
        }
    }

    #[test]
    fn pace_scales_travel_speed() {
        let mut slow = LIFE;
        let mut fast = LIFE;
        slow.kinds[14].pace = 0.70;
        fast.kinds[14].pace = 2.40;
        let a = simulate_species(9, 14, 8, 6.0, &slow);
        let b = simulate_species(9, 14, 8, 6.0, &fast);
        assert!(
            b.mean_speed_bl > a.mean_speed_bl * 1.25,
            "pace did not speed shrimp {:.3} vs {:.3}",
            b.mean_speed_bl,
            a.mean_speed_bl
        );
    }

    #[test]
    fn shrimp_slip_raises_polar() {
        let mut off = LIFE;
        off.kinds[14].slip = 0.08;
        off.kinds[14].zone = 1.60;
        let mut on = LIFE;
        on.kinds[14].slip = 0.70;
        on.kinds[14].zone = 2.20;
        let a = simulate_species(11, 14, 8, 16.0, &off);
        let b = simulate_species(11, 14, 8, 16.0, &on);
        let trained = simulate_species(11, 14, 8, 24.0, &LIFE);
        println!(
            "shrimp polar local-off={:.3} align={:.3} trained={:.3} nnd {:.2}->{:.2}->{:.2}",
            a.polar, b.polar, trained.polar, a.mean_nnd_bl, b.mean_nnd_bl, trained.mean_nnd_bl
        );
        assert!(
            b.polar > 0.70,
            "steady shrimp polar still capped {:.3}",
            b.polar
        );
        assert!(
            b.polar > a.polar + 0.08,
            "local zone did not raise polar {:.3} vs {:.3}",
            b.polar,
            a.polar
        );
        assert!(
            trained.polar > 0.68,
            "written shrimp LIFE polar too low {:.3}",
            trained.polar
        );
        assert!(
            trained.mean_nnd_bl < 1.50,
            "written shrimp nnd too open {:.2}",
            trained.mean_nnd_bl
        );
    }

    #[test]
    fn parse_life_accepts_six_param_rows() {
        let mut src = format!(
            "LifeParams {{\n    body: {:.3}, near: {:.3}, far: {:.3}, push: {:.3}, far_w: {:.3}, gyre: {:.3}, slide: {:.3},\n    kinds: [\n",
            LIFE.body, LIFE.near, LIFE.far, LIFE.push, LIFE.far_w, LIFE.gyre, LIFE.slide
        );
        for (k, sp) in LIFE.kinds.iter().zip(SPECIES.iter()) {
            src.push_str(&format!(
                "        KindLife {{ space: {:.3}, yaw: {:.3}, brake: {:.3}, slip: {:.3}, wander: {:.3}, shy: {:.3} }}, // {}\n",
                k.space, k.yaw, k.brake, k.slip, k.wander, k.shy, sp.id
            ));
        }
        src.push_str("    ],\n}\n");
        let p = parse_life_rs(&src).expect("old 6-param layout");
        assert!(
            (p.kinds[14].pace - 1.0).abs() < 1e-9,
            "missing pace should stay 1, got {}",
            p.kinds[14].pace
        );
        assert!(
            (p.kinds[14].zone - LIFE.kinds[14].zone).abs() < 1e-9,
            "missing zone should stay baked, got {}",
            p.kinds[14].zone
        );
        assert!((p.kinds[0].space - LIFE.kinds[0].space).abs() < 1e-3);
        assert!((p.kinds[14].yaw - LIFE.kinds[14].yaw).abs() < 1e-3);
    }

    fn eval_life(p: &LifeParams) -> f64 {
        let (acc, (closest, final_d, turned)) = std::thread::scope(|scope| {
            let a = scope.spawn(|| score(&simulate_school(3, 17, 8.0, p), p));
            let b = scope.spawn(|| score(&simulate_school(11, 17, 8.0, p), p));
            let h = scope.spawn(|| simulate_headon(p));
            (
                a.join().expect("school-a") + b.join().expect("school-b"),
                h.join().expect("headon"),
            )
        });
        let head = (closest / 0.07).tanh() * (1.0 - ((closest - 0.09).abs() / 0.12).clamp(0.0, 1.0))
            + (final_d / 0.10).tanh() * 0.4
            + ((turned - 0.20) / 0.7).clamp(0.0, 1.0)
            - ((turned - 2.8).max(0.0) * 0.45);
        acc * 0.5 + head
    }

    #[test]
    #[ignore]
    fn hundred_motion_biology_iterations() {
        let baseline = eval_life(&LIFE);
        let (best, log) = evolve(100, 20260814, eval_life);
        let evolved = eval_life(&best);
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.cache/life-obs");
        let _ = std::fs::create_dir_all(&dir);
        if let Ok(mut f) = std::fs::File::create(dir.join("evolution.log")) {
            let _ = writeln!(f, "baseline={baseline:.3} evolved={evolved:.3}");
            for g in &log {
                let _ = writeln!(f, "gen={:3} best={:.3} mean={:.3}", g.gen, g.best, g.mean);
            }
            let _ = writeln!(f, "{best:#?}");
        }
        for g in log.iter().step_by(10) {
            println!(
                "gen {:>3}  best={:.3}  cohort={:.3}",
                g.gen, g.best, g.mean
            );
        }
        println!(
            "biology ES baseline={baseline:.3} evolved={evolved:.3} evals~{}",
            1 + 100 * 4
        );
        println!("{best:#?}");
        assert!(
            log.last().map(|g| g.best).unwrap_or(0.0) >= baseline - 0.15,
            "evolution collapsed"
        );
        assert!(evolved > 8.0, "evolved school still lifeless {evolved}");
        assert!(baseline > 8.0, "baked LIFE too weak {baseline}");
    }

    fn dump_school_paths(seed: u32, seconds: f64, life: &LifeParams, path: &std::path::Path) {
        dump_paths(seed, 17, None, seconds, life, path);
    }

    fn dump_paths(
        seed: u32,
        count: usize,
        as_ci: Option<usize>,
        seconds: f64,
        life: &LifeParams,
        path: &std::path::Path,
    ) {
        let mut inst = spawn_with(seed, count);
        if let Some(ci) = as_ci {
            let g = crate::gait::gait(ci);
            let pace = species_life(ci, life).pace;
            let omega = g.hz * std::f64::consts::TAU;
            let speed = g.cruise * pace;
            let face = species_face(ci, 0.0);
            for c in &mut inst {
                c.ci = ci;
                c.omega = omega;
                c.speed = speed;
                let ang = c.rot;
                c.vx = speed * ang.cos();
                c.vy = -speed * ang.sin();
                c.face = face;
            }
        }
        let scratches = vec![Vec::new(); inst.len()];
        let dt = 1.0 / 60.0;
        let nsteps = (seconds / dt) as usize;
        let Ok(mut f) = std::fs::File::create(path) else {
            return;
        };
        let _ = writeln!(f, "t,i,id,x,y,rot,speed");
        for step in 0..nsteps {
            advance_morph(&mut inst, dt);
            integrate_with(
                &mut inst,
                &scratches,
                step as f64 * dt,
                dt,
                None,
                (1920.0, 1080.0),
                true,
                life,
            );
            if step % 4 != 0 {
                continue;
            }
            let t = step as f64 * dt;
            for (i, c) in inst.iter().enumerate() {
                let _ = writeln!(
                    f,
                    "{t:.3},{i},{},{:.5},{:.5},{:.4},{:.5}",
                    SPECIES[c.ci].id,
                    c.x,
                    c.y,
                    c.rot,
                    c.speed
                );
            }
        }
    }

    fn write_life_rs(p: &LifeParams) -> String {
        let mut s = format!(
            "LifeParams {{\n    body: {:.3}, near: {:.3}, far: {:.3}, push: {:.3}, far_w: {:.3}, gyre: {:.3}, slide: {:.3},\n    kinds: [\n",
            p.body, p.near, p.far, p.push, p.far_w, p.gyre, p.slide
        );
        for (k, sp) in p.kinds.iter().zip(SPECIES.iter()) {
            s.push_str(&format!(
                "        KindLife {{ space: {:.3}, yaw: {:.3}, brake: {:.3}, slip: {:.3}, wander: {:.3}, shy: {:.3}, pace: {:.3}, zone: {:.3} }}, // {}\n",
                k.space, k.yaw, k.brake, k.slip, k.wander, k.shy, k.pace, k.zone, sp.id
            ));
        }
        s.push_str("    ],\n}\n");
        s
    }

    fn parse_life_rs(src: &str) -> Option<LifeParams> {
        let mut nums = Vec::new();
        let mut cur = String::new();
        let flush = |cur: &mut String, nums: &mut Vec<f64>| {
            if cur.is_empty() {
                return;
            }
            if let Ok(v) = cur.parse::<f64>() {
                nums.push(v);
            }
            cur.clear();
        };
        for line in src.lines() {
            let line = line.split("//").next().unwrap_or("");
            for c in line.chars() {
                if c.is_ascii_digit() || c == '.' || (cur.is_empty() && (c == '-' || c == '+')) {
                    cur.push(c);
                } else {
                    flush(&mut cur, &mut nums);
                }
            }
            flush(&mut cur, &mut nums);
        }
        let n_global = 7;
        let n_kind_new = LifeParams::KIND_DIM;
        let n_v8 = n_global + 17 * 8;
        let n_v7 = n_global + 17 * 7;
        let n_v6 = n_global + 17 * 6;
        if nums.len() < n_v6 {
            return None;
        }
        let n_kind = if nums.len() >= n_v8 {
            8
        } else if nums.len() >= n_v7 {
            7
        } else {
            6
        };
        let mut p = LIFE;
        for i in 0..n_global {
            p.set_param(i, nums[i]);
        }
        for ci in 0..17 {
            for k in 0..n_kind {
                p.set_param(
                    n_global + ci * n_kind_new + k,
                    nums[n_global + ci * n_kind + k],
                );
            }
        }
        Some(p.clamp())
    }

    fn parse_observe_roster(raw: &str) -> Vec<usize> {
        let raw = raw.trim();
        if raw.is_empty()
            || raw.eq_ignore_ascii_case("all")
            || raw == "*"
        {
            return (0..LifeParams::N_SPECIES).collect();
        }
        if raw.eq_ignore_ascii_case("shrimp") {
            return vec![14];
        }
        if raw.eq_ignore_ascii_case("jets") {
            return vec![3, 4, 5];
        }
        if raw.eq_ignore_ascii_case("loose") {
            return vec![0, 1, 2, 7, 10, 11, 12, 15, 16];
        }
        let mut out = Vec::new();
        for part in raw.split(',') {
            let p = part.trim();
            if p.eq_ignore_ascii_case("shrimp") {
                if !out.contains(&14) {
                    out.push(14);
                }
                continue;
            }
            if p.eq_ignore_ascii_case("jets") {
                for ci in [3usize, 4, 5] {
                    if !out.contains(&ci) {
                        out.push(ci);
                    }
                }
                continue;
            }
            if p.eq_ignore_ascii_case("loose") {
                for ci in [0usize, 1, 2, 7, 10, 11, 12, 15, 16] {
                    if !out.contains(&ci) {
                        out.push(ci);
                    }
                }
                continue;
            }
            if let Ok(i) = p.parse::<usize>() {
                if i < LifeParams::N_SPECIES && !out.contains(&i) {
                    out.push(i);
                }
            }
        }
        if out.is_empty() {
            vec![14]
        } else {
            out
        }
    }

    #[test]
    fn parse_observe_roster_named_groups() {
        assert_eq!(parse_observe_roster("shrimp"), vec![14]);
        assert_eq!(parse_observe_roster("jets"), vec![3, 4, 5]);
        assert_eq!(parse_observe_roster("loose").len(), 9);
        assert!(!parse_observe_roster("loose").contains(&14));
        assert!(!parse_observe_roster("loose").contains(&3));
        assert_eq!(parse_observe_roster("all").len(), 17);
        assert_eq!(parse_observe_roster("0,14"), vec![0, 14]);
    }

    fn median3(mut xs: [f64; 3]) -> f64 {
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        xs[1]
    }

    struct Confirm {
        score: f64,
        polar: f64,
        nnd: f64,
    }

    fn confirm_species(ci: usize, seconds: f64, life: &LifeParams) -> Confirm {
        const SEEDS: [u32; 3] = [7, 11, 19];
        let mut scores = [0.0; 3];
        let mut polar = [0.0; 3];
        let mut nnd = [0.0; 3];
        for (i, seed) in SEEDS.iter().enumerate() {
            let s = simulate_species(*seed, ci, 8, seconds, life);
            scores[i] = score_species(&s, ci, life);
            polar[i] = s.polar;
            nnd[i] = s.mean_nnd_bl;
        }
        Confirm {
            score: median3(scores),
            polar: median3(polar),
            nnd: median3(nnd),
        }
    }

    #[test]
    fn confirm_median_rejects_single_seed_spike() {
        assert!((median3([1.0, 9.0, 2.0]) - 2.0).abs() < 1e-12);
        let c = confirm_species(14, 16.0, &LIFE);
        assert!(c.polar > 0.64, "LIFE shrimp confirm polar {}", c.polar);
        assert!(c.nnd < 1.50, "LIFE shrimp confirm nnd {}", c.nnd);
    }

    /// 观察 → 记录 → 演化 → 再观察。
    /// OBSERVE_CI=14|shrimp|jets|loose|all 只训列出的种。
    /// OBSERVE_LOCK_SPACE=1 冻结 space；OBSERVE_ALIGN_ONLY=1 只动 slip/zone（不对 yaw）。
    /// 接受冠军前用种子 7/11/19 的中位数确认，避免单次评估噪声。
    #[test]
    #[ignore]
    fn observe_record_optimize_loop() {
        fn env_flag(key: &str) -> bool {
            matches!(
                std::env::var(key).ok().as_deref().map(str::trim),
                Some("1") | Some("true") | Some("yes") | Some("TRUE")
            )
        }

        let hours: f64 = std::env::var("OBSERVE_HOURS")
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(24.0)
            .clamp(0.01, 72.0);
        let gens: u32 = std::env::var("OBSERVE_GENS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(16);
        let eval_s: f64 = std::env::var("OBSERVE_EVAL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(16.0);
        let eval_s = eval_s.clamp(3.0, 24.0);
        let roster = parse_observe_roster(&std::env::var("OBSERVE_CI").unwrap_or_default());
        let search = SpeciesSearch {
            lock_space: env_flag("OBSERVE_LOCK_SPACE"),
            align_only: env_flag("OBSERVE_ALIGN_ONLY"),
        };
        let sigma0: f64 = std::env::var("OBSERVE_SIGMA")
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(CMA_SIGMA0)
            .clamp(0.12, 0.72);
        let single = roster.len() == 1;
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.cache/life-obs");
        let _ = std::fs::create_dir_all(&dir);
        let journal = dir.join("journal.tsv");
        let mut jf = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&journal)
            .expect("journal");
        if jf.metadata().map(|m| m.len()).unwrap_or(1) == 0 {
            let _ = writeln!(
                jf,
                "cycle\telapsed_h\tscore\tmean_nn\tmin_nn\tclosest\tgraze\talign\tcruise\tevade\theadon_d\tturned\tnnd_bl\tyaw\tspecies\tspec_sc\tpolar\tspeed_bl"
            );
        }
        let t0 = std::time::Instant::now();
        let mut parent = LIFE;
        if env_flag("OBSERVE_LOAD_BEST") {
            if let Some(loaded) = std::fs::read_to_string(dir.join("best.rs"))
                .ok()
                .and_then(|s| parse_life_rs(&s))
            {
                for &ci in &roster {
                    if loaded.kinds[ci].space <= species_space_cap(ci) {
                        parent.kinds[ci] = loaded.kinds[ci];
                    }
                }
            }
        }
        if let Some(v) = std::env::var("OBSERVE_SLIP")
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
        {
            for &ci in &roster {
                parent.kinds[ci].slip = v.clamp(0.05, 2.40);
            }
        }
        if let Some(v) = std::env::var("OBSERVE_ZONE")
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
        {
            for &ci in &roster {
                parent.kinds[ci].zone = v.clamp(1.60, 2.80);
            }
        }
        let mut champ_sc = [f64::NEG_INFINITY; LifeParams::N_SPECIES];
        let mut champ_polar = [-1.0f64; LifeParams::N_SPECIES];
        let mut cmas: Vec<Option<SpeciesCma>> = (0..LifeParams::N_SPECIES).map(|_| None).collect();
        let mut cycle = 0u32;
        fn read_cma_ctrl(dir: &std::path::Path, sigma0: f64, gens0: u32) -> (f64, u32) {
            let mut sigma = sigma0;
            let mut gens = gens0;
            if let Ok(s) = std::fs::read_to_string(dir.join("cma-ctrl.txt")) {
                for line in s.lines() {
                    let line = line.trim();
                    if let Some(v) = line.strip_prefix("sigma=") {
                        if let Ok(x) = v.trim().parse::<f64>() {
                            sigma = x.clamp(0.12, 0.72);
                        }
                    }
                    if let Some(v) = line.strip_prefix("gens=") {
                        if let Ok(x) = v.trim().parse::<u32>() {
                            gens = x.clamp(6, 24);
                        }
                    }
                }
            }
            (sigma, gens)
        }
        println!(
            "observe loop hours={hours} gens/cycle={gens} eval={eval_s:.0}s search=CMA-ES sigma0={sigma0:.2} roster={roster:?} lock_space={} align_only={} dir={}",
            search.lock_space,
            search.align_only,
            dir.display()
        );
        while t0.elapsed().as_secs_f64() < hours * 3600.0 {
            let ci = roster[cycle as usize % roster.len()];
            let id = SPECIES[ci].id;
            let spec_eval = if ci == 14 {
                (eval_s * 1.5).clamp(3.0, 24.0)
            } else {
                eval_s
            };
            let spec = simulate_species(7, ci, 8, spec_eval, &parent);
            let spec_sc = score_species(&spec, ci, &parent);
            let (sc, mean_nn, min_nn, closest, graze, align, cruise, evade, turned) = if single {
                (
                    spec_sc,
                    spec.mean_nn,
                    spec.min_nn,
                    spec.closest,
                    spec.graze_frac,
                    spec.align,
                    spec.cruise_ratio,
                    spec.evade_frac,
                    0.0,
                )
            } else {
                let (school, (closest, _final_d, turned)) = std::thread::scope(|scope| {
                    let school = scope.spawn(|| simulate_school(7, 17, eval_s + 2.0, &parent));
                    let head = scope.spawn(|| simulate_headon(&parent));
                    (
                        school.join().expect("school"),
                        head.join().expect("headon"),
                    )
                });
                let sc = score(&school, &parent);
                (
                    sc,
                    school.mean_nn,
                    school.min_nn,
                    closest,
                    school.graze_frac,
                    school.align,
                    school.cruise_ratio,
                    school.evade_frac,
                    turned,
                )
            };
            let parent_c = confirm_species(ci, spec_eval, &parent);
            if champ_sc[ci].is_infinite() {
                champ_sc[ci] = parent_c.score;
                champ_polar[ci] = parent_c.polar;
            }
            let elapsed_h = t0.elapsed().as_secs_f64() / 3600.0;
            let sigma_now = cmas[ci].as_ref().map(|c| c.sigma).unwrap_or(0.20);
            let _ = writeln!(
                jf,
                "{cycle}\t{elapsed_h:.4}\t{sc:.3}\t{mean_nn:.3}\t{min_nn:.3}\t{closest:.3}\t{graze:.3}\t{align:.2}\t{cruise:.2}\t{evade:.2}\t{closest:.3}\t{turned:.2}\t{:.2}\t{:.2}\t{id}\t{spec_sc:.3}\t{:.2}\t{:.2}",
                spec.mean_nnd_bl,
                spec.mean_abs_yaw,
                spec.polar,
                spec.mean_speed_bl
            );
            let _ = jf.flush();
            println!(
                "cycle {cycle} t={elapsed_h:.3}h species={id} spec={spec_sc:.2} champ={:.2} mix={sc:.2} sigma={:.3} nnd={:.2}BL yaw={:.2} polar={:.2} v={:.2}BL/s headon={closest:.3}/{turned:.2}",
                champ_sc[ci],
                sigma_now,
                spec.mean_nnd_bl,
                spec.mean_abs_yaw,
                spec.polar,
                spec.mean_speed_bl
            );
            let dump_every = if single { 2 } else { 8 };
            if cycle % dump_every == 0 {
                let dump_life = parent;
                let dump_path = dir.join(format!("paths-{cycle:04}.csv"));
                let dump_seed = 7u32.wrapping_add(cycle);
                std::thread::spawn(move || {
                    if single {
                        dump_paths(dump_seed, 8, Some(ci), 16.0, &dump_life, &dump_path);
                    } else {
                        dump_school_paths(dump_seed, 16.0, &dump_life, &dump_path);
                    }
                });
                if let Ok(mut f) = std::fs::File::create(dir.join("best.rs")) {
                    let _ = write!(f, "{}", write_life_rs(&parent));
                }
            }
            let seed = 20260815u32.wrapping_add(cycle.wrapping_mul(17));
            let eval = |p: &LifeParams| {
                let a = score_species(&simulate_species(seed, ci, 8, spec_eval, p), ci, p);
                let b = score_species(
                    &simulate_species(seed.wrapping_add(3), ci, 8, spec_eval, p),
                    ci,
                    p,
                );
                0.5 * (a + b)
            };
            let (ctrl_sigma, ctrl_gens) = read_cma_ctrl(&dir, sigma0, gens);
            if cmas[ci].is_none() {
                cmas[ci] = Some(SpeciesCma::from_life(&parent, ci, search, ctrl_sigma));
            } else if cmas[ci].as_ref().is_some_and(|c| c.sigma <= 0.05) {
                println!(
                    "CMA restart {id} sigma collapsed; inflate to {:.2} (restarts were {})",
                    ctrl_sigma.max(sigma0 * 1.15).min(0.65),
                    cmas[ci].as_ref().map(|c| c.restarts).unwrap_or(0)
                );
                cmas[ci] = Some(SpeciesCma::from_life(
                    &parent,
                    ci,
                    search,
                    ctrl_sigma.max(sigma0 * 1.15).min(0.65),
                ));
            } else {
                let before = cmas[ci].as_ref().map(|c| c.sigma).unwrap_or(0.0);
                cmas[ci].as_mut().expect("cma").inflate_to(ctrl_sigma);
                let after = cmas[ci].as_ref().map(|c| c.sigma).unwrap_or(0.0);
                if after > before + 1e-6 {
                    println!("CMA inflate {id} sigma {before:.3} -> {after:.3} gens={ctrl_gens}");
                }
            }
            let (cand, _) = cmas[ci]
                .as_mut()
                .expect("cma")
                .run(parent, ci, ctrl_gens, seed, eval);
            let cand_c = confirm_species(ci, spec_eval, &cand);
            let sigma_ci = cmas[ci].as_ref().map(|c| c.sigma).unwrap_or(0.0);
            let polar_ok = if ci == 14 {
                cand_c.polar >= 0.68 && cand_c.polar + 0.03 >= parent_c.polar
            } else if SPECIES_BIO[ci].w_polar > 0.0 {
                cand_c.polar <= SPECIES_BIO[ci].polar + 0.38
                    && cand_c.polar <= parent_c.polar + 0.04
            } else {
                true
            };
            if cand_c.score > champ_sc[ci] + 1e-4 && polar_ok {
                println!(
                    "new champion {id} {:.3} -> {:.3} polar {:.2}->{:.2} nnd {:.2}->{:.2} sigma={:.3}",
                    champ_sc[ci],
                    cand_c.score,
                    parent_c.polar,
                    cand_c.polar,
                    parent_c.nnd,
                    cand_c.nnd,
                    sigma_ci
                );
                parent.kinds[ci] = cand.kinds[ci];
                champ_sc[ci] = cand_c.score;
                champ_polar[ci] = cand_c.polar;
            } else if !polar_ok {
                println!(
                    "reject {id} confirm={:.3} polar={:.2} parent_p={:.2} nnd={:.2} sigma={:.3}",
                    cand_c.score, cand_c.polar, parent_c.polar, cand_c.nnd, sigma_ci
                );
            } else {
                println!(
                    "keep champion {id} {:.3} confirm={:.3} polar={:.2} sigma={:.3}",
                    champ_sc[ci], cand_c.score, cand_c.polar, sigma_ci
                );
            }
            cycle += 1;
        }
        if let Ok(mut f) = std::fs::File::create(dir.join("best.rs")) {
            let _ = write!(f, "{}", write_life_rs(&parent));
        }
        println!("observe done cycles={cycle} final={}", write_life_rs(&parent));
        assert!(cycle >= 1, "loop exited before one cycle");
    }
}
