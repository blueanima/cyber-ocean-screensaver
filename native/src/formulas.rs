//! 与 `fucan/gallery.py` 中的公式一一对应（JS 浮点语义，内部用 f64）。

pub const VIEW: f64 = 400.0;

#[inline]
fn finite(x: f64, y: f64) -> bool {
    x.is_finite() && y.is_finite()
}

#[inline]
fn push(out: &mut Vec<[f32; 2]>, x: f64, y: f64) {
    if finite(x, y) {
        out.push([x as f32, y as f32]);
    }
}

pub fn fill_fucan(t: f64, step: usize, out: &mut Vec<[f32; 2]>) {
    let n_loop = 22000;
    let mut i = 0usize;
    while i < n_loop {
        let x = (i % 100) as f64;
        let y = (i / 100) as f64;
        let k = x / 4.0 - 12.5;
        let e = y / 9.0 + 6.0;
        let o = (k * k + e * e).sqrt() / 9.0;
        if k.abs() < 1e-9 || o < 1e-9 || (y / 2.0).cos().abs() < 0.015 {
            i += step;
            continue;
        }
        let ht = 0.5 * (y / 2.0).tan();
        if !ht.is_finite() || ht.abs() > 60.0 {
            i += step;
            continue;
        }
        let c = o / 2.0 + e / 2.0 - t / 4.0;
        let q = (3.0 / k) * (ht + y.cos()) + k * (5.0 / o + o * y.sin() * (e + 4.0 * o - t).sin());
        let xw = q + 40.0 * c.cos();
        let yw = q * c.sin() - (o * k * k) / 6.0 + 12.0 * e * o;
        if finite(xw, yw) {
            let scale = 0.82;
            out.push([(200.0 + xw * scale) as f32, (28.0 + (yw - 50.0) * scale) as f32]);
        }
        i += step;
    }
}

pub fn fill_youyan(t: f64, step: usize, out: &mut Vec<[f32; 2]>) {
    let mut i = 0usize;
    while i < 18000 {
        let x = (i % 100) as f64;
        let y = (i / 100) as f64;
        let k = x / 4.0 - 12.5;
        let e = y / 9.0 + 5.0;
        let o = (k * k + e * e).sqrt() / 9.0;
        if k.abs() < 1e-6 {
            i += step;
            continue;
        }
        let q = x + 99.0 + (1.0 / k).tan() + o * k * ((e * 9.0).cos() / 4.0 + (y / 2.0).cos()) * (o * 4.0 - t).sin();
        let c = o * e / 30.0 - t / 8.0;
        push(
            out,
            q * 0.7 * c.sin() + 9.0 * (y / 19.0 + t).cos() + 200.0,
            200.0 + q / 2.0 * c.cos(),
        );
        i += step;
    }
}

pub fn fill_jichong(t: f64, step: usize, out: &mut Vec<[f32; 2]>) {
    let mut i = 0usize;
    while i < 9000 {
        let x = i as f64;
        let y = i as f64 / 235.0;
        let e = y / 8.0 - 13.0;
        let k = (4.0 + (y * 2.0 - t).sin() * 3.0) * (x / 29.0).cos();
        if k.abs() < 1e-6 {
            i += step;
            continue;
        }
        let d = (k * k + e * e).sqrt();
        let q = 3.0 * (k * 2.0).sin() + 0.3 / k + (y / 25.0).sin() * k * (9.0 + 4.0 * (e * 9.0 - d * 3.0 + t * 2.0).sin());
        push(out, q + 30.0 * (d - t).cos() + 200.0, 620.0 - q * (d - t).sin() - d * 39.0);
        i += step;
    }
}

pub fn fill_jelly(t: f64, step: usize, out: &mut Vec<[f32; 2]>) {
    let mut i = 0usize;
    while i < 10000 {
        let x = (i % 200) as f64;
        let y = i as f64 / 43.0;
        let k = 5.0 * (x / 14.0).cos() * (y / 30.0).cos();
        let e = y / 8.0 - 13.0;
        let d = (k * k + e * e) / 59.0 + 4.0;
        let a = k.atan2(e);
        let q = 60.0 - 3.0 * (a * e).sin() + k * (3.0 + 4.0 / d * (d * d - t * 2.0).sin());
        let c = d / 2.0 + e / 99.0 - t / 18.0;
        push(out, q * c.sin() + 200.0, (q + d * 9.0) * c.cos() + 200.0);
        i += step;
    }
}

