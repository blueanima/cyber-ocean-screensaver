//! 按物种的真实推进方式驱动。
//!
//! | 物种 | 推进 | 朝向 |
//! |---|---|---|
//! | 小水母/星云/花水母 | 伞盖快缩向后喷水，慢张滑行；触手被动拖在尾流 | 伞盖朝前 |
//! | 栉水母 | 八列栉带连续划水，口端朝前，几乎匀速 | 口端朝前 |
//! | 浮蚕/蚰蜒/磷虾 | 附肢异时划水，航向稳 | 头朝前 |
//! | 脊虫/触须虫/锯鳗/涡虫 | 身体行波，头领、尾推 | 头朝前 |
//! | 海天使 | 一对翼瓣扑打，一拍一冲 | 头朝前 |
//! | 羽鳃 | 滤食悬停 | 冠朝上/前 |
//! | 六瓣花/轮虫花/八腕星 | 辐射对称，慢转着漂 | 无头 |
//! | 螺灯 | 螺旋前进 | 壳轴朝前 |
//!
//! 头尾标定见 `formulas::HeadingKind`：中线是身体脊椎，头尾只在两端。

#[cfg(test)]
use crate::formulas::SPECIES;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GaitKind {
    /// 水母：伞体快缩喷水，慢张滑行
    Jet,
    /// 栉水母：栉带连续划水，几乎匀速
    Ciliary,
    /// 浮蚕 / 磷虾：附肢异时划水，航向稳、速度匀
    Metachronal,
    /// 鳗 / 脊虫：身体行波，略有侧摆
    Undulate,
    /// 海天使：翼瓣扑打，一拍一冲
    Flap,
    /// 辐射对称：慢转着漂
    SpinDrift,
    /// 羽鳃：滤食悬停
    Hover,
    /// 螺灯：螺旋前进
    Helix,
}

#[derive(Clone, Copy)]
pub struct Gait {
    pub kind: GaitKind,
    pub hz: f64,
    pub duty: f64,
    pub drag: f64,
    pub cruise: f64,
    pub pulse: f64,
    #[allow(dead_code)]
    pub yaw: f64,
    pub spin: f64,
    pub sway: f64,
    pub bell: f64,
    pub morph: f64,
    pub rise: f64,
}

