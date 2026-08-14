use crate::formulas::{FillFn, SPECIES, VIEW};
use crate::life::{kind_life, LifeParams, LIFE};

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
            face: 0.0,
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

        let kl = kind_life(g.kind, life);
        let wfreq = match g.kind {
            crate::gait::GaitKind::Jet | crate::gait::GaitKind::Hover => 0.020,
            crate::gait::GaitKind::SpinDrift => 0.018,
            _ => 0.048,
        };
        let mut bdes = kl.wander * (ocean_t * wfreq + c.phase).sin();
        bdes += (fwd_x * c.fy - fwd_y * c.fx) * 0.32;
        let gl = ((c.x - 0.5).hypot(c.y - 0.5)).max(0.08);
        let gnx = (c.y - 0.5) / gl;
        let gny = (0.5 - c.x) / gl;
        let nnd_hint = nd.get(i).copied().unwrap_or(f64::MAX);
        let g_w = if nnd_hint < 0.16 { 0.22 } else { 1.0 };
        bdes += (fwd_x * gny - fwd_y * gnx) * life.gyre * g_w;

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
                c.evade_t = (0.22 + 0.40 * urgency).max(c.evade_t);
                let yaw = kl.yaw * urgency.max(0.18);
                c.rot += c.evade_dir * yaw * dt;
                fwd_x = c.rot.cos();
                fwd_y = -c.rot.sin();
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
        if nnd < sense {
            let nx = nnx[i];
            let ny = nny[i];
            let prox = (1.0 - nnd / sense).clamp(0.0, 1.0);
            if on_course || nnd < my_r * 1.7 {
                let into = (-c.vx * nx - c.vy * ny).max(0.0);
                let k = if on_course { 0.40 + 0.40 * prox } else { 0.22 };
                c.vx += nx * into * k;
                c.vy += ny * into * k;
            }
            if c.evade_dir.abs() > 0.5 && (on_course || nnd < sense * 0.75) {
                let sx = -ny * c.evade_dir;
                let sy = nx * c.evade_dir;
                c.vx += sx * c.speed * kl.slip * prox * 0.85;
                c.vy += sy * c.speed * kl.slip * prox * 0.85;
            }
        }
        c.x += c.vx * dt + c.fx * dt * life.slide;
        c.y += c.vy * dt + c.fy * dt * life.slide;

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
    use crate::life::{evolve, score, LIFE};
    use std::io::Write;

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
}
