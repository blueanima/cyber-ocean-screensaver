//! 群体生活：均匀占位、相遇退避、以及运动生物学演化搜索。

use crate::gait::GaitKind;

#[derive(Clone, Copy, Debug)]
pub struct KindLife {
    /// 个体空间倍数
    pub space: f64,
    /// 退避转向 rad/s
    pub yaw: f64,
    /// 对头接近时减速
    pub brake: f64,
    /// 侧滑让路
    pub slip: f64,
    /// 航向漫游幅度
    pub wander: f64,
    /// 多早开始让（远场敏感）
    pub shy: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct LifeParams {
    /// 身体半径 ≈ scale * body
    pub body: f64,
    pub near: f64,
    pub far: f64,
    pub push: f64,
    pub far_w: f64,
    pub gyre: f64,
    pub slide: f64,
    pub kinds: [KindLife; 8],
}

impl KindLife {
    #[cfg(test)]
    fn clamp(self) -> Self {
        Self {
            space: self.space.clamp(0.80, 2.10),
            yaw: self.yaw.clamp(0.08, 0.78),
            brake: self.brake.clamp(0.02, 0.62),
            slip: self.slip.clamp(0.08, 1.05),
            wander: self.wander.clamp(0.015, 0.18),
            shy: self.shy.clamp(0.28, 1.25),
        }
    }
}

impl LifeParams {
    #[cfg(test)]
    pub fn clamp(self) -> Self {
        let mut kinds = self.kinds;
        for k in &mut kinds {
            *k = k.clamp();
        }
        Self {
            body: self.body.clamp(0.07, 0.17),
            near: self.near.clamp(0.90, 1.70),
            far: self.far.clamp(1.90, 4.20),
            push: self.push.clamp(0.50, 3.80),
            far_w: self.far_w.clamp(0.12, 1.40),
            gyre: self.gyre.clamp(0.0, 0.16),
            slide: self.slide.clamp(0.06, 0.28),
            kinds,
        }
    }