pub fn fill_nebula(t: f64, step: usize, out: &mut Vec<[f32; 2]>) {
    let mut i = 0usize;
    while i < 20000 {
        let x = (i % 200) as f64;
        let y = i as f64 / 200.0;
        let k = x / 8.0 - 12.5;
        let e = y / 8.0 - 12.5;
        let o = (k * k + e * e) / 169.0;
        let d = 0.5 + 5.0 * o.cos();
        push(
            out,
            x + d * k * (d * 2.0 + o + t).sin() + e * (e + t).cos() + 100.0,
            y / 4.0 - o * 135.0 + d * 6.0 * (d * 3.0 + o * 9.0 + t).cos() + 275.0,
        );
        i += step;
    }
}

pub fn fill_lantern(t: f64, step: usize, out: &mut Vec<[f32; 2]>) {
    let mut i = 0usize;
    while i < 10000 {
        let x = (i % 200) as f64;
        let y = i as f64 / 55.0;
        let k = 9.0 * (x / 8.0).cos();
        let e = y / 8.0 - 12.5;
        let d = (k * k + e * e) / 99.0 + t.sin() / 6.0 + 0.5;
        if d.abs() < 1e-6 {
            i += step;
            continue;
        }
        let q = 99.0 - e * (k.atan2(e) * 7.0).sin() / d + k * (3.0 + (d * d - t).cos() * 2.0);
        let c = d / 2.0 + e / 69.0 - t / 16.0;
        push(out, q * c.sin() + 200.0, (q + 19.0 * d) * c.cos() + 200.0);
        i += step;
    }
}

pub fn fill_feather(t: f64, step: usize, out: &mut Vec<[f32; 2]>) {
    let mut i = 1usize;
    while i <= 9000 {
        let y = i as f64 / 790.0;
        let k = if y < 5.0 {
            6.0 + (((y.floor() as i32) ^ 1) as f64).sin() * 6.0
        } else {
            4.0 + y.cos()
        };
        let cs = (i as f64 + t / 4.0).cos();
        let d = ((k * cs) * (k * cs) + (y / 3.0 - 13.0) * (y / 3.0 - 13.0)).sqrt();
        let q = y * k * cs / 5.0 * (2.0 + (d * 2.0 + y - t * 4.0).sin());
        let c = d / 3.0 - t / 2.0 + (i % 2) as f64;
        push(out, q + 90.0 * c.cos() + 200.0, 400.0 - (q * c.sin() + d * 29.0 - 170.0));
        i += step;
    }
}

pub fn fill_tentacle(t: f64, step: usize, out: &mut Vec<[f32; 2]>) {
    let mut i = 1usize;
    while i <= 9000 {
        let y = i as f64 / 345.0;
        let mut x = y;
        if y < 11.0 {
            x = 6.0 + (((x.floor() as i32) ^ 8) as f64).sin() * 6.0;
        } else {
            x = x / 5.0 + (x / 2.0).cos();
        }
        let e = y / 7.0 - 13.0;
        let k = x * (i as f64 - t / 4.0).cos();
        let d = (k * k + e * e).sqrt() + (e / 4.0 + t).sin() / 2.0;
        if d.abs() < 1e-6 {
            i += step;
            continue;
        }
        let q = y * k / d * (3.0 + (d * 2.0 + y / 2.0 - t * 4.0).sin());
        let c = d / 2.0 + 1.0 - t / 2.0;
        push(out, q + 60.0 * c.cos() + 200.0, 400.0 - (q * c.sin() + d * 29.0 - 170.0));
        i += step;
    }
}