pub const GAITS: &[Gait] = &[
    Gait { kind: GaitKind::Metachronal, hz: 1.15, duty: 0.50, drag: 2.20, cruise: 0.018, pulse: 0.010, yaw: 0.035, spin: 0.00, sway: 0.038, bell: 0.00, morph: 1.15, rise: 0.000 }, // fucan
    Gait { kind: GaitKind::Metachronal, hz: 1.35, duty: 0.50, drag: 2.30, cruise: 0.020, pulse: 0.012, yaw: 0.040, spin: 0.00, sway: 0.042, bell: 0.00, morph: 1.20, rise: 0.000 }, // youyan
    Gait { kind: GaitKind::Undulate, hz: 0.85, duty: 0.50, drag: 2.00, cruise: 0.015, pulse: 0.018, yaw: 0.090, spin: 0.00, sway: 0.085, bell: 0.00, morph: 1.25, rise: 0.000 }, // jichong
    Gait { kind: GaitKind::Jet, hz: 0.48, duty: 0.28, drag: 2.80, cruise: 0.006, pulse: 0.28, yaw: 0.000, spin: 0.00, sway: 0.000, bell: 0.16, morph: 1.35, rise: -0.0010 }, // jelly
    Gait { kind: GaitKind::Jet, hz: 0.42, duty: 0.30, drag: 2.70, cruise: 0.006, pulse: 0.24, yaw: 0.008, spin: 0.00, sway: 0.000, bell: 0.14, morph: 1.25, rise: -0.0009 }, // nebula
    Gait { kind: GaitKind::Jet, hz: 0.45, duty: 0.29, drag: 2.75, cruise: 0.006, pulse: 0.26, yaw: 0.006, spin: 0.00, sway: 0.000, bell: 0.15, morph: 1.30, rise: -0.0009 }, // lantern
    Gait { kind: GaitKind::Hover, hz: 0.28, duty: 0.40, drag: 4.00, cruise: 0.004, pulse: 0.008, yaw: 0.020, spin: 0.04, sway: 0.018, bell: 0.04, morph: 0.85, rise: -0.0008 }, // feather
    Gait { kind: GaitKind::Undulate, hz: 0.72, duty: 0.50, drag: 2.10, cruise: 0.015, pulse: 0.016, yaw: 0.075, spin: 0.00, sway: 0.070, bell: 0.00, morph: 1.15, rise: 0.000 }, // tentacle
    Gait { kind: GaitKind::SpinDrift, hz: 0.22, duty: 0.50, drag: 2.80, cruise: 0.005, pulse: 0.000, yaw: 0.000, spin: 0.22, sway: 0.000, bell: 0.03, morph: 1.00, rise: -0.0004 }, // flower6
    Gait { kind: GaitKind::SpinDrift, hz: 0.20, duty: 0.50, drag: 2.60, cruise: 0.006, pulse: 0.000, yaw: 0.000, spin: 0.20, sway: 0.000, bell: 0.00, morph: 0.72, rise: 0.000 }, // wheel
    Gait { kind: GaitKind::Helix, hz: 0.55, duty: 0.50, drag: 2.20, cruise: 0.014, pulse: 0.008, yaw: 0.030, spin: 0.18, sway: 0.032, bell: 0.00, morph: 1.15, rise: 0.000 }, // spiral
    Gait { kind: GaitKind::Ciliary, hz: 1.60, duty: 0.50, drag: 1.80, cruise: 0.016, pulse: 0.004, yaw: 0.012, spin: 0.03, sway: 0.014, bell: 0.02, morph: 1.10, rise: -0.0008 }, // comb
    Gait { kind: GaitKind::Undulate, hz: 0.95, duty: 0.50, drag: 1.90, cruise: 0.018, pulse: 0.020, yaw: 0.110, spin: 0.00, sway: 0.095, bell: 0.00, morph: 1.30, rise: 0.000 }, // saweel
    Gait { kind: GaitKind::SpinDrift, hz: 0.18, duty: 0.50, drag: 2.90, cruise: 0.004, pulse: 0.000, yaw: 0.000, spin: 0.16, sway: 0.000, bell: 0.04, morph: 0.95, rise: -0.0004 }, // star8
    Gait { kind: GaitKind::Metachronal, hz: 1.55, duty: 0.45, drag: 2.40, cruise: 0.021, pulse: 0.016, yaw: 0.030, spin: 0.00, sway: 0.030, bell: 0.00, morph: 1.20, rise: 0.000 }, // shrimp
    Gait { kind: GaitKind::Undulate, hz: 0.62, duty: 0.50, drag: 2.30, cruise: 0.014, pulse: 0.010, yaw: 0.072, spin: 0.00, sway: 0.052, bell: 0.00, morph: 1.10, rise: 0.000 }, // vortex
    Gait { kind: GaitKind::Flap, hz: 1.05, duty: 0.38, drag: 2.40, cruise: 0.012, pulse: 0.035, yaw: 0.020, spin: 0.00, sway: 0.028, bell: 0.06, morph: 1.15, rise: -0.0006 }, // angel
];

const _: () = assert!(GAITS.len() == 17);

pub fn gait(ci: usize) -> Gait {
    GAITS[ci.min(GAITS.len() - 1)]
}

pub struct Drive {
    pub speed: f64,
    pub d_rot: f64,
    pub spin_vis: f64,
    pub bell: f64,
    pub sway: f64,
    pub morph: f64,
    #[allow(dead_code)]
    pub slip: f64,
}

fn approach(speed: f64, target: f64, rate: f64, dt: f64) -> f64 {
    let blend = 1.0 - (-rate * dt).exp();
    speed + (target - speed) * blend.clamp(0.0, 1.0)
}