    #[cfg(test)]
    pub fn mutate(&self, rng: &mut impl FnMut() -> f64) -> Self {
        fn j(rng: &mut impl FnMut() -> f64, v: f64) -> f64 {
            let u = rng() + rng() + rng() - 1.5;
            let jump = if rng() < 0.12 { 2.4 } else { 1.0 };
            v * (1.0 + 0.14 * u * jump)
        }
        let mut kinds = self.kinds;
        for k in &mut kinds {
            *k = KindLife {
                space: j(rng, k.space),
                yaw: j(rng, k.yaw),
                brake: j(rng, k.brake),
                slip: j(rng, k.slip),
                wander: j(rng, k.wander),
                shy: j(rng, k.shy),
            };
        }
        Self {
            body: j(rng, self.body),
            near: j(rng, self.near),
            far: j(rng, self.far),
            push: j(rng, self.push),
            far_w: j(rng, self.far_w),
            gyre: j(rng, self.gyre),
            slide: j(rng, self.slide),
            kinds,
        }
        .clamp()
    }
}

pub fn kind_ix(kind: GaitKind) -> usize {
    match kind {
        GaitKind::Jet => 0,
        GaitKind::Ciliary => 1,
        GaitKind::Metachronal => 2,
        GaitKind::Undulate => 3,
        GaitKind::Flap => 4,
        GaitKind::SpinDrift => 5,
        GaitKind::Hover => 6,
        GaitKind::Helix => 7,
    }
}

pub fn kind_life(kind: GaitKind, life: &LifeParams) -> KindLife {
    life.kinds[kind_ix(kind)]
}

/// 100 代 (1+4) ES 烘出的群体参数（2026-08-14，fitness 15.94 → 16.17）。
/// 水母早感慢让；蠕虫/虾主动侧滑；海天使快转；辐射花贴身漂开。
pub const LIFE: LifeParams = LifeParams {
    body: 0.112,
    near: 1.522,
    far: 3.529,
    push: 2.940,
    far_w: 0.542,
    gyre: 0.045,
    slide: 0.124,
    kinds: [
        KindLife { space: 1.211, yaw: 0.193, brake: 0.164, slip: 0.273, wander: 0.047, shy: 1.017 }, // jet
        KindLife { space: 1.169, yaw: 0.288, brake: 0.074, slip: 0.395, wander: 0.044, shy: 0.725 }, // ciliary
        KindLife { space: 0.984, yaw: 0.614, brake: 0.087, slip: 0.439, wander: 0.078, shy: 0.894 }, // metachronal
        KindLife { space: 1.443, yaw: 0.448, brake: 0.116, slip: 0.602, wander: 0.102, shy: 0.484 }, // undulate
        KindLife { space: 1.171, yaw: 0.720, brake: 0.260, slip: 0.161, wander: 0.065, shy: 0.844 }, // flap
        KindLife { space: 0.880, yaw: 0.158, brake: 0.083, slip: 0.974, wander: 0.016, shy: 0.997 }, // spin
        KindLife { space: 1.280, yaw: 0.280, brake: 0.406, slip: 0.782, wander: 0.031, shy: 1.001 }, // hover
        KindLife { space: 1.024, yaw: 0.362, brake: 0.084, slip: 0.298, wander: 0.080, shy: 0.617 }, // helix
    ],
};

pub fn mulberry32(mut a: u32) -> impl FnMut() -> f64 {
    move || {
        a = a.wrapping_add(0x6D2B79F5);
        let mut tt = (a ^ (a >> 15)).wrapping_mul(1 | a);
        tt = tt.wrapping_add((tt ^ (tt >> 7)).wrapping_mul(61 | tt)) ^ tt;
        ((tt ^ (tt >> 14)) as f64) / 4_294_967_296.0
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SchoolStats {
    pub mean_nn: f64,
    pub min_nn: f64,
    pub overlap_frac: f64,
    pub align: f64,
    pub yaw_flips: f64,
    pub cell_entropy: f64,
    pub corner_frac: f64,
    pub mean_speed: f64,
    pub closest: f64,
}

#[cfg(test)]
pub fn score(s: &SchoolStats) -> f64 {
    let spread = (s.mean_nn / 0.13).tanh();
    let floor = (s.min_nn / 0.055).tanh();
    let close = (s.closest / 0.045).tanh();
    let overlap = (-s.overlap_frac * 10.0).exp();
    let align = s.align.clamp(0.0, 1.0);
    let flips = (-s.yaw_flips / 3.5).exp();
    let cover = (s.cell_entropy / (16.0f64.ln())).clamp(0.0, 1.0);
    let corner = (1.0 - s.corner_frac).clamp(0.0, 1.0);
    let alive = ((s.mean_speed - 0.002) / 0.012).clamp(0.0, 1.0);
    2.2 * spread
        + 2.6 * floor
        + 2.0 * close
        + 2.3 * overlap
        + 1.1 * align
        + 0.9 * flips
        + 1.6 * cover
        + 0.8 * corner
        + 0.6 * alive
}

#[cfg(test)]
pub struct GenLog {
    pub gen: u32,
    pub best: f64,
    pub mean: f64,
}

#[cfg(test)]
pub fn evolve<F>(gens: u32, seed: u32, mut eval: F) -> (LifeParams, Vec<GenLog>)
where
    F: FnMut(&LifeParams) -> f64,
{
    let mut rng = mulberry32(seed);
    let mut parent = LIFE;
    let mut parent_s = eval(&parent);
    let mut log = Vec::with_capacity(gens as usize);
    const LAMBDA: usize = 4;
    for gen in 0..gens {
        let mut scores = [0.0; LAMBDA];
        let mut kids = [parent; LAMBDA];
        let mut best_i = 0usize;
        let mut best_s = f64::NEG_INFINITY;
        let mut acc = 0.0;
        for i in 0..LAMBDA {
            kids[i] = parent.mutate(&mut rng);
            scores[i] = eval(&kids[i]);
            acc += scores[i];
            if scores[i] > best_s {
                best_s = scores[i];
                best_i = i;
            }
        }
        if best_s >= parent_s {
            parent = kids[best_i];
            parent_s = best_s;
        }
        log.push(GenLog {
            gen,
            best: parent_s,
            mean: acc / LAMBDA as f64,
        });
    }
    (parent, log)
}