pub fn fill_flower6(t: f64, step: usize, out: &mut Vec<[f32; 2]>) {
    let copies = 6;
    let ang = std::f64::consts::PI / 3.0;
    let mut i = 1usize;
    while i <= 5000 {
        let k = (i % 25) as f64 - 12.0;
        let e = i as f64 / 800.0;
        let d = 7.0 * ((k * k + e * e).sqrt() / 3.0 + t / 2.0).cos();
        let bx = k * 4.0 + d * k * (d + e / 9.0 + t).sin();
        let by = e * 2.0 - d * 9.0 - d * 9.0 * (d + t).cos();
        for j in 0..copies {
            let a = j as f64 * ang;
            let (ca, sa) = (a.cos(), a.sin());
            push(out, ca * bx - sa * by + 200.0, sa * bx + ca * by + 200.0);
        }
        i += step;
    }
}

pub fn fill_wheel(t: f64, step: usize, out: &mut Vec<[f32; 2]>) {
    let copies = 14;
    let ang = std::f64::consts::PI / 7.0;
    let mut i = 1usize;
    while i <= 3500 {
        let k = (i % 50) as f64 - 25.0;
        let e = i as f64 / 1100.0;
        let d = 5.0 * ((k * k + e * e).sqrt() - t + (i % 2) as f64).cos();
        if d.abs() < 0.12 {
            i += step;
            continue;
        }
        let bx = k + k * d / 6.0 * (d + e / 3.0 + t).sin();
        let by = 90.0 + e * d - e / d * 2.0 * (d + t).cos();
        for j in 0..copies {
            let a = j as f64 * ang;
            let (ca, sa) = (a.cos(), a.sin());
            push(out, ca * bx - sa * by + 200.0, sa * bx + ca * by + 200.0);
        }
        i += step;
    }
}

pub fn fill_spiral(t: f64, step: usize, out: &mut Vec<[f32; 2]>) {
    let mut i = 0usize;
    while i < 12000 {
        let x = (i % 120) as f64;
        let y = (i / 120) as f64;
        let k = x / 5.0 - 12.0;
        let e = y / 8.0 - 8.0;
        let o = (k * k + e * e).sqrt() / 8.0;
        let c = o * 1.15 + t / 5.0;
        let q = 22.0 + 10.0 * (e * 0.8 + t).sin() + k * (1.6 + 0.35 * (3.0 * o - t).sin());
        push(out, q * c.cos() + 10.0 * (e * 2.0 + t).sin() + 200.0, q * c.sin() * 0.88 + 200.0);
        i += step;
    }
}

pub fn fill_comb(t: f64, step: usize, out: &mut Vec<[f32; 2]>) {
    let mut i = 0usize;
    while i < 10000 {
        let x = (i % 180) as f64;
        let y = i as f64 / 50.0;
        let k = 7.0 * (x / 10.0).cos() * (y / 35.0).cos();
        let e = y / 8.0 - 12.0;
        let d = (k * k + e * e) / 70.0 + 3.0;
        if d.abs() < 1e-6 {
            i += step;
            continue;
        }
        let a = k.atan2(e);
        let q = 48.0 - 4.0 * (a * 4.0).sin() + k * (2.2 + 3.0 / d * (d * d - t).sin());
        let c = d / 2.4 + e / 85.0 - t / 14.0;
        push(
            out,
            q * c.sin() + 200.0,
            (q + 7.0 * d) * c.cos() * 0.78 + 12.0 * (x / 18.0 + t).sin() + 210.0,
        );
        i += step;
    }
}

pub fn fill_saw_eel(t: f64, step: usize, out: &mut Vec<[f32; 2]>) {
    let mut i = 0usize;
    while i < 9000 {
        let x = i as f64;
        let y = i as f64 / 210.0;
        let e = y / 9.0 - 12.0;
        let k = (3.5 + (y * 1.6 - t).sin() * 2.4) * (x / 22.0).cos();
        if k.abs() < 1e-6 {
            i += step;
            continue;
        }
        let d = (k * k + e * e).sqrt();
        let q = 2.2 * (k * 3.0).sin() + 0.25 / k + (y / 18.0).sin() * k * (7.0 + 3.0 * (e * 6.0 - d * 2.0 + t * 2.0).sin());
        push(
            out,
            q + 24.0 * (d * 0.7 - t).cos() + 200.0,
            560.0 - q * (d * 0.7 - t).sin() - d * 32.0,
        );
        i += step;
    }
}