pub fn drive(kind: GaitKind, g: Gait, phi: f64, amp: f64, bias: f64, speed: f64, dt: f64) -> Drive {
    let tau = std::f64::consts::TAU;
    let u = phi.rem_euclid(tau) / tau;
    let sphi = phi.sin();
    let duty = g.duty.clamp(0.18, 0.6);
    let contracting = u < duty;
    let contract = if contracting {
        1.0 - u / duty
    } else {
        0.0
    };
    let refill = if contracting {
        0.0
    } else {
        ((u - duty) / (1.0 - duty)).clamp(0.0, 1.0)
    };

    let mut target = g.cruise * amp;
    let mut impulse = 0.0;
    let mut pulsed = false;
    let d_rot;
    let mut bell = 1.0;
    let mut sway = 0.0;
    let morph;
    let mut spin_vis = 0.0;
    let mut slip = 0.0;

    match kind {
        // 水母：快缩喷水 + 慢张滑行；放松前期有被动回能（PER）二次加速
        GaitKind::Jet => {
            pulsed = true;
            impulse = if contracting {
                g.pulse * amp * g.hz * tau * contract.powi(2)
            } else if refill < 0.38 {
                g.pulse * amp * g.hz * 0.28 * (1.0 - refill / 0.38)
            } else {
                0.0
            };
            bell = if contracting {
                1.0 - g.bell * (u / duty)
            } else {
                1.0 - g.bell + g.bell * refill
            };
            morph = if contracting {
                g.morph * 1.28
            } else {
                g.morph * (0.48 + 0.22 * refill)
            };
            d_rot = bias * 0.018;
        }
        GaitKind::Ciliary => {
            target = g.cruise * amp * (0.96 + 0.04 * sphi.abs());
            d_rot = bias * 0.020;
            bell = 1.0 + g.bell * 0.5 * sphi;
            sway = g.sway * sphi;
            morph = g.morph * (0.94 + 0.08 * sphi.abs());
        }
        GaitKind::Metachronal => {
            target = g.cruise * amp * (0.92 + 0.08 * (0.5 + 0.5 * sphi));
            d_rot = bias * 0.024;
            sway = g.sway * sphi;
            morph = g.morph * (0.90 + 0.16 * (0.5 + 0.5 * sphi));
            slip = 0.05 * sphi;
        }
        GaitKind::Undulate => {
            target = g.cruise * amp * (0.88 + 0.12 * sphi.max(0.0).powi(2));
            d_rot = bias * 0.028;
            sway = g.sway * sphi;
            morph = g.morph * (0.86 + 0.22 * sphi.abs());
            slip = 0.10 * sphi;
        }
        GaitKind::Flap => {
            pulsed = true;
            impulse = if contracting {
                g.pulse * amp * g.hz * tau * contract
            } else {
                0.0
            };
            target = g.cruise * amp;
            d_rot = bias * 0.022;
            sway = g.sway * sphi;
            bell = 1.0 - g.bell * 0.55 * if contracting { u / duty } else { 0.0 };
            morph = if contracting {
                g.morph * 1.12
            } else {
                g.morph * 0.82
            };
        }
        GaitKind::SpinDrift => {
            target = g.cruise * amp;
            d_rot = bias * 0.016;
            sway = 0.0;
            bell = 1.0;
            morph = g.morph * (0.96 + 0.04 * sphi.abs());
            spin_vis = g.spin;
        }
        GaitKind::Hover => {
            target = g.cruise * amp * (0.70 + 0.30 * sphi);
            d_rot = bias * 0.014;
            sway = g.sway * sphi;
            bell = 1.0 + g.bell * sphi;
            morph = g.morph * (0.88 + 0.16 * sphi.abs());
        }
        GaitKind::Helix => {
            target = g.cruise * amp * (0.90 + 0.10 * sphi);
            d_rot = bias * 0.020;
            sway = g.sway * sphi;
            morph = g.morph * (0.90 + 0.14 * sphi.abs());
            spin_vis = g.spin;
            slip = 0.06 * sphi;
        }
    }

    let mut spd = if pulsed {
        speed * (1.0 - g.drag * dt).max(0.0) + impulse * dt + target * dt * 0.15
    } else {
        approach(speed, target, g.drag, dt)
    };
    if pulsed {
        spd = spd.max(g.cruise * amp * 0.70);
    }
    spd = spd.clamp(0.0, 0.10);

    Drive {
        speed: spd,
        d_rot: d_rot * dt,
        spin_vis: spin_vis * dt,
        bell: bell.clamp(0.78, 1.18),
        sway,
        morph,
        slip,
    }
}

#[cfg(test)]
pub struct Metrics {
    pub id: &'static str,
    pub kind: GaitKind,
    pub mean_speed: f64,
    pub speed_cv: f64,
    pub yaw_rms: f64,
    pub spin: f64,
    pub disp: f64,
    pub align: f64,
    pub yaw_flips: u32,
}

#[cfg(test)]
pub struct Sample {
    pub x: f64,
    pub y: f64,
    pub rot: f64,
    pub speed: f64,
}

