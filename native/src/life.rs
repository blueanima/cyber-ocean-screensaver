//! 群体生活：均匀占位、相遇退避、以及运动生物学演化搜索。

use crate::gait::GaitKind;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KindLife {
    /// 个体空间倍数
    pub space: f64,
    /// 退避转向 rad/s
    pub yaw: f64,
    /// 对头接近时减速
    pub brake: f64,
    /// 定向对齐增益（Couzin 定向圈；yaw 只负责让路）
    pub slip: f64,
    /// 航向漫游幅度
    pub wander: f64,
    /// 多早开始让（远场敏感）
    pub shy: f64,
    /// 巡游倍率（乘在步态 cruise 上，让 BL/s 能被分种搜索碰到）
    pub pace: f64,
    /// 定向圈半径 / 接触半径；圈外再加一圈弱吸引。
    pub zone: f64,
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
    #[allow(dead_code)]
    pub slide: f64,
    pub kinds: [KindLife; 17],
}

/// 花、轮、星无航向：不跟邻居拧头，分种搜索也不抖 yaw/wander。
pub const fn heading_is_trainable(ci: usize) -> bool {
    !matches!(ci, 8 | 9 | 13)
}

impl KindLife {
    #[cfg(test)]
    fn clamp(self) -> Self {
        Self {
            space: self.space.clamp(0.50, 2.40),
            yaw: self.yaw.clamp(0.08, 0.78),
            brake: self.brake.clamp(0.02, 0.62),
            slip: self.slip.clamp(0.05, 2.40),
            wander: self.wander.clamp(0.015, 0.18),
            shy: self.shy.clamp(0.28, 1.25),
            pace: self.pace.clamp(0.70, 2.80),
            zone: self.zone.clamp(1.60, 2.80),
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
            gyre: self.gyre.clamp(0.0, 0.18),
            slide: self.slide.clamp(0.06, 0.28),
            kinds,
        }
    }

    #[cfg(test)]
    pub(crate) const KIND_DIM: usize = 8;
    #[cfg(test)]
    pub(crate) const N_PARAM: usize = 7 + 17 * 8;
    pub const N_SPECIES: usize = 17;
    #[cfg(test)]
    pub const fn heading_trainable(ci: usize) -> bool {
        heading_is_trainable(ci)
    }

    #[cfg(test)]
    pub(crate) fn param_bounds(i: usize) -> (f64, f64) {
        if i < 7 {
            return match i {
                0 => (0.07, 0.17),
                1 => (0.90, 1.70),
                2 => (1.35, 3.60),
                3 => (0.50, 3.80),
                4 => (0.12, 1.40),
                5 => (0.0, 0.18),
                6 => (0.06, 0.28),
                _ => unreachable!(),
            };
        }
        match (i - 7) % Self::KIND_DIM {
            0 => (0.50, 2.40),
            1 => (0.08, 0.78),
            2 => (0.02, 0.62),
            3 => (0.05, 2.40),
            4 => (0.015, 0.18),
            5 => (0.28, 1.25),
            6 => (0.70, 2.80),
            7 => (1.60, 2.80),
            _ => unreachable!(),
        }
    }

    #[cfg(test)]
    pub(crate) fn param(&self, i: usize) -> f64 {
        if i < 7 {
            return match i {
                0 => self.body,
                1 => self.near,
                2 => self.far,
                3 => self.push,
                4 => self.far_w,
                5 => self.gyre,
                6 => self.slide,
                _ => unreachable!(),
            };
        }
        let j = i - 7;
        let k = &self.kinds[j / Self::KIND_DIM];
        match j % Self::KIND_DIM {
            0 => k.space,
            1 => k.yaw,
            2 => k.brake,
            3 => k.slip,
            4 => k.wander,
            5 => k.shy,
            6 => k.pace,
            7 => k.zone,
            _ => unreachable!(),
        }
    }

    #[cfg(test)]
    fn encode(&self) -> [f64; Self::N_PARAM] {
        let mut x = [0.0; Self::N_PARAM];
        for i in 0..Self::N_PARAM {
            let (lo, hi) = Self::param_bounds(i);
            let span = (hi - lo).max(1e-9);
            x[i] = ((self.param(i) - lo) / span).clamp(0.0, 1.0);
        }
        x
    }

    #[cfg(test)]
    pub(crate) fn set_param(&mut self, i: usize, v: f64) {
        if i < 7 {
            match i {
                0 => self.body = v,
                1 => self.near = v,
                2 => self.far = v,
                3 => self.push = v,
                4 => self.far_w = v,
                5 => self.gyre = v,
                6 => self.slide = v,
                _ => unreachable!(),
            }
            return;
        }
        let j = i - 7;
        let k = &mut self.kinds[j / Self::KIND_DIM];
        match j % Self::KIND_DIM {
            0 => k.space = v,
            1 => k.yaw = v,
            2 => k.brake = v,
            3 => k.slip = v,
            4 => k.wander = v,
            5 => k.shy = v,
            6 => k.pace = v,
            7 => k.zone = v,
            _ => unreachable!(),
        }
    }

    /// 只动 1–3 个参数，步长由 sigma 控制。
    #[cfg(test)]
    pub fn mutate(&self, rng: &mut impl FnMut() -> f64, sigma: f64) -> Self {
        let mut out = *self;
        let n_touch = 1 + (rng() * 3.0).floor() as usize;
        let mut picked = [usize::MAX; 3];
        let mut n = 0usize;
        let mut guard = 0u32;
        while n < n_touch && guard < 32 {
            guard += 1;
            let i = (rng() * Self::N_PARAM as f64).floor() as usize % Self::N_PARAM;
            if picked[..n].contains(&i) {
                continue;
            }
            picked[n] = i;
            n += 1;
            let u = rng() + rng() + rng() - 1.5;
            out.set_param(i, out.param(i) * (1.0 + sigma * u));
        }
        out.clamp()
    }

    /// 只抖动某一个数字生物的生活参数。辐射种不碰 yaw/wander。
    #[cfg(test)]
    pub fn mutate_species(&self, ci: usize, rng: &mut impl FnMut() -> f64, sigma: f64) -> Self {
        let mut out = *self;
        let ci = ci.min(Self::N_SPECIES - 1);
        let base = 7 + ci * Self::KIND_DIM;
        let choices: &[usize] = if heading_is_trainable(ci) {
            &[0, 1, 2, 3, 4, 5, 6, 7]
        } else {
            &[0, 2, 3, 5, 6, 7]
        };
        let n_touch = 1 + (rng() * 3.0).floor() as usize;
        let mut picked = [usize::MAX; 3];
        let mut n = 0usize;
        let mut guard = 0u32;
        while n < n_touch && guard < 32 {
            guard += 1;
            let k = choices[(rng() * choices.len() as f64).floor() as usize % choices.len()];
            if picked[..n].contains(&k) {
                continue;
            }
            picked[n] = k;
            n += 1;
            let i = base + k;
            let u = rng() + rng() + rng() - 1.5;
            out.set_param(i, out.param(i) * (1.0 + sigma * u));
        }
        out.clamp()
    }
}

#[allow(dead_code)]
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