pub fn fill_star8(t: f64, step: usize, out: &mut Vec<[f32; 2]>) {
    let copies = 8;
    let ang = std::f64::consts::PI / 4.0;
    let mut i = 1usize;
    while i <= 4000 {
        let k = (i % 20) as f64 - 10.0;
        let e = i as f64 / 900.0;
        let d = 6.0 * ((k * k + e * e).sqrt() / 4.0 + t / 3.0).cos();
        let bx = 5.0 * k + d * k * (d + t).sin();
        let by = 2.5 * e - 8.0 * d * (d + e / 8.0 + t).cos();
        for j in 0..copies {
            let a = j as f64 * ang;
            let (ca, sa) = (a.cos(), a.sin());
            push(out, ca * bx - sa * by + 200.0, sa * bx + ca * by + 200.0);
        }
        i += step;
    }
}

pub fn fill_shrimp(t: f64, step: usize, out: &mut Vec<[f32; 2]>) {
    let mut i = 0usize;
    while i < 14000 {
        let x = (i % 100) as f64;
        let y = (i / 100) as f64;
        let k = x / 4.0 - 12.5;
        let e = y / 8.0 + 3.5;
        let o = (k * k + e * e).sqrt() / 8.0;
        if k.abs() < 1e-6 || o < 1e-6 {
            i += step;
            continue;
        }
        let q = 55.0 + 10.0 * (k * 0.8).sin() + k * (2.2 + 0.55 * o * (y * 0.7 - t).sin());
        let c = o / 3.2 + e / 22.0 - t / 9.0;
        push(
            out,
            q * 0.5 * c.sin() + 7.0 * (y / 16.0 + t).cos() + 200.0,
            200.0 + q * 0.38 * c.cos() + 6.0 * (k + t * 0.6).sin(),
        );
        i += step;
    }
}

pub fn fill_vortex(t: f64, step: usize, out: &mut Vec<[f32; 2]>) {
    let mut i = 0usize;
    while i < 14000 {
        let x = (i % 200) as f64 - 100.0;
        let y = (i as f64 / 200.0) - 35.0;
        let r = (x * x + y * y).sqrt() / 38.0;
        let th = y.atan2(x);
        let rbig = 62.0 + 22.0 * (3.0 * th + t).sin() + 10.0 * (r * 5.0 - t * 2.0).sin();
        push(
            out,
            rbig * (th + r * 0.45 + t / 7.0).cos() + 200.0,
            rbig * (th + r * 0.45 + t / 7.0).sin() * 0.9 + 200.0,
        );
        i += step;
    }
}

pub fn fill_angel(t: f64, step: usize, out: &mut Vec<[f32; 2]>) {
    let mut i = 0usize;
    while i < 10000 {
        let x = (i % 160) as f64;
        let y = i as f64 / 65.0;
        let k = 6.5 * (x / 12.0).cos() * (y / 38.0).cos();
        let e = y / 9.0 - 11.0;
        let d = (k * k + e * e) / 68.0 + 2.4;
        if d.abs() < 1e-6 {
            i += step;
            continue;
        }
        let a = k.atan2(e);
        let q = 38.0 - 5.0 * (a * 3.0).sin() + k * (2.0 + 3.2 / d * (d * 2.2 - t).sin());
        let c = d / 2.1 + e / 88.0 - t / 15.0;
        push(
            out,
            q * c.sin() + 200.0,
            (q + 6.5 * d) * c.cos() + 8.0 * (e * 0.5 + t).sin() + 205.0,
        );
        i += step;
    }
}

pub type FillFn = fn(f64, usize, &mut Vec<[f32; 2]>);

/// 头尾怎么从身体上读。加新种必须选一类，并出对照图确认。
///
/// 中线是身体脊椎（去掉附肢/翅膀后的对称轴），头尾只在中线两端，不在边上。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeadingKind {
    /// n 次旋转拷贝：无头
    Radial,
    /// 伞盖水母：k=0 是中线，头在伞盖（中线最下方），触手是尾
    Bell,
    /// 细长身体：沿身体参数走脊椎，头朝航向
    Spine,
    /// 附肢在 k≈0 炸开：去掉附肢后的身体曲线才是脊椎
    SpineNoLegs,
    /// 有翅膀：头在身体尖端，不在翅膀上
    Torso,
}