#[cfg(test)]
pub fn simulate(ci: usize, seconds: f64) -> (Metrics, Vec<Sample>) {
    let g = gait(ci);
    let spec = &SPECIES[ci];
    let mut phi = 0.3;
    let amp = 1.0;
    let mut speed = g.cruise;
    let mut rot = 0.4;
    let mut x = 0.5;
    let mut y = 0.5;
    let mut bias = 0.0;
    let dt = 1.0 / 60.0;
    let n = (seconds / dt) as usize;
    let mut speeds = Vec::with_capacity(n);
    let mut yaws = Vec::with_capacity(n);
    let mut samples = Vec::with_capacity(n);
    let mut spin = 0.0;
    let mut align_acc = 0.0;
    let mut flips = 0u32;
    let mut prev_yaw: f64 = 0.0;
    let x0 = x;
    let y0 = y;
    for i in 0..n {
        let t = i as f64 * dt;
        phi += g.hz * std::f64::consts::TAU * dt;
        let bdes = 0.08 * (t * 0.028).sin();
        bias += dt * 0.45 * (bdes - bias);
        let d = drive(g.kind, g, phi, amp, bias, speed, dt);
        speed = d.speed;
        rot += d.d_rot;
        spin += d.d_rot.abs() + d.spin_vis.abs();
        let fx = rot.cos();
        let fy = -rot.sin();
        let vx = speed * fx + speed * d.slip * fy;
        let vy = speed * fy - speed * d.slip * fx + g.rise;
        x += vx * dt;
        y += vy * dt;
        let v = (vx * vx + vy * vy).sqrt();
        if v > 1e-6 {
            align_acc += (fx * vx + fy * vy) / v;
        }
        let yaw: f64 = d.d_rot / dt;
        if i > 8 && prev_yaw.signum() != 0.0 && yaw.signum() != 0.0 && prev_yaw.signum() != yaw.signum() && yaw.abs() > 0.004 {
            flips += 1;
        }
        prev_yaw = yaw;
        speeds.push(speed);
        yaws.push(yaw);
        samples.push(Sample { x, y, rot, speed });
        let _ = spec;
    }
    let mean = speeds.iter().sum::<f64>() / n as f64;
    let var = speeds.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / n as f64;
    let cv = if mean > 1e-5 { var.sqrt() / mean } else { 0.0 };
    let yaw_rms = (yaws.iter().map(|y| y * y).sum::<f64>() / n as f64).sqrt();
    (
        Metrics {
            id: spec.id,
            kind: g.kind,
            mean_speed: mean,
            speed_cv: cv,
            yaw_rms,
            spin,
            disp: ((x - x0).hypot(y - y0)),
            align: align_acc / n as f64,
            yaw_flips: flips,
        },
        samples,
    )
}

