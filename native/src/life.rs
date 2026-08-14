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
            far: self.far.clamp(1.35, 3.60),
            push: self.push.clamp(0.50, 3.80),
            far_w: self.far_w.clamp(0.12, 1.40),
            gyre: self.gyre.clamp(0.0, 0.10),
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

/// 第二轮 100 代 ES（擦身适应度 13.04 → 13.59）。对头迫近才让，擦肩通过。
pub const LIFE: LifeParams = LifeParams {
    body: 0.086,
    near: 1.121,
    far: 1.927,
    push: 2.090,
    far_w: 0.204,
    gyre: 0.017,
    slide: 0.113,
    kinds: [
        KindLife { space: 0.948, yaw: 0.252, brake: 0.144, slip: 0.514, wander: 0.046, shy: 0.434 }, // jet
        KindLife { space: 0.934, yaw: 0.267, brake: 0.055, slip: 0.947, wander: 0.027, shy: 0.694 }, // ciliary
        KindLife { space: 1.250, yaw: 0.413, brake: 0.120, slip: 0.459, wander: 0.103, shy: 0.300 }, // metachronal
        KindLife { space: 1.707, yaw: 0.720, brake: 0.067, slip: 0.262, wander: 0.134, shy: 0.333 }, // undulate
        KindLife { space: 1.557, yaw: 0.551, brake: 0.104, slip: 0.316, wander: 0.065, shy: 0.387 }, // flap
        KindLife { space: 0.929, yaw: 0.126, brake: 0.041, slip: 1.000, wander: 0.019, shy: 0.860 }, // spin
        KindLife { space: 1.055, yaw: 0.142, brake: 0.502, slip: 0.720, wander: 0.027, shy: 0.578 }, // hover
        KindLife { space: 1.351, yaw: 0.409, brake: 0.099, slip: 0.193, wander: 0.078, shy: 0.300 }, // helix
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
    #[allow(dead_code)]
    pub corner_frac: f64,
    pub mean_speed: f64,
    pub closest: f64,
    pub graze_frac: f64,
    pub gyre_align: f64,
    pub cruise_ratio: f64,
    pub evade_frac: f64,
}

#[cfg(test)]
pub fn score(s: &SchoolStats) -> f64 {
    let nn_t = 1.0 - ((s.mean_nn - 0.155).abs() / 0.09).clamp(0.0, 1.0);
    let pile = (s.min_nn / 0.042).tanh();
    let shy = if s.closest > 0.13 {
        ((s.closest - 0.13) / 0.10).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let crush = if s.closest < 0.030 { 1.0 } else { 0.0 };
    let graze = (s.graze_frac / 0.035).clamp(0.0, 1.0);
    let overlap = (-s.overlap_frac * 12.0).exp();
    let align = s.align.clamp(0.0, 1.0);
    let flips = (-s.yaw_flips / 3.0).exp();
    let cover = (s.cell_entropy / (16.0f64.ln())).clamp(0.0, 1.0);
    let carousel = 1.0 - ((s.gyre_align - 0.32).abs() / 0.50).clamp(0.0, 1.0);
    let alive = s.cruise_ratio.clamp(0.0, 1.15).min(1.0);
    let evade = (1.0 - (s.evade_frac - 0.12).max(0.0) * 1.4).clamp(0.0, 1.0);
    2.4 * nn_t + 2.0 * pile - 1.8 * shy - 2.2 * crush + 1.9 * graze + 2.0 * overlap + 1.0 * align
        + 0.8 * flips
        + 1.3 * cover
        + 0.9 * carousel
        + 1.3 * alive
        + 0.7 * evade
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
