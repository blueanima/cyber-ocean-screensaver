use std::sync::atomic::{AtomicUsize, Ordering};

use crate::formulas::{FillFn, SPECIES, VIEW};
use crate::life::{kind_life, LifeParams, LIFE};

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
    for a in 0..n {
        let ka = kind_life(crate::gait::gait(inst[a].ci).kind, life);
        let ra = inst[a].scale * life.body * ka.space;
        for b in (a + 1)..n {
            let ddx = inst[a].x - inst[b].x;
            let ddy = inst[a].y - inst[b].y;
            let dd = (ddx * ddx + ddy * ddy).sqrt();
            if dd < 1e-5 {
                continue;
            }
            let kb = kind_life(crate::gait::gait(inst[b].ci).kind, life);
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

        let kl = kind_life(g.kind, life);
        let wfreq = match g.kind {
            crate::gait::GaitKind::Jet | crate::gait::GaitKind::Hover => 0.020,
            crate::gait::GaitKind::SpinDrift => 0.018,
            _ => 0.048,
        };
        let rot0 = c.rot;
        let mut bdes = kl.wander * (ocean_t * wfreq + c.phase).sin();
        let steer_f = match g.kind {
            crate::gait::GaitKind::SpinDrift | crate::gait::GaitKind::Helix => 0.05,
            crate::gait::GaitKind::Hover | crate::gait::GaitKind::Jet => 0.14,
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
            crate::gait::GaitKind::Helix => 0.30,
            _ => 1.0,
        };
        bdes += (fwd_x * gny - fwd_y * gnx) * life.gyre * g_w * g_scale;

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

        let my_r = c.scale * life.body * kl.space;
        let sense = match g.kind {
            crate::gait::GaitKind::Jet | crate::gait::GaitKind::Hover => {
                (my_r * 2.20).clamp(0.055, 0.11)
            }
            crate::gait::GaitKind::SpinDrift => (my_r * 2.30).clamp(0.060, 0.12),
            _ => (my_r * (2.35 + 1.05 * kl.shy)).clamp(0.070, 0.16),
        };
        let nnd = nd[i];
        let mut on_course = false;
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
            on_course = nnd < range && closing > 0.20 && impact < hit_r;
            if on_course || (loom && nnd < range) {
                on_course = true;
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
                let urgency = (1.0 - (impact / hit_r.max(1e-6)).clamp(0.0, 1.0))
                    * closing.clamp(0.0, 1.0)
                    * (0.45 + 0.55 * prox);
                c.evade_t = (0.18 + 0.28 * urgency).max(c.evade_t);
                // 已经擦得开就侧滑，不要继续拧航向画圈。
                if impact < hit_r * 0.88 {
                    let yaw = kl.yaw * urgency.max(0.12);
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

        let max_yaw = match g.kind {
            crate::gait::GaitKind::SpinDrift => 0.16,
            crate::gait::GaitKind::Helix => 0.20,
            crate::gait::GaitKind::Hover => 0.22,
            crate::gait::GaitKind::Jet => 0.20,
            crate::gait::GaitKind::Ciliary => 0.28,
            _ => 0.40,
        };
        let dheading = wrap_pi(c.rot - rot0);
        c.rot = rot0 + dheading.clamp(-max_yaw * dt, max_yaw * dt);
        fwd_x = c.rot.cos();
        fwd_y = -c.rot.sin();
        if g.kind == crate::gait::GaitKind::Jet {
            // 触手被动随流：画面朝向滞后于喷水轴，转向时触手扫过尾流。
            let stream = (c.speed / g.cruise.max(1e-6)).clamp(0.45, 1.6);
            let tau = 0.42 / stream;
            let old_vis = rot0 + c.pose_sway;
            let a = 1.0 - (-dt / tau).exp();
            let new_vis = old_vis + wrap_pi(c.rot - old_vis) * a;
            c.pose_sway = wrap_pi(new_vis - c.rot).clamp(-0.36, 0.36);
        }

        if on_course {
            c.speed *= 0.92;
        }
        let drift = 0.0014 * (ocean_t * 0.07 + c.phase).sin();
        // 前进只沿航向：让路靠转向，不靠横移。
        c.vx = c.speed * fwd_x;
        c.vy = c.speed * fwd_y;
        if matches!(
            g.kind,
            crate::gait::GaitKind::Jet | crate::gait::GaitKind::Hover
        ) {
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
        }
        if !matches!(
            g.kind,
            crate::gait::GaitKind::SpinDrift | crate::gait::GaitKind::Hover
        ) {
            let along = (c.vx * fwd_x + c.vy * fwd_y).max(0.0);
            c.vx = along * fwd_x;
            c.vy = along * fwd_y;
        }
        c.x += c.vx * dt;
        c.y += c.vy * dt;

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
    let mut inst = spawn_with(seed, count);
    let scratches = vec![Vec::new(); inst.len()];
    let dt = 1.0 / 60.0;
    let nsteps = (seconds / dt) as usize;
    let warmup = ((1.2 / dt) as usize).min(nsteps / 4);
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
            }
            continue;
        }
        samples += 1.0;
        let mut frame_min = f64::MAX;
        for i in 0..n {
            let mut nn = f64::MAX;
            for j in 0..n {
                if i == j {
                    continue;
                }
                let d = (inst[i].x - inst[j].x).hypot(inst[i].y - inst[j].y);
                nn = nn.min(d);
                if j > i {
                    closest = closest.min(d);
                    let ka = kind_life(crate::gait::gait(inst[i].ci).kind, life);
                    let kb = kind_life(crate::gait::gait(inst[j].ci).kind, life);
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
            nn_acc += nn;
            let fx = inst[i].rot.cos();
            let fy = -inst[i].rot.sin();
            let v = (inst[i].vx * inst[i].vx + inst[i].vy * inst[i].vy).sqrt();
            if v > 1e-6 {
                align_acc += (fx * inst[i].vx + fy * inst[i].vy) / v;
            }
            let yaw = wrap_pi(inst[i].rot - prev_rot[i]) / dt;
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
            let cruise = crate::gait::gait(inst[i].ci).cruise.max(1e-6);
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
    }
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
    use crate::life::{evolve, evolve_from, score, LifeParams, LIFE};
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
            "school mean_nn={:.3} min={:.3} closest={:.3} graze={:.3} overlap={:.3} align={:.2} flips={:.2} H={:.2} gyre={:.2} cruise={:.2} evade={:.2} v={:.4} score={:.2}",
            s.mean_nn,
            s.min_nn,
            s.closest,
            s.graze_frac,
            s.overlap_frac,
            s.align,
            s.yaw_flips,
            s.cell_entropy,
            s.gyre_align,
            s.cruise_ratio,
            s.evade_frac,
            s.mean_speed,
            score(&s)
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
    fn headon_yields_and_turns() {
        let (closest, final_d, turned) = simulate_headon(&LIFE);
        println!("headon closest={closest:.3} final={final_d:.3} turned={turned:.2}");
        assert!(closest > 0.045, "they passed through each other {closest}");
        assert!(closest < 0.20, "too shy to approach {closest}");
        assert!(final_d > 0.07, "still stuck together {final_d}");
        assert!(turned > 0.22, "did not turn away {turned}");
        assert!(turned < 4.2, "spun out {turned}");
    }

    fn eval_life(p: &LifeParams) -> f64 {
        let mut acc = 0.0;
        for seed in [3u32, 11] {
            acc += score(&simulate_school(seed, 17, 8.0, p));
        }
        let (closest, final_d, turned) = simulate_headon(p);
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
        let mut inst = spawn_with(seed, 17);
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
        let names = [
            "jet", "ciliary", "metachronal", "undulate", "flap", "spin", "hover", "helix",
        ];
        let mut s = format!(
            "LifeParams {{\n    body: {:.3}, near: {:.3}, far: {:.3}, push: {:.3}, far_w: {:.3}, gyre: {:.3}, slide: {:.3},\n    kinds: [\n",
            p.body, p.near, p.far, p.push, p.far_w, p.gyre, p.slide
        );
        for (k, name) in p.kinds.iter().zip(names) {
            s.push_str(&format!(
                "        KindLife {{ space: {:.3}, yaw: {:.3}, brake: {:.3}, slip: {:.3}, wander: {:.3}, shy: {:.3} }}, // {name}\n",
                k.space, k.yaw, k.brake, k.slip, k.wander, k.shy
            ));
        }
        s.push_str("    ],\n}\n");
        s
    }

    /// 观察 → 记录 → 演化 → 再观察。默认 24 小时，可用 OBSERVE_HOURS 覆盖。
    #[test]
    #[ignore]
    fn observe_record_optimize_loop() {
        let hours: f64 = std::env::var("OBSERVE_HOURS")
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(24.0)
            .clamp(0.01, 72.0);
        let gens: u32 = std::env::var("OBSERVE_GENS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(16);
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
                "cycle\telapsed_h\tscore\tmean_nn\tmin_nn\tclosest\tgraze\talign\tcruise\tevade\theadon_d\tturned"
            );
        }
        let t0 = std::time::Instant::now();
        let mut parent = LIFE;
        let mut cycle = 0u32;
        println!("observe loop hours={hours} gens/cycle={gens} dir={}", dir.display());
        while t0.elapsed().as_secs_f64() < hours * 3600.0 {
            let (school, (closest, _final_d, turned)) = std::thread::scope(|scope| {
                let school = scope.spawn(|| simulate_school(7, 17, 12.0, &parent));
                let head = scope.spawn(|| simulate_headon(&parent));
                (
                    school.join().expect("school"),
                    head.join().expect("headon"),
                )
            });
            let sc = score(&school);
            let elapsed_h = t0.elapsed().as_secs_f64() / 3600.0;
            let _ = writeln!(
                jf,
                "{cycle}\t{elapsed_h:.4}\t{sc:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.2}\t{:.2}\t{:.2}\t{closest:.3}\t{turned:.2}",
                school.mean_nn,
                school.min_nn,
                school.closest,
                school.graze_frac,
                school.align,
                school.cruise_ratio,
                school.evade_frac
            );
            let _ = jf.flush();
            println!(
                "cycle {cycle} t={elapsed_h:.3}h score={sc:.2} nn={:.3} close={:.3} graze={:.3} cruise={:.2} headon={closest:.3}/{turned:.2}",
                school.mean_nn, school.closest, school.graze_frac, school.cruise_ratio
            );
            if cycle % 8 == 0 {
                dump_school_paths(7, 10.0, &parent, &dir.join(format!("paths-{cycle:04}.csv")));
                if let Ok(mut f) = std::fs::File::create(dir.join("best.rs")) {
                    let _ = write!(f, "{}", write_life_rs(&parent));
                }
            }
            let seed = 20260815u32.wrapping_add(cycle.wrapping_mul(17));
            let (best, _) = evolve_from(parent, gens, seed, eval_life);
            parent = best;
            cycle += 1;
        }
        if let Ok(mut f) = std::fs::File::create(dir.join("best.rs")) {
            let _ = write!(f, "{}", write_life_rs(&parent));
        }
        println!("observe done cycles={cycle} final={}", write_life_rs(&parent));
        assert!(cycle >= 1, "loop exited before one cycle");
    }
}