#[cfg(test)]
fn check(m: &Metrics) -> Result<(), String> {
    if m.kind != GaitKind::SpinDrift && m.spin > 3.2 {
        return Err(format!("{} 转圈过多 spin={:.2}", m.id, m.spin));
    }
    if m.align < 0.82 {
        return Err(format!("{} 头向与前进不一致 align={:.2}", m.id, m.align));
    }
    if m.kind != GaitKind::SpinDrift && m.yaw_flips > 8 {
        return Err(format!("{} 左右连转 flips={}", m.id, m.yaw_flips));
    }
    match m.kind {
        GaitKind::Jet => {
            if m.speed_cv < 0.14 {
                return Err(format!("{} 喷水不够脉冲 cv={:.2}", m.id, m.speed_cv));
            }
            if m.yaw_rms > 0.12 {
                return Err(format!("{} 水母不该摇摆 yaw={:.2}", m.id, m.yaw_rms));
            }
            if m.disp < 0.035 {
                return Err(format!("{} 几乎没动 disp={:.2}", m.id, m.disp));
            }
            if m.mean_speed > 0.036 {
                return Err(format!("{} 喷水滑得太快 v={:.3}", m.id, m.mean_speed));
            }
        }
        GaitKind::Ciliary => {
            if m.speed_cv > 0.20 {
                return Err(format!("{} 栉水母应匀速 cv={:.2}", m.id, m.speed_cv));
            }
            if m.yaw_rms > 0.12 {
                return Err(format!("{} 栉水母摇摆过大 yaw={:.2}", m.id, m.yaw_rms));
            }
            if m.mean_speed < 0.006 {
                return Err(format!("{} 太慢 v={:.3}", m.id, m.mean_speed));
            }
            if m.mean_speed > 0.028 {
                return Err(format!("{} 载具感过强 v={:.3}", m.id, m.mean_speed));
            }
        }
        GaitKind::Metachronal => {
            if m.speed_cv > 0.30 {
                return Err(format!("{} 异时划水应较稳 cv={:.2}", m.id, m.speed_cv));
            }
            if m.mean_speed < 0.006 {
                return Err(format!("{} 太慢 v={:.3}", m.id, m.mean_speed));
            }
            if m.mean_speed > 0.028 {
                return Err(format!("{} 载具感过强 v={:.3}", m.id, m.mean_speed));
            }
            if m.yaw_rms > 0.18 {
                return Err(format!("{} 划水摇摆过大 yaw={:.2}", m.id, m.yaw_rms));
            }
        }
        GaitKind::Undulate => {
            if m.disp < 0.045 {
                return Err(format!("{} 波动没前进 disp={:.2}", m.id, m.disp));
            }
            if m.speed_cv > 0.28 {
                return Err(format!("{} 波动不应一冲一冲 cv={:.2}", m.id, m.speed_cv));
            }
            if m.mean_speed > 0.028 {
                return Err(format!("{} 载具感过强 v={:.3}", m.id, m.mean_speed));
            }
            if m.yaw_rms > 0.20 {
                return Err(format!("{} 头应朝前 yaw={:.2}", m.id, m.yaw_rms));
            }
        }
        GaitKind::Flap => {
            if m.speed_cv < 0.08 {
                return Err(format!("{} 扑翼应有节奏 cv={:.2}", m.id, m.speed_cv));
            }
            if m.mean_speed > 0.032 {
                return Err(format!("{} 海天使冲得太快 v={:.3}", m.id, m.mean_speed));
            }
            if m.yaw_rms > 0.18 {
                return Err(format!("{} 海天使摇摆过大 yaw={:.2}", m.id, m.yaw_rms));
            }
        }
        GaitKind::SpinDrift => {
            if m.spin < 0.45 || m.spin > 4.2 {
                return Err(format!("{} 自旋异常 spin={:.2}", m.id, m.spin));
            }
        }
        GaitKind::Hover => {
            if m.mean_speed > 0.012 {
                return Err(format!("{} 悬停过快 v={:.3}", m.id, m.mean_speed));
            }
        }
        GaitKind::Helix => {
            if m.disp < 0.04 {
                return Err(format!("{} 螺旋没前进 disp={:.2}", m.id, m.disp));
            }
            if m.mean_speed > 0.028 {
                return Err(format!("{} 载具感过强 v={:.3}", m.id, m.mean_speed));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
pub fn tune_all() -> Vec<(Metrics, Result<(), String>, Vec<Sample>)> {
    (0..SPECIES.len())
        .map(|ci| {
            let (m, s) = simulate(ci, 8.0);
            let r = check(&m);
            (m, r, s)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn every_species_has_living_gait() {
        assert_eq!(GAITS.len(), SPECIES.len());
        let mut failed = Vec::new();
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.cache/gait-obs");
        let _ = std::fs::create_dir_all(&dir);
        for (m, r, samples) in tune_all() {
            println!(
                "{:<10} {:<12} v={:.3} cv={:.2} yaw={:.3} spin={:.2} disp={:.3} align={:.2} flips={} {:?}",
                m.id,
                format!("{:?}", m.kind),
                m.mean_speed,
                m.speed_cv,
                m.yaw_rms,
                m.spin,
                m.disp,
                m.align,
                m.yaw_flips,
                r.as_ref().map(|_| "ok").unwrap_or("fail")
            );
            let csv = dir.join(format!("{}.csv", m.id));
            if let Ok(mut f) = std::fs::File::create(&csv) {
                let _ = writeln!(f, "x,y,rot,speed");
                for s in samples.iter().step_by(4) {
                    let _ = writeln!(f, "{:.5},{:.5},{:.4},{:.5}", s.x, s.y, s.rot, s.speed);
                }
            }
            if let Err(e) = r {
                failed.push(format!(
                    "{e}  (v={:.3} cv={:.2} yaw={:.2} spin={:.2} disp={:.2} align={:.2} flips={})",
                    m.mean_speed, m.speed_cv, m.yaw_rms, m.spin, m.disp, m.align, m.yaw_flips
                ));
            }
        }
        assert!(failed.is_empty(), "gait checks failed:\n{}", failed.join("\n"));
    }
}