pub struct Species {
    #[allow(dead_code)]
    pub id: &'static str,
    pub name: &'static str,
    pub name_en: &'static str,
    pub formula: &'static [&'static str],
    pub dt: f64,
    #[allow(dead_code)]
    pub heading: HeadingKind,
    pub fill: FillFn,
}

pub const SPECIES: &[Species] = &[
    Species { id: "fucan", name: "北斗浮蚕", name_en: "Beidou Fucan", formula: &["k=x/4-12.5  e=y/9+6  o=sqrt(k*k+e*e)/9", "c=o/2+e/2-t/4", "q=(3/k)(0.5*tan(y/2)+cos y)+k(5/o+o*sin y*sin(e+4o-t))"], dt: std::f64::consts::PI / 90.0, heading: HeadingKind::SpineNoLegs, fill: fill_fucan },
    Species { id: "youyan", name: "蚰蜒", name_en: "House Centipede", formula: &["k=x/4-12.5  e=y/9+5  o=sqrt(k*k+e*e)/9", "q=x+99+tan(1/k)+o k (cos(9e)/4+cos(y/2))sin(4o-t)", "c=o e/30-t/8"], dt: std::f64::consts::PI / 90.0, heading: HeadingKind::SpineNoLegs, fill: fill_youyan },
    Species { id: "jichong", name: "脊虫", name_en: "Spine Worm", formula: &["e=y/8-13", "k=(4+3 sin(2y-t))cos(x/29)  d=sqrt(k*k+e*e)", "q=3 sin(2k)+0.3/k+sin(y/25)*k*(9+4 sin(9e-3d+2t))"], dt: std::f64::consts::PI / 240.0, heading: HeadingKind::Spine, fill: fill_jichong },
    Species { id: "jelly", name: "小水母", name_en: "Jellyfish", formula: &["k=5 cos(x/14)cos(y/30)  e=y/8-13", "d=(k*k+e*e)/59+4  a=atan2(k,e)", "q=60-3 sin(a e)+k(3+4/d sin(d*d-2t))"], dt: std::f64::consts::PI / 20.0, heading: HeadingKind::Bell, fill: fill_jelly },
    Species { id: "nebula", name: "星云水母", name_en: "Nebula Jelly", formula: &["k=x/8-12.5  e=y/8-12.5", "o=(k*k+e*e)/169  d=0.5+5 cos(o)", "X=x+d k sin(2d+o+t)+e cos(e+t)"], dt: std::f64::consts::PI / 120.0, heading: HeadingKind::Bell, fill: fill_nebula },
    Species { id: "lantern", name: "花水母", name_en: "Lantern Jelly", formula: &["k=9 cos(x/8)  e=y/8-12.5", "d=(k*k+e*e)/99+sin(t)/6+0.5", "q=99-e sin(7 atan2(k,e))/d+k(3+2 cos(d*d-t))"], dt: std::f64::consts::PI / 120.0, heading: HeadingKind::Bell, fill: fill_lantern },
    Species { id: "feather", name: "羽鳃", name_en: "Feather Gill", formula: &["y=i/790  k=6+6 sin(floor(y))", "d=sqrt((k cos(i+t/4))^2+(y/3-13)^2)", "q=y k cos(i+t/4)/5*(2+sin(2d+y-4t))"], dt: std::f64::consts::PI / 90.0, heading: HeadingKind::Spine, fill: fill_feather },
    Species { id: "tentacle", name: "触须虫", name_en: "Tentacle Worm", formula: &["y=i/345  e=y/7-13  k=x cos(i-t/4)", "d=sqrt(k*k+e*e)+0.5 sin(e/4+t)", "q=y k/d*(3+sin(2d+y/2-4t))"], dt: std::f64::consts::PI / 120.0, heading: HeadingKind::Spine, fill: fill_tentacle },
    Species { id: "flower6", name: "六瓣花", name_en: "Six-petal", formula: &["k=(i%25)-12  e=i/800", "d=7 cos(sqrt(k*k+e*e)/3+t/2)", "rotate PI/3 x 6"], dt: std::f64::consts::PI / 240.0, heading: HeadingKind::Radial, fill: fill_flower6 },
    Species { id: "wheel", name: "轮虫花", name_en: "Rotifer Wheel", formula: &["k=(i%50)-25  e=i/1100", "d=5 cos(sqrt(k*k+e*e)-t)", "rotate PI/7 x 14"], dt: std::f64::consts::PI / 240.0, heading: HeadingKind::Radial, fill: fill_wheel },
    Species { id: "spiral", name: "螺灯", name_en: "Spiral Lamp", formula: &["k=x/5-12  e=y/8-8  o=sqrt(k*k+e*e)/8", "c=1.15 o+t/5", "q=22+10 sin(0.8e+t)+k(1.6+0.35 sin(3o-t))"], dt: std::f64::consts::PI / 90.0, heading: HeadingKind::Spine, fill: fill_spiral },
    Species { id: "comb", name: "栉水母", name_en: "Comb Jelly", formula: &["k=7 cos(x/10)cos(y/35)  e=y/8-12", "d=(k*k+e*e)/70+3  a=atan2(k,e)", "q=48-4 sin(4a)+k(2.2+3/d sin(d*d-t))"], dt: std::f64::consts::PI / 80.0, heading: HeadingKind::Bell, fill: fill_comb },
    Species { id: "saweel", name: "锯鳗", name_en: "Saw Eel", formula: &["e=y/9-12", "k=(3.5+2.4 sin(1.6y-t))cos(x/22)  d=sqrt(k*k+e*e)", "q=2.2 sin(3k)+0.25/k+sin(y/18)*k*(7+3 sin(6e-2d+2t))"], dt: std::f64::consts::PI / 180.0, heading: HeadingKind::Spine, fill: fill_saw_eel },
    Species { id: "star8", name: "八腕星", name_en: "Octo Star", formula: &["k=(i%20)-10  e=i/900", "d=6 cos(sqrt(k*k+e*e)/4+t/3)", "rotate PI/4 x 8"], dt: std::f64::consts::PI / 200.0, heading: HeadingKind::Radial, fill: fill_star8 },
    Species { id: "shrimp", name: "磷虾", name_en: "Krill", formula: &["k=x/4-12.5  e=y/8+3.5  o=sqrt(k*k+e*e)/8", "q=55+10 sin(0.8k)+k(2.2+0.55 o sin(0.7y-t))", "c=o/3.2+e/22-t/9"], dt: std::f64::consts::PI / 70.0, heading: HeadingKind::Spine, fill: fill_shrimp },
    Species { id: "vortex", name: "涡虫", name_en: "Vortex Worm", formula: &["r=sqrt(x*x+y*y)/38  th=atan2(y,x)", "R=62+22 sin(3 th+t)+10 sin(5r-2t)", "<R cos(th+0.45r+t/7), 0.9 R sin(...)>"], dt: std::f64::consts::PI / 100.0, heading: HeadingKind::Spine, fill: fill_vortex },
    Species { id: "angel", name: "海天使", name_en: "Sea Angel", formula: &["k=6.5 cos(x/12)cos(y/38)  e=y/9-11", "d=(k*k+e*e)/68+2.4  a=atan2(k,e)", "q=38-5 sin(3a)+k(2+3.2/d sin(2.2d-t))"], dt: std::f64::consts::PI / 90.0, heading: HeadingKind::Torso, fill: fill_angel },
];

#[cfg(test)]
mod heading_kind_tests {
    use super::*;

    #[test]
    fn every_species_declares_heading() {
        for s in SPECIES {
            match s.id {
                "jelly" | "nebula" | "lantern" | "comb" => {
                    assert_eq!(s.heading, HeadingKind::Bell, "{}", s.id)
                }
                "flower6" | "wheel" | "star8" => {
                    assert_eq!(s.heading, HeadingKind::Radial, "{}", s.id)
                }
                "fucan" | "youyan" => {
                    assert_eq!(s.heading, HeadingKind::SpineNoLegs, "{}", s.id)
                }
                "angel" => assert_eq!(s.heading, HeadingKind::Torso),
                _ => assert_eq!(s.heading, HeadingKind::Spine, "{}", s.id),
            }
        }
    }
}