#[allow(dead_code)]
pub fn kind_life(kind: GaitKind, life: &LifeParams) -> KindLife {
    // 无个体下标时，落到该步态的代表种。
    let ci = match kind {
        GaitKind::Jet => 3,
        GaitKind::Ciliary => 11,
        GaitKind::Metachronal => 14,
        GaitKind::Undulate => 12,
        GaitKind::Flap => 16,
        GaitKind::SpinDrift => 8,
        GaitKind::Hover => 6,
        GaitKind::Helix => 10,
    };
    species_life(ci, life)
}

pub fn species_life(ci: usize, life: &LifeParams) -> KindLife {
    life.kinds[ci.min(life.kinds.len() - 1)]
}

/// 每种数字生物独立生活参数（下标 = 物种 ci）。
/// 10 分钟分种对照后的场；分种训练只改对应行。
pub const LIFE: LifeParams = LifeParams {
    body: 0.170,
    near: 1.099,
    far: 1.779,
    push: 2.049,
    far_w: 0.163,
    gyre: 0.012,
    slide: 0.102,
    kinds: [
        KindLife { space: 0.851, yaw: 0.272, brake: 0.082, slip: 0.388, wander: 0.056, shy: 0.295, pace: 1.000, zone: 2.200 }, // fucan
        KindLife { space: 0.886, yaw: 0.302, brake: 0.080, slip: 0.391, wander: 0.057, shy: 0.320, pace: 1.000, zone: 2.200 }, // youyan
        KindLife { space: 1.523, yaw: 0.779, brake: 0.049, slip: 0.243, wander: 0.055, shy: 0.323, pace: 1.000, zone: 2.200 }, // jichong
        KindLife { space: 1.285, yaw: 0.245, brake: 0.109, slip: 0.321, wander: 0.039, shy: 0.354, pace: 1.000, zone: 2.200 }, // jelly
        KindLife { space: 1.417, yaw: 0.248, brake: 0.106, slip: 0.320, wander: 0.039, shy: 0.361, pace: 1.000, zone: 2.200 }, // nebula
        KindLife { space: 1.312, yaw: 0.241, brake: 0.101, slip: 0.301, wander: 0.038, shy: 0.372, pace: 1.000, zone: 2.200 }, // lantern
        KindLife { space: 0.838, yaw: 0.142, brake: 0.342, slip: 0.480, wander: 0.029, shy: 0.462, pace: 1.000, zone: 2.200 }, // feather
        KindLife { space: 1.304, yaw: 0.774, brake: 0.052, slip: 0.247, wander: 0.055, shy: 0.326, pace: 1.000, zone: 2.200 }, // tentacle
        KindLife { space: 0.801, yaw: 0.099, brake: 0.031, slip: 0.554, wander: 0.019, shy: 0.405, pace: 1.000, zone: 2.200 }, // flower6
        KindLife { space: 0.801, yaw: 0.099, brake: 0.031, slip: 0.554, wander: 0.019, shy: 0.405, pace: 1.000, zone: 2.200 }, // wheel
        KindLife { space: 0.800, yaw: 0.158, brake: 0.081, slip: 0.138, wander: 0.036, shy: 0.302, pace: 1.000, zone: 2.200 }, // spiral
        KindLife { space: 0.801, yaw: 0.299, brake: 0.050, slip: 0.419, wander: 0.028, shy: 0.436, pace: 1.000, zone: 2.200 }, // comb
        KindLife { space: 1.466, yaw: 0.778, brake: 0.049, slip: 0.250, wander: 0.056, shy: 0.322, pace: 1.000, zone: 2.200 }, // saweel
        KindLife { space: 0.801, yaw: 0.099, brake: 0.031, slip: 0.554, wander: 0.019, shy: 0.405, pace: 1.000, zone: 2.200 }, // star8
        KindLife { space: 0.800, yaw: 0.321, brake: 0.044, slip: 0.174, wander: 0.056, shy: 0.280, pace: 1.000, zone: 1.614 }, // shrimp
        KindLife { space: 2.100, yaw: 0.774, brake: 0.049, slip: 0.249, wander: 0.057, shy: 0.302, pace: 1.000, zone: 2.200 }, // vortex
        KindLife { space: 1.139, yaw: 0.253, brake: 0.090, slip: 0.241, wander: 0.052, shy: 0.371, pace: 1.000, zone: 2.200 }, // angel
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
    /// 全场最近邻 / 体长（日志用；打分按物种分项，见 `kinds`）
    pub mean_nnd_bl: f64,
    pub min_nnd_bl: f64,
    pub mean_abs_yaw: f64,
    pub sharp_frac: f64,
    pub polar: f64,
    pub mean_speed_bl: f64,
    pub kinds: [KindBio; 17],
}

/// 某一种数字生物在一场仿真里的对照统计。
#[cfg(test)]
#[derive(Clone, Copy, Debug, Default)]
pub struct KindBio {
    pub n: f64,
    pub nnd_bl: f64,
    pub yaw: f64,
    pub polar: f64,
    pub have_polar: bool,
    pub speed_bl: f64,
}

/// 灯鱼群游（仅细长群游种的文献锚，不是全场靶）。
#[cfg(test)]
#[allow(dead_code)]
pub const BIO_NND_BL: f64 = 0.99;
#[cfg(test)]
#[allow(dead_code)]
pub const BIO_POLAR: f64 = 0.98;
#[cfg(test)]
#[allow(dead_code)]
pub const BIO_YAW: f64 = 0.51;
#[cfg(test)]
#[allow(dead_code)]
pub const BIO_SHARP: f64 = 0.009;

/// 按数字生物独立对照。顺序同 `SPECIES`。出处 `data/ethology/taxa.json`。
#[cfg(test)]
pub struct KindTarget {
    pub nnd: f64,
    pub nnd_scale: f64,
    pub yaw: f64,
    pub yaw_scale: f64,
    pub polar: f64,
    pub speed: f64,
    pub speed_scale: f64,
    pub speed_floor: f64,
    pub w_nnd: f64,
    pub w_yaw: f64,
    pub w_polar: f64,
    pub w_spd: f64,
}

#[cfg(test)]
const fn kt(
    nnd: f64,
    nnd_scale: f64,
    yaw: f64,
    yaw_scale: f64,
    polar: f64,
    speed: f64,
    speed_scale: f64,
    speed_floor: f64,
    w_nnd: f64,
    w_yaw: f64,
    w_polar: f64,
    w_spd: f64,
) -> KindTarget {
    KindTarget {
        nnd,
        nnd_scale,
        yaw,
        yaw_scale,
        polar,
        speed,
        speed_scale,
        speed_floor,
        w_nnd,
        w_yaw,
        w_polar,
        w_spd,
    }
}

#[cfg(test)]
pub const SPECIES_BIO: [KindTarget; 17] = [
    // fucan ← 浮蚕科 Tomopteris：远洋游泳多毛类，不密集成群
    kt(2.80, 1.80, 0.14, 0.30, 0.32, 1.20, 1.20, 0.08, 1.1, 1.1, 0.4, 1.2),
    // youyan ← 海蟑螂/端足类：附肢划水，松散聚集
    kt(2.00, 1.50, 0.12, 0.28, 0.50, 1.50, 1.40, 0.08, 1.2, 1.1, 0.8, 1.2),
    // jichong ← Nereis 等游走多毛类：~1.3 BL/s，几乎不集群
    kt(3.00, 2.00, 0.18, 0.32, 0.28, 1.30, 1.20, 0.08, 1.1, 1.2, 0.3, 1.3),
    // jelly ← Aurelia aurita：~0.34 BL/s，水华聚集但不鱼群极化
    kt(2.20, 1.80, 0.22, 0.35, 0.38, 0.35, 0.50, 0.04, 1.0, 1.2, 0.8, 1.4),
    // nebula ← Cyanea/Chrysaora：更大更慢，更散
    kt(2.50, 1.90, 0.20, 0.35, 0.32, 0.28, 0.45, 0.04, 1.0, 1.1, 0.6, 1.4),
    // lantern ← Aequorea 等花水母：脉动略快
    kt(2.00, 1.70, 0.25, 0.35, 0.40, 0.45, 0.55, 0.04, 1.0, 1.2, 0.7, 1.4),
    // feather ← 缨鳃虫：滤食悬停
    kt(2.40, 1.80, 0.06, 0.30, 0.15, 0.12, 0.40, 0.02, 1.0, 0.3, 0.0, 1.2),
    // tentacle ← 浮蚕/须虫：单体远洋
    kt(2.80, 1.90, 0.16, 0.32, 0.26, 1.10, 1.20, 0.08, 1.1, 1.1, 0.3, 1.2),
    // flower6 ← 银币水母/僧帽水母漂浮体：辐射对称无航向
    kt(3.00, 2.00, 0.00, 1.00, 0.00, 0.12, 0.40, 0.02, 1.0, 0.0, 0.0, 0.8),
    // wheel ← 轮虫 Brachionus：微体漂游
    kt(2.50, 1.80, 0.00, 1.00, 0.00, 0.18, 0.40, 0.02, 1.0, 0.0, 0.0, 0.8),
    // spiral ← 螺蛸/Limacina：螺旋/壳轴前进
    kt(3.20, 2.00, 0.20, 0.35, 0.30, 0.60, 0.80, 0.12, 0.8, 0.8, 0.4, 1.0),
    // comb ← Mnemiopsis：觅食 ~0.1 BL/s，机动但不集群
    kt(3.20, 2.20, 0.18, 0.35, 0.22, 0.12, 0.40, 0.04, 0.8, 1.0, 0.3, 1.4),
    // saweel ← Anguilla：巡游 0.5–2 BL/s，头航向稳
    kt(3.00, 2.00, 0.14, 0.30, 0.40, 1.40, 1.20, 0.08, 1.1, 1.2, 0.5, 1.3),
    // star8 ← 海星/八放珊瑚：辐射爬/漂
    kt(2.80, 2.00, 0.00, 1.00, 0.00, 0.10, 0.40, 0.02, 1.0, 0.0, 0.0, 0.8),
    // shrimp ← Euphausia superba：NND~1 BL，极化 0.78，~2 BL/s
    // 极化主、转向只罚乱拧（过稳不扣）。
    kt(1.05, 1.10, 0.10, 0.22, 0.78, 2.00, 1.60, 0.08, 1.0, 0.20, 2.8, 1.2),
    // vortex ← 涡虫：慢游，松散
    kt(2.60, 1.80, 0.15, 0.32, 0.25, 0.80, 0.90, 0.08, 1.0, 1.0, 0.3, 1.1),
    // angel ← Clione limacina：扑翼悬停/慢游
    kt(3.00, 2.00, 0.12, 0.30, 0.28, 0.70, 0.80, 0.12, 0.8, 0.8, 0.3, 1.2),
];

#[cfg(test)]
/// 仿真世界速度上限约 0.10；折合体长后游泳种达不到文献 1–2 BL/s。
/// 游速项对到这个能达上限，避免永远打 0。
#[cfg(test)]
const SCORE_VBL_CAP: f64 = 0.85;

#[cfg(test)]
/// 松散种锁 space 上限，避免顶到 2.40 刷 nnd。磷虾更紧。
pub fn species_space_cap(ci: usize) -> f64 {
    match ci {
        14 => 1.15,
        8 | 9 | 13 => 1.35,
        0 | 2 | 6 | 7 | 10 | 11 | 12 | 15 | 16 => 1.55,
        _ => 2.10,
    }
}

#[cfg(test)]
fn space_hits_cap(space: f64, cap: f64) -> bool {
    space >= cap - 0.02
}

#[cfg(test)]
fn space_bound_tax(space: f64, cap: f64) -> f64 {
    if space_hits_cap(space, cap) {
        1.0
    } else {
        0.0
    }
}

#[cfg(test)]
fn kind_match(k: &KindBio, t: &KindTarget, space: f64, cap: f64) -> f64 {
    if k.n < 4.0 {
        return 0.0;
    }
    let nnd_err = k.nnd_bl - t.nnd;
    // space 触顶不再给 nnd 分。超过靶的间距按过散扣，比偏近更严。
    let nnd = if space_hits_cap(space, cap) {
        0.0
    } else if nnd_err > 0.0 {
        (1.0 - (nnd_err / (t.nnd_scale * 0.50).max(1e-6)).clamp(0.0, 1.0)).max(0.0)
    } else {
        1.0 - (nnd_err.abs() / t.nnd_scale.max(1e-6)).clamp(0.0, 1.0)
    };
    // 高极化种：过稳不扣，只罚乱拧。避免 CMA 把 yaw 压扁、对齐跟着死。
    let yaw = if t.w_polar >= t.w_yaw {
        if k.yaw > t.yaw + t.yaw_scale {
            (1.0 - (k.yaw - t.yaw - t.yaw_scale) / t.yaw_scale.max(1e-6)).clamp(0.0, 1.0)
        } else {
            1.0
        }
    } else {
        1.0 - ((k.yaw - t.yaw).abs() / t.yaw_scale.max(1e-6)).clamp(0.0, 1.0)
    };
    let spd_tgt = t.speed.min(SCORE_VBL_CAP);
    let spd_scale = t.speed_scale.min((spd_tgt * 0.70).max(0.12));
    let mut spd = 1.0 - ((k.speed_bl - spd_tgt).abs() / spd_scale.max(1e-6)).clamp(0.0, 1.0);
    if k.speed_bl < t.speed_floor {
        spd *= (k.speed_bl / t.speed_floor.max(1e-6)).clamp(0.0, 1.0);
    }
    let mut w = t.w_nnd + t.w_yaw + t.w_spd;
    let mut s = t.w_nnd * nnd + t.w_yaw * yaw + t.w_spd * spd;
    if k.have_polar && t.w_polar > 0.0 {
        let polar = (1.0 - (k.polar - t.polar).abs() / 0.40).clamp(0.0, 1.0);
        s += t.w_polar * polar;
        w += t.w_polar;
    }
    if w <= 1e-6 {
        0.0
    } else {
        s / w
    }
}

#[cfg(test)]
pub fn score_species(s: &SchoolStats, ci: usize, life: &LifeParams) -> f64 {
    let ci = ci.min(16);
    let t = &SPECIES_BIO[ci];
    let k = if s.kinds[ci].n >= 4.0 {
        s.kinds[ci]
    } else {
        KindBio {
            n: 32.0,
            nnd_bl: s.mean_nnd_bl,
            yaw: s.mean_abs_yaw,
            polar: s.polar,
            have_polar: t.w_polar > 0.0 && s.polar > 0.0,
            speed_bl: s.mean_speed_bl,
        }
    };
    let space = life.kinds[ci].space;
    let cap = species_space_cap(ci);
    let pile = if s.min_nnd_bl >= 0.70 {
        1.0
    } else {
        (s.min_nnd_bl / 0.70).clamp(0.0, 1.0)
    };
    let crush = if s.min_nnd_bl < 0.55 { 1.0 } else { 0.0 };
    let overlap = (-s.overlap_frac * 18.0).exp();
    let alive = s.cruise_ratio.clamp(0.0, 1.15).min(1.0);
    3.6 * kind_match(&k, t, space, cap) + 1.4 * pile - 2.6 * crush + 2.0 * overlap + 0.8 * alive
        - 1.8 * space_bound_tax(space, cap)
}

#[cfg(test)]
pub fn score(s: &SchoolStats, life: &LifeParams) -> f64 {
    let pile = if s.min_nnd_bl >= 0.70 {
        1.0
    } else {
        (s.min_nnd_bl / 0.70).clamp(0.0, 1.0)
    };
    let crush = if s.min_nnd_bl < 0.40 { 1.0 } else { 0.0 };
    let overlap = (-s.overlap_frac * 12.0).exp();
    let flips = (-s.yaw_flips / 3.0).exp();
    let cover = (s.cell_entropy / (16.0f64.ln())).clamp(0.0, 1.0);
    let alive = s.cruise_ratio.clamp(0.0, 1.15).min(1.0);
    let evade = (1.0 - (s.evade_frac - 0.12).max(0.0) * 1.4).clamp(0.0, 1.0);
    let mut bio = 0.0;
    let mut nw = 0.0;
    let mut tax = 0.0;
    for (i, (k, t)) in s.kinds.iter().zip(SPECIES_BIO.iter()).enumerate() {
        if k.n < 4.0 {
            continue;
        }
        let cap = species_space_cap(i);
        bio += kind_match(k, t, life.kinds[i].space, cap);
        tax += space_bound_tax(life.kinds[i].space, cap);
        nw += 1.0;
    }
    let bio = if nw > 0.0 { bio / nw } else { 0.0 };
    3.4 * bio + 1.4 * pile - 2.2 * crush + 1.6 * overlap + 0.7 * flips + 0.25 * cover + 1.0 * alive
        + 0.5 * evade
        - 0.45 * tax
}

#[cfg(test)]
pub struct GenLog {
    pub gen: u32,
    pub best: f64,
    pub mean: f64,
}

#[cfg(test)]
pub fn evolve<F>(gens: u32, seed: u32, eval: F) -> (LifeParams, Vec<GenLog>)
where
    F: Fn(&LifeParams) -> f64 + Sync,
{
    let (best, _, log) = evolve_from(LIFE, gens, seed, 0.03, eval);
    (best, log)
}

#[cfg(test)]
pub fn evolve_from<F>(
    start: LifeParams,
    gens: u32,
    seed: u32,
    mut sigma: f64,
    eval: F,
) -> (LifeParams, f64, Vec<GenLog>)
where
    F: Fn(&LifeParams) -> f64 + Sync,
{
    let mut rng = mulberry32(seed);
    let mut parent = start;
    let mut parent_s = eval(&parent);
    let mut log = Vec::with_capacity(gens as usize);
    let mut stale = 0u32;
    sigma = sigma.clamp(0.008, 0.16);
    // 铺开到所有核，避免单核 100% 把封装温度顶上去。
    let lambda = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .clamp(4, 16);
    for gen in 0..gens {
        let mut kids = vec![parent; lambda];
        for kid in kids.iter_mut() {
            *kid = parent.mutate(&mut rng, sigma);
        }
        let scores: Vec<f64> = std::thread::scope(|scope| {
            let hs: Vec<_> = kids.iter().map(|kid| scope.spawn(|| eval(kid))).collect();
            hs.into_iter()
                .map(|h| h.join().expect("eval"))
                .collect()
        });
        let mut best_i = 0usize;
        let mut best_s = f64::NEG_INFINITY;
        let mut acc = 0.0;
        for (i, &s) in scores.iter().enumerate() {
            acc += s;
            if s > best_s {
                best_s = s;
                best_i = i;
            }
        }
        if best_s > parent_s + 1e-4 {
            parent = kids[best_i];
            parent_s = best_s;
            sigma = (sigma * 0.82).max(0.008);
            stale = 0;
        } else {
            stale += 1;
            if stale >= 2 {
                sigma = (sigma * 1.28).min(0.16);
                stale = 0;
            }
        }
        log.push(GenLog {
            gen,
            best: parent_s,
            mean: acc / lambda as f64,
        });
    }
    (parent, sigma, log)
}

#[cfg(test)]
#[derive(Clone, Copy, Debug)]
pub struct SpeciesSearch {
    pub lock_space: bool,
    pub align_only: bool,
}

#[cfg(test)]
impl SpeciesSearch {
    pub fn dims(self, ci: usize) -> Vec<usize> {
        if self.align_only {
            return if heading_is_trainable(ci) {
                vec![3, 7]
            } else {
                vec![7]
            };
        }
        let mut out = Vec::with_capacity(7);
        for k in 0..7 {
            if k == 0 && self.lock_space {
                continue;
            }
            if (k == 1 || k == 4) && !heading_is_trainable(ci) {
                continue;
            }
            out.push(k);
        }
        out
    }
}

#[cfg(test)]
fn species_kind_bounds(ci: usize, k: usize) -> (f64, f64) {
    if k == 0 {
        (0.50, species_space_cap(ci))
    } else {
        LifeParams::param_bounds(7 + k)
    }
}

#[cfg(test)]
fn enc_log(v: f64, lo: f64, hi: f64) -> f64 {
    let lo = lo.max(1e-12);
    let hi = hi.max(lo * 1.0001);
    ((v.max(lo).min(hi).ln() - lo.ln()) / (hi.ln() - lo.ln())).clamp(0.0, 1.0)
}

#[cfg(test)]
fn dec_log(u: f64, lo: f64, hi: f64) -> f64 {
    let lo = lo.max(1e-12);
    let hi = hi.max(lo * 1.0001);
    (lo.ln() + u.clamp(0.0, 1.0) * (hi.ln() - lo.ln())).exp()
}

#[cfg(test)]
fn gauss(rng: &mut impl FnMut() -> f64) -> f64 {
    let u1 = rng().max(1e-12);
    let u2 = rng();
    (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
}

#[cfg(test)]
fn jacobi_spd(c_in: &[f64], n: usize) -> (Vec<f64>, Vec<f64>) {
    let mut a = c_in.to_vec();
    let mut v = vec![0.0; n * n];
    for i in 0..n {
        v[i * n + i] = 1.0;
    }
    for _ in 0..64 {
        let mut max = 0.0;
        let mut p = 0usize;
        let mut q = 1usize;
        for i in 0..n {
            for j in i + 1..n {
                let val = a[i * n + j].abs();
                if val > max {
                    max = val;
                    p = i;
                    q = j;
                }
            }
        }
        if max < 1e-15 {
            break;
        }
        let app = a[p * n + p];
        let aqq = a[q * n + q];
        let apq = a[p * n + q];
        let tau = (aqq - app) / (2.0 * apq);
        let t = if tau >= 0.0 {
            1.0 / (tau + (1.0 + tau * tau).sqrt())
        } else {
            -1.0 / (-tau + (1.0 + tau * tau).sqrt())
        };
        let c = 1.0 / (1.0 + t * t).sqrt();
        let s = t * c;
        for i in 0..n {
            if i == p || i == q {
                continue;
            }
            let aip = a[i * n + p];
            let aiq = a[i * n + q];
            a[i * n + p] = c * aip - s * aiq;
            a[p * n + i] = a[i * n + p];
            a[i * n + q] = s * aip + c * aiq;
            a[q * n + i] = a[i * n + q];
        }
        a[p * n + p] = c * c * app - 2.0 * s * c * apq + s * s * aqq;
        a[q * n + q] = s * s * app + 2.0 * s * c * apq + c * c * aqq;
        a[p * n + q] = 0.0;
        a[q * n + p] = 0.0;
        for i in 0..n {
            let vip = v[i * n + p];
            let viq = v[i * n + q];
            v[i * n + p] = c * vip - s * viq;
            v[i * n + q] = s * vip + c * viq;
        }
    }
    let mut d = vec![0.0; n];
    for i in 0..n {
        d[i] = a[i * n + i].max(1e-16).sqrt();
    }
    (v, d)
}

#[cfg(test)]
pub(crate) const CMA_SIGMA0: f64 = 0.38;
#[cfg(test)]
const CMA_SIGMA_MIN: f64 = 0.045;
#[cfg(test)]
const CMA_SIGMA_MAX: f64 = 0.75;
#[cfg(test)]
const CMA_PEN: f64 = 2.0;
#[cfg(test)]
const CMA_DAMPS_SCALE: f64 = 0.62;
#[cfg(test)]
const CMA_Y_CLIP: f64 = 4.0;

#[cfg(test)]
pub struct SpeciesCma {
    pub dims: Vec<usize>,
    pub m: Vec<f64>,
    pub sigma: f64,
    pub restarts: u32,
    sigma0: f64,
    stale: u32,
    c: Vec<f64>,
    pc: Vec<f64>,
    ps: Vec<f64>,
    b: Vec<f64>,
    d: Vec<f64>,
    count: u32,
}

#[cfg(test)]
impl SpeciesCma {
    pub fn from_life(p: &LifeParams, ci: usize, search: SpeciesSearch, sigma: f64) -> Self {
        let dims = search.dims(ci);
        let n = dims.len().max(1);
        let mut m = vec![0.5; n];
        for (i, &k) in dims.iter().enumerate() {
            let (lo, hi) = species_kind_bounds(ci, k);
            m[i] = enc_log(p.param(7 + ci * LifeParams::KIND_DIM + k), lo, hi);
        }
        let mut c = vec![0.0; n * n];
        let mut b = vec![0.0; n * n];
        let d = vec![1.0; n];
        for i in 0..n {
            c[i * n + i] = 1.0;
            b[i * n + i] = 1.0;
        }
        let sigma = sigma.clamp(0.08, CMA_SIGMA_MAX);
        Self {
            dims,
            m,
            sigma,
            restarts: 0,
            sigma0: sigma,
            stale: 0,
            c,
            pc: vec![0.0; n],
            ps: vec![0.0; n],
            b,
            d,
            count: 0,
        }
    }

    fn reset_cov(&mut self) {
        let n = self.m.len();
        self.c = vec![0.0; n * n];
        self.b = vec![0.0; n * n];
        self.d = vec![1.0; n];
        for i in 0..n {
            self.c[i * n + i] = 1.0;
            self.b[i * n + i] = 1.0;
        }
        self.pc = vec![0.0; n];
        self.ps = vec![0.0; n];
        self.count = 0;
    }

    fn ipop(&mut self, rng: &mut impl FnMut() -> f64) {
        self.restarts += 1;
        let n = self.m.len();
        if self.restarts % 2 == 0 {
            for i in 0..n {
                self.m[i] = (0.35 * self.m[i].clamp(0.0, 1.0) + 0.65 * rng()).clamp(0.0, 1.0);
            }
        } else {
            for i in 0..n {
                self.m[i] = self.m[i].clamp(0.0, 1.0);
            }
        }
        self.reset_cov();
        let grow = 1.7f64.powi(self.restarts.min(5) as i32);
        self.sigma = (self.sigma0.max(0.22) * grow).clamp(0.20, CMA_SIGMA_MAX);
        self.stale = 0;
    }

    fn apply(&self, template: LifeParams, ci: usize, x: &[f64]) -> LifeParams {
        let mut p = template;
        let space0 = p.kinds[ci].space;
        for (i, &k) in self.dims.iter().enumerate() {
            let (lo, hi) = species_kind_bounds(ci, k);
            p.set_param(7 + ci * LifeParams::KIND_DIM + k, dec_log(x[i], lo, hi));
        }
        p = p.clamp();
        p.kinds[ci].space = p.kinds[ci].space.min(species_space_cap(ci));
        if !self.dims.contains(&0) {
            p.kinds[ci].space = space0;
        }
        p
    }

    pub fn run<F>(
        &mut self,
        start: LifeParams,
        ci: usize,
        gens: u32,
        seed: u32,
        eval: F,
    ) -> (LifeParams, Vec<GenLog>)
    where
        F: Fn(&LifeParams) -> f64 + Sync,
    {
        let n = self.m.len();
        if n == 0 || self.dims.is_empty() {
            return (start, Vec::new());
        }
        let mut rng = mulberry32(seed);
        let cores = std::thread::available_parallelism()
            .map(|k| k.get())
            .unwrap_or(4)
            .clamp(4, 16);
        let lambda0 = (cores * 3 / 2).clamp(12, 20);
        let lambda = ((lambda0 as f64) * (1.0 + 0.35 * self.restarts.min(4) as f64)).round()
            as usize;
        let lambda = lambda.clamp(12, 28);
        let n_inject = 2.min(lambda / 4).max(1);
        let mu = (lambda / 2).max(1);
        let mut w = vec![0.0; mu];
        let mut wsum = 0.0;
        for i in 0..mu {
            w[i] = ((mu as f64) + 0.5).ln() - ((i + 1) as f64).ln();
            wsum += w[i];
        }
        for wi in w.iter_mut() {
            *wi /= wsum;
        }
        let mueff = 1.0 / w.iter().map(|wi| wi * wi).sum::<f64>();
        let nf = n as f64;
        let cc = (4.0 + mueff / nf) / (nf + 4.0 + 2.0 * mueff / nf);
        let cs = (mueff + 2.0) / (nf + mueff + 5.0);
        let c1 = 2.0 / ((nf + 1.3).powi(2) + mueff);
        let cmu = (1.0 - c1).min(
            2.0 * (mueff - 2.0 + 1.0 / mueff) / ((nf + 2.0).powi(2) + mueff),
        );
        let damps = CMA_DAMPS_SCALE
            * (1.0 + 2.0 * (0.0f64).max(((mueff - 1.0) / (nf + 1.0)).sqrt() - 1.0) + cs);
        let chi_n = nf.sqrt() * (1.0 - 1.0 / (4.0 * nf) + 1.0 / (21.0 * nf * nf));
        let pen_w = CMA_PEN;

        let mut best_p = start;
        let mut best_s = eval(&start);
        let mut prev_best = best_s;
        let mut log = Vec::with_capacity(gens as usize);

        for gen in 0..gens {
            let (b, d) = jacobi_spd(&self.c, n);
            self.b = b;
            self.d = d;
            let mut zs = vec![vec![0.0; n]; lambda];
            let mut ys = vec![vec![0.0; n]; lambda];
            let mut xs = vec![vec![0.0; n]; lambda];
            let mut kids = vec![start; lambda];
            let mut pens = vec![0.0; lambda];
            for k in 0..lambda {
                let inject = k >= lambda - n_inject;
                if inject {
                    let far = k == lambda - 1;
                    for i in 0..n {
                        zs[k][i] = 0.0;
                        xs[k][i] = if far {
                            if self.m[i] < 0.5 {
                                0.88 + 0.12 * rng()
                            } else {
                                0.12 * rng()
                            }
                        } else {
                            rng()
                        };
                        ys[k][i] = ((xs[k][i] - self.m[i]) / self.sigma.max(1e-6))
                            .clamp(-CMA_Y_CLIP, CMA_Y_CLIP);
                    }
                } else {
                    for i in 0..n {
                        zs[k][i] = gauss(&mut rng);
                    }
                    for i in 0..n {
                        let mut acc = 0.0;
                        for j in 0..n {
                            acc += self.b[i * n + j] * self.d[j] * zs[k][j];
                        }
                        ys[k][i] = acc;
                        xs[k][i] = self.m[i] + self.sigma * acc;
                    }
                }
                let mut feas = xs[k].clone();
                let mut pen = 0.0;
                for i in 0..n {
                    let clip = feas[i].clamp(0.0, 1.0);
                    pen += (feas[i] - clip) * (feas[i] - clip);
                    feas[i] = clip;
                }
                pens[k] = pen_w * pen;
                kids[k] = self.apply(start, ci, &feas);
            }
            let raw: Vec<f64> = std::thread::scope(|scope| {
                let hs: Vec<_> = kids.iter().map(|kid| scope.spawn(|| eval(kid))).collect();
                hs.into_iter()
                    .map(|h| h.join().expect("eval"))
                    .collect()
            });
            let scores: Vec<f64> = raw.iter().zip(pens.iter()).map(|(s, p)| s - p).collect();
            let mut order: Vec<usize> = (0..lambda).collect();
            order.sort_by(|&i, &j| {
                scores[j]
                    .partial_cmp(&scores[i])
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let mut acc = 0.0;
            for (k, &s) in raw.iter().enumerate() {
                acc += s;
                if s > best_s {
                    best_s = s;
                    best_p = kids[k];
                }
            }

            let mut yw = vec![0.0; n];
            let mut zw = vec![0.0; n];
            for (r, &k) in order.iter().take(mu).enumerate() {
                for i in 0..n {
                    yw[i] += w[r] * ys[k][i];
                    zw[i] += w[r] * zs[k][i];
                }
            }
            for i in 0..n {
                self.m[i] = (self.m[i] + self.sigma * yw[i]).clamp(-0.35, 1.35);
            }
            let mut ps_n = 0.0;
            for i in 0..n {
                let mut bz = 0.0;
                for j in 0..n {
                    bz += self.b[i * n + j] * zw[j];
                }
                self.ps[i] = (1.0 - cs) * self.ps[i] + (cs * (2.0 - cs) * mueff).sqrt() * bz;
                ps_n += self.ps[i] * self.ps[i];
            }
            ps_n = ps_n.sqrt();
            self.count += lambda as u32;
            let hsig = if ps_n
                / (1.0 - (1.0 - cs).powi(2).powf(self.count as f64 / lambda as f64)).sqrt()
                / chi_n
                < 1.4 + 2.0 / (nf + 1.0)
            {
                1.0
            } else {
                0.0
            };
            for i in 0..n {
                self.pc[i] = (1.0 - cc) * self.pc[i]
                    + hsig * (cc * (2.0 - cc) * mueff).sqrt() * yw[i];
            }
            let ccov = 1.0 - c1 - cmu;
            let mut cnew = vec![0.0; n * n];
            for i in 0..n {
                for j in 0..n {
                    cnew[i * n + j] = ccov * self.c[i * n + j]
                        + c1
                            * (self.pc[i] * self.pc[j]
                                + (1.0 - hsig) * cc * (2.0 - cc) * self.c[i * n + j]);
                }
            }
            for (r, &k) in order.iter().take(mu).enumerate() {
                for i in 0..n {
                    for j in 0..n {
                        cnew[i * n + j] += cmu * w[r] * ys[k][i] * ys[k][j];
                    }
                }
            }
            for i in 0..n {
                for j in 0..i {
                    let s = 0.5 * (cnew[i * n + j] + cnew[j * n + i]);
                    cnew[i * n + j] = s;
                    cnew[j * n + i] = s;
                }
                cnew[i * n + i] = cnew[i * n + i].max(1e-12);
            }
            self.c = cnew;
            self.sigma = (self.sigma * (cs / damps * (ps_n / chi_n - 1.0)).exp())
                .clamp(CMA_SIGMA_MIN, CMA_SIGMA_MAX);
            if best_s > prev_best + 1e-5 {
                self.stale = 0;
                prev_best = best_s;
            } else {
                self.stale += 1;
            }
            if (self.sigma <= CMA_SIGMA_MIN + 0.004 && self.stale >= 3) || self.stale >= 8 {
                self.ipop(&mut rng);
            }
            log.push(GenLog {
                gen,
                best: best_s,
                mean: acc / lambda as f64,
            });
        }
        (best_p, log)
    }
}

#[cfg(test)]
#[allow(dead_code)]
pub fn evolve_species<F>(
    start: LifeParams,
    ci: usize,
    gens: u32,
    seed: u32,
    sigma: f64,
    eval: F,
) -> (LifeParams, f64, Vec<GenLog>)
where
    F: Fn(&LifeParams) -> f64 + Sync,
{
    evolve_species_cfg(
        start,
        ci,
        gens,
        seed,
        sigma,
        SpeciesSearch {
            lock_space: false,
            align_only: false,
        },
        eval,
    )
}

#[cfg(test)]
pub fn evolve_species_cfg<F>(
    start: LifeParams,
    ci: usize,
    gens: u32,
    seed: u32,
    sigma: f64,
    search: SpeciesSearch,
    eval: F,
) -> (LifeParams, f64, Vec<GenLog>)
where
    F: Fn(&LifeParams) -> f64 + Sync,
{
    let mut cma = SpeciesCma::from_life(&start, ci, search, sigma);
    let (best, log) = cma.run(start, ci, gens, seed, eval);
    (best, cma.sigma, log)
}

/// 线性岭回归代理：用已评估的 (参数, 分数) 拟合，再沿权重大的维提议。
#[cfg(test)]
pub struct Surrogate {
    xs: Vec<[f64; LifeParams::N_PARAM]>,
    ys: Vec<f64>,
    w: [f64; LifeParams::N_PARAM],
    b: f64,
    ready: bool,
}

#[cfg(test)]
#[allow(dead_code)]
impl Surrogate {
    const CAP: usize = 480;
    const MIN: usize = 24;

    pub fn new() -> Self {
        Self {
            xs: Vec::new(),
            ys: Vec::new(),
            w: [0.0; LifeParams::N_PARAM],
            b: 0.0,
            ready: false,
        }
    }

    pub fn len(&self) -> usize {
        self.xs.len()
    }

    pub fn push(&mut self, p: &LifeParams, y: f64) {
        if !y.is_finite() {
            return;
        }
        self.xs.push(p.encode());
        self.ys.push(y);
        if self.xs.len() > Self::CAP {
            let drop = self.xs.len() - Self::CAP;
            self.xs.drain(..drop);
            self.ys.drain(..drop);
        }
        self.ready = false;
    }

    pub fn fit(&mut self) -> bool {
        let n = self.xs.len();
        if n < Self::MIN {
            self.ready = false;
            return false;
        }
        let mean = self.ys.iter().copied().sum::<f64>() / n as f64;
        let mut xtx = [[0.0; LifeParams::N_PARAM]; LifeParams::N_PARAM];
        let mut xty = [0.0; LifeParams::N_PARAM];
        for (x, &y) in self.xs.iter().zip(self.ys.iter()) {
            let dy = y - mean;
            for i in 0..LifeParams::N_PARAM {
                xty[i] += x[i] * dy;
                for j in i..LifeParams::N_PARAM {
                    xtx[i][j] += x[i] * x[j];
                }
            }
        }
        for i in 0..LifeParams::N_PARAM {
            for j in 0..i {
                xtx[i][j] = xtx[j][i];
            }
            xtx[i][i] += 0.35 * n as f64;
        }
        if !solve_linear(&mut xtx, &mut xty) {
            self.ready = false;
            return false;
        }
        self.w = xty;
        self.b = mean;
        self.ready = true;
        true
    }

    pub fn predict(&self, p: &LifeParams) -> f64 {
        if !self.ready {
            return f64::NEG_INFINITY;
        }
        let x = p.encode();
        self.w
            .iter()
            .zip(x.iter())
            .map(|(w, xi)| w * xi)
            .sum::<f64>()
            + self.b
    }

    pub fn propose(
        &self,
        champ: &LifeParams,
        rng: &mut impl FnMut() -> f64,
        sigma: f64,
        n_try: usize,
    ) -> Option<LifeParams> {
        if !self.ready {
            return None;
        }
        let mut best_p: Option<LifeParams> = None;
        let mut best_y = self.predict(champ);
        let grad = self.grad_step(champ, sigma);
        let yg = self.predict(&grad);
        if yg > best_y {
            best_y = yg;
            best_p = Some(grad);
        }
        for _ in 0..n_try {
            let p = self.biased_mutate(champ, rng, sigma);
            let y = self.predict(&p);
            if y > best_y {
                best_y = y;
                best_p = Some(p);
            }
        }
        best_p
    }

    fn grad_step(&self, champ: &LifeParams, sigma: f64) -> LifeParams {
        let mut idx: Vec<usize> = (0..LifeParams::N_PARAM).collect();
        idx.sort_by(|a, b| {
            self.w[*b]
                .abs()
                .partial_cmp(&self.w[*a].abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let mut p = *champ;
        for &i in idx.iter().take(3) {
            let (lo, hi) = LifeParams::param_bounds(i);
            let step = (hi - lo) * sigma * 0.8 * self.w[i].signum();
            p.set_param(i, p.param(i) + step);
        }
        p.clamp()
    }

    fn pick(&self, rng: &mut impl FnMut() -> f64) -> usize {
        let mut tot = 0.0;
        for i in 0..LifeParams::N_PARAM {
            tot += self.w[i].abs() + 0.04;
        }
        let mut u = rng() * tot;
        for i in 0..LifeParams::N_PARAM {
            u -= self.w[i].abs() + 0.04;
            if u <= 0.0 {
                return i;
            }
        }
        LifeParams::N_PARAM - 1
    }

    fn biased_mutate(
        &self,
        champ: &LifeParams,
        rng: &mut impl FnMut() -> f64,
        sigma: f64,
    ) -> LifeParams {
        let mut out = *champ;
        let n_touch = 1 + (rng() * 3.0).floor() as usize;
        let mut picked = [usize::MAX; 3];
        let mut n = 0usize;
        let mut guard = 0u32;
        while n < n_touch && guard < 48 {
            guard += 1;
            let i = self.pick(rng);
            if picked[..n].contains(&i) {
                continue;
            }
            picked[n] = i;
            n += 1;
            let u = rng() + rng() + rng() - 1.5;
            out.set_param(i, out.param(i) * (1.0 + sigma * u));
        }
        out.clamp()
    }
}

#[cfg(test)]
fn solve_linear(a: &mut [[f64; LifeParams::N_PARAM]; LifeParams::N_PARAM], b: &mut [f64; LifeParams::N_PARAM]) -> bool {
    const D: usize = LifeParams::N_PARAM;
    for k in 0..D {
        let mut piv = k;
        let mut best = a[k][k].abs();
        for i in k + 1..D {
            let v = a[i][k].abs();
            if v > best {
                best = v;
                piv = i;
            }
        }
        if best < 1e-12 {
            return false;
        }
        if piv != k {
            a.swap(k, piv);
            b.swap(k, piv);
        }
        let diag = a[k][k];
        for j in k..D {
            a[k][j] /= diag;
        }
        b[k] /= diag;
        for i in 0..D {
            if i == k {
                continue;
            }
            let f = a[i][k];
            if f == 0.0 {
                continue;
            }
            for j in k..D {
                a[i][j] -= f * a[k][j];
            }
            b[i] -= f * b[k];
        }
    }
    true
}

#[cfg(test)]
#[test]
fn surrogate_recovers_body_direction() {
    let mut s = Surrogate::new();
    let mut rng = mulberry32(7);
    for _ in 0..220 {
        let p = LIFE.mutate(&mut rng, 0.10);
        s.push(&p, p.body * 10.0);
    }
    assert!(s.fit(), "ridge fit");
    let up = {
        let mut p = LIFE;
        p.body = (p.body + 0.03).min(0.17);
        p
    };
    let down = {
        let mut p = LIFE;
        p.body = (p.body - 0.03).max(0.07);
        p
    };
    assert!(
        s.predict(&up) > s.predict(&down),
        "surrogate should prefer larger body"
    );
}

#[cfg(test)]
#[test]
fn every_digital_species_has_taxon_target() {
    assert_eq!(crate::formulas::SPECIES.len(), SPECIES_BIO.len());
    assert_eq!(LIFE.kinds.len(), SPECIES_BIO.len());
}

#[cfg(test)]
#[test]
fn mutate_species_touches_only_that_row() {
    let mut moved = false;
    for seed in 1..64 {
        let mut rng = mulberry32(seed);
        let p = LIFE.mutate_species(14, &mut rng, 0.40);
        assert_eq!(p.body, LIFE.body);
        assert_eq!(p.gyre, LIFE.gyre);
        for i in 0..17 {
            if i == 14 {
                continue;
            }
            assert_eq!(p.kinds[i], LIFE.kinds[i], "species {i} must stay put");
        }
        if p.kinds[14] != LIFE.kinds[14] {
            moved = true;
            break;
        }
    }
    assert!(moved, "shrimp row never moved");
}

#[cfg(test)]
#[test]
fn radial_mutate_skips_heading() {
    let mut rng = mulberry32(21);
    let mut moved = false;
    for _ in 0..48 {
        let p = LIFE.mutate_species(8, &mut rng, 0.50);
        assert_eq!(p.kinds[8].yaw, LIFE.kinds[8].yaw, "flower6 yaw");
        assert_eq!(p.kinds[8].wander, LIFE.kinds[8].wander, "flower6 wander");
        if p.kinds[8] != LIFE.kinds[8] {
            moved = true;
        }
    }
    assert!(moved, "flower6 other params never moved");
}

#[cfg(test)]
#[test]
fn space_ceiling_does_not_pay_nnd() {
    let mut s = SchoolStats {
        min_nnd_bl: 0.90,
        overlap_frac: 0.0,
        cruise_ratio: 0.85,
        ..SchoolStats::default()
    };
    s.kinds[0] = KindBio {
        n: 32.0,
        nnd_bl: 2.80,
        yaw: 0.14,
        polar: 0.32,
        have_polar: true,
        speed_bl: 0.50,
    };
    let mut hi = LIFE;
    hi.kinds[0].space = 2.40;
    let mut mid = LIFE;
    mid.kinds[0].space = 1.10;
    let a = score_species(&s, 0, &mid);
    let b = score_species(&s, 0, &hi);
    assert!(
        a > b + 0.8,
        "space at cap still paid nnd: interior={a:.3} cap={b:.3}"
    );
}

#[cfg(test)]
#[test]
fn nnd_overshoot_scores_below_target() {
    let mut s = SchoolStats {
        min_nnd_bl: 0.90,
        overlap_frac: 0.0,
        cruise_ratio: 0.85,
        ..SchoolStats::default()
    };
    s.kinds[14] = KindBio {
        n: 32.0,
        nnd_bl: 1.05,
        yaw: 0.10,
        polar: 0.78,
        have_polar: true,
        speed_bl: 0.80,
    };
    let on_tgt = score_species(&s, 14, &LIFE);
    s.kinds[14].nnd_bl = 1.80;
    let spread = score_species(&s, 14, &LIFE);
    assert!(
        on_tgt > spread + 0.4,
        "overshoot nnd not taxed: on={on_tgt:.3} spread={spread:.3}"
    );
}

#[cfg(test)]
#[test]
fn shrimp_polar_outweighs_nnd() {
    let t = &SPECIES_BIO[14];
    assert!(
        t.w_polar > t.w_nnd,
        "shrimp polar {} <= nnd {}",
        t.w_polar,
        t.w_nnd
    );
    assert!(
        t.w_polar > t.w_yaw * 4.0,
        "shrimp yaw still dominates polar {} vs yaw {}",
        t.w_polar,
        t.w_yaw
    );
}

#[cfg(test)]
#[test]
fn shrimp_low_yaw_not_better_than_polar() {
    let mut s = SchoolStats {
        min_nnd_bl: 0.90,
        overlap_frac: 0.0,
        cruise_ratio: 0.85,
        ..SchoolStats::default()
    };
    s.kinds[14] = KindBio {
        n: 32.0,
        nnd_bl: 1.05,
        yaw: 0.08,
        polar: 0.20,
        have_polar: true,
        speed_bl: 0.80,
    };
    let low_polar = score_species(&s, 14, &LIFE);
    s.kinds[14].yaw = 0.35;
    s.kinds[14].polar = 0.70;
    let high_polar = score_species(&s, 14, &LIFE);
    assert!(
        high_polar > low_polar + 0.8,
        "yaw-floor still beats polar: low={low_polar:.3} high={high_polar:.3}"
    );
}

#[cfg(test)]
#[test]
fn cma_align_only_moves_slip_not_yaw() {
    let target = 1.60;
    let eval = |p: &LifeParams| -((p.kinds[14].slip - target).powi(2));
    let search = SpeciesSearch {
        lock_space: true,
        align_only: true,
    };
    let (best, _, log) = evolve_species_cfg(LIFE, 14, 14, 11, 0.22, search, eval);
    assert!(
        (best.kinds[14].space - LIFE.kinds[14].space).abs() < 1e-12,
        "space moved {}",
        best.kinds[14].space
    );
    assert!(
        (best.kinds[14].yaw - LIFE.kinds[14].yaw).abs() < 1e-12,
        "yaw moved {}",
        best.kinds[14].yaw
    );
    assert!(
        (best.kinds[14].pace - LIFE.kinds[14].pace).abs() < 1e-12,
        "pace moved {}",
        best.kinds[14].pace
    );
    assert!(
        (best.kinds[14].slip - target).abs() < 0.20,
        "slip {} not near {target}, last={:?}",
        best.kinds[14].slip,
        log.last().map(|g| g.best)
    );
}

#[cfg(test)]
#[test]
fn cma_inject_reaches_high_slip() {
    let eval = |p: &LifeParams| p.kinds[14].slip;
    let search = SpeciesSearch {
        lock_space: true,
        align_only: true,
    };
    let (best, _, _) = evolve_species_cfg(LIFE, 14, 6, 3, 0.38, search, eval);
    assert!(
        best.kinds[14].slip > 1.80,
        "aggressive inject/IPOP still stuck at slip {}",
        best.kinds[14].slip
    );
    assert!(
        (best.kinds[14].yaw - LIFE.kinds[14].yaw).abs() < 1e-12,
        "yaw moved"
    );
}

#[cfg(test)]
#[test]
fn cma_ipop_inflates_sigma_on_plateau() {
    let search = SpeciesSearch {
        lock_space: true,
        align_only: true,
    };
    let mut cma = SpeciesCma::from_life(&LIFE, 14, search, 0.08);
    let eval = |_p: &LifeParams| 1.0;
    let _ = cma.run(LIFE, 14, 12, 5, eval);
    assert!(
        cma.restarts >= 1,
        "flat landscape never restarted (sigma={:.3})",
        cma.sigma
    );
    assert!(
        cma.sigma > 0.16,
        "IPOP left sigma collapsed {:.3}",
        cma.sigma
    );
}
