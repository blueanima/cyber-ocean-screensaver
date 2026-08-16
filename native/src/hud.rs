//! 网页同款图例：中英名称 + 虚线指向当前生物。

use std::collections::HashMap;

use crate::formulas::SPECIES;
use crate::ocean::{Creature, LegendGeom};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct HudInstance {
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub uv0: [f32; 2],
    pub uv1: [f32; 2],
    pub rgba: [f32; 4],
    pub extra: [f32; 2],
}

#[derive(Clone, Copy)]
struct Glyph {
    uv0: [f32; 2],
    uv1: [f32; 2],
    w: f32,
    h: f32,
    xmin: f32,
    advance: f32,
}

pub struct TextEngine {
    font: Option<fontdue::Font>,
    atlas: Vec<u8>,
    atlas_w: u32,
    atlas_h: u32,
    glyphs: HashMap<(char, u32), Glyph>,
    pack_x: u32,
    pack_y: u32,
    pack_row_h: u32,
}

const ATLAS_W: u32 = 2048;
const ATLAS_H: u32 = 2048;
const PX_TITLE: u32 = 20;
const PX_NAME: u32 = 18;
const PX_EN: u32 = 14;
const PX_FORMULA: u32 = 16;

fn bundled_font_paths() -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let rel = "fonts/DroidSansFallbackFull.ttf";
    if let Ok(appdir) = std::env::var("APPDIR") {
        out.push(
            std::path::PathBuf::from(appdir)
                .join("usr/share/cyber-ocean")
                .join(rel),
        );
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(bin) = exe.parent() {
            out.push(bin.join("../share/cyber-ocean").join(rel));
            out.push(bin.join(rel));
            out.push(bin.join("../../..").join(rel));
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        out.push(cwd.join(rel));
    }
    out
}

fn font_paths() -> Vec<std::path::PathBuf> {
    let mut out = bundled_font_paths();
    out.extend([
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc".into(),
        "/usr/share/fonts/noto-cjk/NotoSansCJKsc-Regular.otf".into(),
        "/usr/share/fonts/opentype/noto/NotoSansCJKsc-Regular.otf".into(),
        "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf".into(),
        "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc".into(),
        "/usr/share/fonts/truetype/arphic/uming.ttc".into(),
        "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc".into(),
        "/System/Library/Fonts/PingFang.ttc".into(),
        "/System/Library/Fonts/STHeiti Light.ttc".into(),
        "/System/Library/Fonts/Hiragino Sans GB.ttc".into(),
        "C:\\Windows\\Fonts\\msyh.ttc".into(),
        "C:\\Windows\\Fonts\\msyh.ttf".into(),
        "C:\\Windows\\Fonts\\simhei.ttf".into(),
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf".into(),
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf".into(),
        "/usr/share/fonts/truetype/freefont/FreeSans.ttf".into(),
        "/usr/share/fonts/truetype/ttf-dejavu/DejaVuSans.ttf".into(),
    ]);
    out
}

fn font_covers_legend(font: &fontdue::Font) -> bool {
    font.has_glyph('图')
        && font.has_glyph('蚕')
        && font.has_glyph('L')
        && font.has_glyph('e')
        && font.has_glyph('水')
}

fn load_font_from_bytes(bytes: &[u8]) -> Option<fontdue::Font> {
    for index in 0..8u32 {
        let settings = fontdue::FontSettings {
            collection_index: index,
            scale: 40.0,
            load_substitutions: true,
        };
        let Ok(font) = fontdue::Font::from_bytes(bytes, settings) else {
            continue;
        };
        if font_covers_legend(&font) {
            return Some(font);
        }
    }
    None
}

impl TextEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            font: None,
            atlas: vec![0u8; (ATLAS_W * ATLAS_H) as usize],
            atlas_w: ATLAS_W,
            atlas_h: ATLAS_H,
            glyphs: HashMap::new(),
            pack_x: 1,
            pack_y: 1,
            pack_row_h: 0,
        };
        for path in font_paths() {
            let Ok(bytes) = std::fs::read(&path) else {
                continue;
            };
            if let Some(font) = load_font_from_bytes(&bytes) {
                eprintln!("图例字体：{}", path.display());
                engine.font = Some(font);
                break;
            }
        }
        if engine.font.is_none() {
            eprintln!("未找到中文字体，图例中文可能缺失；仍尝试拉丁字体画公式");
            for path in font_paths() {
                let Ok(bytes) = std::fs::read(&path) else {
                    continue;
                };
                for index in 0..4u32 {
                    let settings = fontdue::FontSettings {
                        collection_index: index,
                        scale: 40.0,
                        load_substitutions: true,
                    };
                    if let Ok(font) = fontdue::Font::from_bytes(bytes.as_slice(), settings) {
                        if font.has_glyph('k') && font.has_glyph('=') {
                            eprintln!("公式字体：{}", path.display());
                            engine.font = Some(font);
                            break;
                        }
                    }
                }
                if engine.font.is_some() {
                    break;
                }
            }
        }
        if engine.font.is_some() {
            engine.bake();
        } else {
            eprintln!("未找到可用字体，图例和公式文字无法绘制");
        }
        engine
    }

    fn bake(&mut self) {
        let mut chars: Vec<char> = "图例种子  / Legend seed ·0123456789-—".chars().collect();
        chars.extend(' '..='~');
        for spec in SPECIES {
            chars.extend(spec.name.chars());
            chars.extend(spec.name_en.chars());
            for line in spec.formula {
                chars.extend(line.chars());
            }
        }
        chars.extend("FORMULA公式".chars());
        chars.sort_unstable();
        chars.dedup();
        for size in [PX_FORMULA, PX_EN, PX_NAME, PX_TITLE] {
            for &ch in &chars {
                self.ensure_glyph(ch, size);
            }
        }
    }

    fn ensure_glyph(&mut self, ch: char, px: u32) {
        if self.glyphs.contains_key(&(ch, px)) {
            return;
        }
        let Some(font) = self.font.as_ref() else {
            return;
        };
        if !font.has_glyph(ch) && ch != ' ' {
            return;
        }
        if ch == ' ' {
            self.glyphs.insert(
                (ch, px),
                Glyph {
                    uv0: [0.0, 0.0],
                    uv1: [0.0, 0.0],
                    w: 0.0,
                    h: 0.0,
                    xmin: 0.0,
                    advance: px as f32 * 0.34,
                },
            );
            return;
        }
        let (metrics, bitmap) = font.rasterize(ch, px as f32);
        let gw = metrics.width as u32;
        let gh = metrics.height as u32;
        if gw == 0 || gh == 0 {
            self.glyphs.insert(
                (ch, px),
                Glyph {
                    uv0: [0.0, 0.0],
                    uv1: [0.0, 0.0],
                    w: 0.0,
                    h: 0.0,
                    xmin: 0.0,
                    advance: metrics.advance_width,
                },
            );
            return;
        }
        if self.pack_x + gw + 1 >= self.atlas_w {
            self.pack_x = 1;
            self.pack_y += self.pack_row_h + 1;
            self.pack_row_h = 0;
        }
        if self.pack_y + gh + 1 >= self.atlas_h {
            return;
        }
        let gx = self.pack_x;
        let gy = self.pack_y;
        for row in 0..gh {
            let dst = ((gy + row) * self.atlas_w + gx) as usize;
            let src = (row * gw) as usize;
            self.atlas[dst..dst + gw as usize].copy_from_slice(&bitmap[src..src + gw as usize]);
        }
        self.pack_row_h = self.pack_row_h.max(gh);
        self.pack_x += gw + 1;
        self.glyphs.insert(
            (ch, px),
            Glyph {
                uv0: [gx as f32 / self.atlas_w as f32, gy as f32 / self.atlas_h as f32],
                uv1: [
                    (gx + gw) as f32 / self.atlas_w as f32,
                    (gy + gh) as f32 / self.atlas_h as f32,
                ],
                w: gw as f32,
                h: gh as f32,
                xmin: metrics.xmin as f32,
                advance: metrics.advance_width,
            },
        );
    }

    pub fn atlas_info(&self) -> (u32, u32, &[u8]) {
        (self.atlas_w, self.atlas_h, &self.atlas)
    }

    fn glyph(&self, ch: char, px: u32) -> Option<&Glyph> {
        self.glyphs
            .get(&(ch, px))
            .or_else(|| self.glyphs.get(&(ch, PX_FORMULA)))
            .or_else(|| self.glyphs.get(&(ch, PX_EN)))
            .or_else(|| self.glyphs.get(&(ch, PX_NAME)))
            .or_else(|| self.glyphs.get(&(ch, PX_TITLE)))
    }

    fn draw_text(
        &self,
        out: &mut Vec<HudInstance>,
        text: &str,
        mut x: f32,
        baseline: f32,
        px: u32,
        rgba: [f32; 4],
        scale: f32,
    ) {
        let s = scale.clamp(0.62, 2.4);
        for ch in text.chars() {
            let Some(g) = self.glyph(ch, px) else {
                x += px as f32 * 0.5 * s;
                continue;
            };
            if g.w > 0.0 && g.h > 0.0 {
                let w = g.w * s;
                let h = g.h * s;
                let gx = x + g.xmin * s;
                let gy = baseline - h * 0.5;
                out.push(rect(
                    [gx + w * 0.5, gy + h * 0.5],
                    [w, h],
                    rgba,
                    -1.0,
                    0.0,
                    g.uv0,
                    g.uv1,
                ));
            }
            x += g.advance * s;
        }
    }

    fn measure_text(&self, text: &str, px: u32, scale: f32) -> f32 {
        let s = scale.clamp(0.62, 2.4);
        let mut w = 0.0f32;
        for ch in text.chars() {
            if let Some(g) = self.glyph(ch, px) {
                w += g.advance * s;
            } else {
                w += px as f32 * 0.5 * s;
            }
        }
        w
    }
}

fn rect(
    pos: [f32; 2],
    size: [f32; 2],
    rgba: [f32; 4],
    radius: f32,
    angle: f32,
    uv0: [f32; 2],
    uv1: [f32; 2],
) -> HudInstance {
    HudInstance {
        pos,
        size,
        uv0,
        uv1,
        rgba,
        extra: [radius, angle],
    }
}

fn solid(pos: [f32; 2], size: [f32; 2], rgba: [f32; 4], radius: f32, angle: f32) -> HudInstance {
    rect(pos, size, rgba, radius, angle, [0.0, 0.0], [0.0, 0.0])
}

fn cubic(p0: [f32; 2], p1: [f32; 2], p2: [f32; 2], p3: [f32; 2], t: f32) -> [f32; 2] {
    let u = 1.0 - t;
    let a = u * u * u;
    let b = 3.0 * u * u * t;
    let c = 3.0 * u * t * t;
    let d = t * t * t;
    [
        a * p0[0] + b * p1[0] + c * p2[0] + d * p3[0],
        a * p0[1] + b * p1[1] + c * p2[1] + d * p3[1],
    ]
}

fn push_dashed_bezier(
    out: &mut Vec<HudInstance>,
    p0: [f32; 2],
    p1: [f32; 2],
    p2: [f32; 2],
    p3: [f32; 2],
    ocean_t: f64,
    rgba: [f32; 4],
    thickness: f32,
) {
    let mut prev = p0;
    let mut dist = 0.0f32;
    let offset = (-ocean_t * 28.0) as f32;
    let period = 10.0;
    let on = 5.0;
    let steps = 48;
    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let p = cubic(p0, p1, p2, p3, t);
        let dx = p[0] - prev[0];
        let dy = p[1] - prev[1];
        let len = (dx * dx + dy * dy).sqrt().max(0.001);
        let mid_dist = dist + len * 0.5;
        let phase = (mid_dist + offset).rem_euclid(period);
        if phase < on {
            let ang = dy.atan2(dx);
            out.push(solid(
                [(prev[0] + p[0]) * 0.5, (prev[1] + p[1]) * 0.5],
                [len + 0.6, thickness],
                rgba,
                0.0,
                ang,
            ));
        }
        dist += len;
        prev = p;
    }
}

fn push_ring(out: &mut Vec<HudInstance>, cx: f32, cy: f32, radius: f32, rgba: [f32; 4], thickness: f32) {
    let n = 48;
    let mut prev = [cx + radius, cy];
    for i in 1..=n {
        let a = i as f32 / n as f32 * std::f32::consts::TAU;
        let p = [cx + radius * a.cos(), cy + radius * a.sin()];
        let dx = p[0] - prev[0];
        let dy = p[1] - prev[1];
        let len = (dx * dx + dy * dy).sqrt().max(0.001);
        out.push(solid(
            [(prev[0] + p[0]) * 0.5, (prev[1] + p[1]) * 0.5],
            [len + 0.4, thickness],
            rgba,
            0.0,
            dy.atan2(dx),
        ));
        prev = p;
    }
}

fn push_ring_ticks(out: &mut Vec<HudInstance>, cx: f32, cy: f32, radius: f32, rgba: [f32; 4], s: f32) {
    for k in 0..4 {
        let a = k as f32 * std::f32::consts::FRAC_PI_2;
        let (sa, ca) = (a.sin(), a.cos());
        let inner = radius - 5.0 * s;
        let outer = radius + 5.0 * s;
        let p0 = [cx + inner * ca, cy + inner * sa];
        let p1 = [cx + outer * ca, cy + outer * sa];
        out.push(solid(
            [(p0[0] + p1[0]) * 0.5, (p0[1] + p1[1]) * 0.5],
            [10.0 * s, 1.6 * s],
            rgba,
            0.0,
            a,
        ));
    }
}

fn wrapped_formulas(lines: &[&str], max_chars: usize) -> Vec<String> {
    let mut out = Vec::new();
    for line in lines {
        let mut rest = *line;
        while rest.len() > max_chars {
            let head = &rest[..max_chars];
            let cut = head.rfind(' ').filter(|i| *i > max_chars / 3).unwrap_or(max_chars);
            out.push(rest[..cut].to_string());
            rest = rest[cut..].trim_start();
        }
        if !rest.is_empty() {
            out.push(rest.to_string());
        }
    }
    out
}

fn push_formula_panel(
    out: &mut Vec<HudInstance>,
    text: &TextEngine,
    header: &str,
    lines: &[String],
    mut left: f32,
    mut top: f32,
    s: f32,
    screen: (f32, f32),
    text_scale: f32,
) {
    let (sw, sh) = screen;
    let pad = 12.0 * s;
    let line_h = 18.0 * s;
    let mut panel_w = text.measure_text(header, PX_FORMULA, text_scale);
    for line in lines {
        panel_w = panel_w.max(text.measure_text(line, PX_FORMULA, text_scale));
    }
    panel_w = (panel_w + pad * 2.0).clamp(180.0 * s, (sw * 0.88).max(240.0));
    let panel_h = pad * 2.0 + 18.0 * s + lines.len() as f32 * line_h;
    left = left.clamp(8.0, (sw - panel_w - 8.0).max(8.0));
    top = top.clamp(8.0, (sh - panel_h - 8.0).max(8.0));
    let cx = left + panel_w * 0.5;
    let cy = top + panel_h * 0.5;
    out.push(solid(
        [cx, cy],
        [panel_w, panel_h],
        [0.04, 0.10, 0.22, 0.90],
        8.0 * s,
        0.0,
    ));
    push_brackets(
        out,
        left,
        top,
        panel_w,
        panel_h,
        12.0 * s,
        1.6 * s,
        [0.75, 0.88, 1.0, 0.90],
    );
    text.draw_text(
        out,
        header,
        left + pad,
        top + pad + 8.0 * s,
        PX_FORMULA,
        [0.70, 0.85, 1.0, 1.0],
        text_scale,
    );
    for (i, line) in lines.iter().enumerate() {
        text.draw_text(
            out,
            line,
            left + pad,
            top + pad + 26.0 * s + i as f32 * line_h,
            PX_FORMULA,
            [1.0, 1.0, 1.0, 1.0],
            text_scale,
        );
    }
}

fn push_brackets(
    out: &mut Vec<HudInstance>,
    x0: f32,
    y0: f32,
    bw: f32,
    bh: f32,
    len: f32,
    th: f32,
    rgba: [f32; 4],
) {
    let x1 = x0 + bw;
    let y1 = y0 + bh;
    let segs = [
        ([x0 + len * 0.5, y0], [len, th]),
        ([x0, y0 + len * 0.5], [th, len]),
        ([x1 - len * 0.5, y0], [len, th]),
        ([x1, y0 + len * 0.5], [th, len]),
        ([x0 + len * 0.5, y1], [len, th]),
        ([x0, y1 - len * 0.5], [th, len]),
        ([x1 - len * 0.5, y1], [len, th]),
        ([x1, y1 - len * 0.5], [th, len]),
    ];
    for (pos, size) in segs {
        out.push(solid(pos, size, rgba, 0.0, 0.0));
    }
}

pub fn build_chrome(screen: (f32, f32), ocean_t: f64, out: &mut Vec<HudInstance>) {
    out.clear();
    let (w, h) = screen;
    if w < 8.0 || h < 8.0 {
        return;
    }
    let s = (w.min(h) / 1080.0).clamp(0.75, 1.35);
    let inset = 10.0 * s;
    let len = 28.0 * s;
    let th = 1.8 * s;
    let pulse = 0.70 + 0.18 * ((ocean_t * 1.1).sin() as f32);
    let rgba = [0.78, 0.90, 1.0, pulse];
    push_brackets(out, inset, inset, w - inset * 2.0, h - inset * 2.0, len, th, rgba);
    let tick = 7.0 * s;
    out.push(solid([w * 0.5, inset], [tick, th], rgba, 0.0, 0.0));
    out.push(solid([w * 0.5, h - inset], [tick, th], rgba, 0.0, 0.0));
    out.push(solid([inset, h * 0.5], [th, tick], rgba, 0.0, 0.0));
    out.push(solid([w - inset, h * 0.5], [th, tick], rgba, 0.0, 0.0));
}

pub fn build_hud(
    inst: &[Creature],
    geom: LegendGeom,
    text: &TextEngine,
    highlight: usize,
    seed: u32,
    ocean_t: f64,
    lang: &str,
    screen: (f32, f32),
    show_legend: bool,
    show_formula: bool,
    out: &mut Vec<HudInstance>,
) {
    if inst.is_empty() {
        return;
    }
    let layout = geom;
    let x0 = layout.x0;
    let y0 = layout.y0;
    let bw = layout.box_w;
    let bh = layout.box_h;
    let hi = highlight.min(inst.len() - 1);
    let s = layout.scale.clamp(0.82, 1.45);
    let en = lang.eq_ignore_ascii_case("en");
    let title = if en { "Legend" } else { "图例 / Legend" };
    let sub = if en {
        format!("seed {seed} · {}", inst.len())
    } else {
        format!("种子 seed {seed} · {}", inst.len())
    };

    if show_legend {
        out.push(solid(
            [x0 + bw * 0.5, y0 + bh * 0.5],
            [bw, bh],
            [0.04, 0.10, 0.22, 0.80],
            10.0 * s,
            0.0,
        ));
        push_brackets(
            out,
            x0,
            y0,
            bw,
            bh,
            14.0 * s,
            1.6 * s,
            [0.70, 0.85, 1.0, 0.75],
        );

        out.push(solid(
            [x0 + bw * 0.5, y0 + layout.head + (hi as f32 + 0.5) * layout.row_h],
            [bw - 10.0 * s, layout.row_h],
            [0.45, 0.70, 1.0, 0.16],
            0.0,
            0.0,
        ));

        text.draw_text(
            out,
            title,
            x0 + 12.0 * s,
            y0 + 14.0 * s,
            PX_TITLE,
            [0.85, 0.92, 1.0, 0.95],
            s * 0.82,
        );
        text.draw_text(
            out,
            &sub,
            x0 + 12.0 * s,
            y0 + 30.0 * s,
            PX_EN,
            [0.70, 0.80, 0.95, 0.70],
            s * 0.82,
        );

        let label_x = x0 + 48.0 * s;
        for (k, c) in inst.iter().enumerate() {
            let spec = &SPECIES[c.ci];
            let ly = y0 + layout.head + (k as f32 + 0.5) * layout.row_h;
            let on = k == hi;
            let depth = 0.45 + (1.0 - c.y as f32) * 0.55;
            let name_a = if on { 0.98 } else { 0.78 * depth.max(0.55) };
            let en_a = if on { 0.88 } else { 0.62 * depth.max(0.50) };
            let name_rgb = if on {
                [1.0, 1.0, 1.0]
            } else {
                [0.88, 0.92, 1.0]
            };
            let en_rgb = if on {
                [0.78, 0.88, 1.0]
            } else {
                [0.62, 0.74, 0.90]
            };
            text.draw_text(
                out,
                spec.name,
                label_x,
                ly - 5.0 * s,
                PX_NAME,
                [name_rgb[0], name_rgb[1], name_rgb[2], name_a],
                s * 0.78,
            );
            text.draw_text(
                out,
                spec.name_en,
                label_x,
                ly + 7.0 * s,
                PX_EN,
                [en_rgb[0], en_rgb[1], en_rgb[2], en_a],
                s * 0.78,
            );
        }

        let c = &inst[hi];
        let ly = y0 + layout.head + (hi as f32 + 0.5) * layout.row_h;
        let p0 = [x0 + bw - 8.0 * s, ly];
        let p1 = [x0 + bw + 48.0 * s, ly];
        let p2 = [(x0 + bw + c.ax) * 0.5, (ly + c.ay) * 0.5];
        let p3 = [c.ax, c.ay];
        let line_a = 0.28 + 0.2 * c.pulse as f32;
        push_dashed_bezier(
            out,
            p0,
            p1,
            p2,
            p3,
            ocean_t,
            [0.78, 0.88, 1.0, line_a],
            1.2 * s,
        );
        out.push(solid(p0, [4.4 * s, 4.4 * s], [0.92, 0.96, 1.0, 0.90], 2.2 * s, 0.0));
    }

    let c = &inst[hi];
    let spec = &SPECIES[c.ci];
    if show_legend || show_formula {
        let ring_r = c.radius.max(10.0);
        let ring = [0.92, 0.96, 1.0, 0.90];
        push_ring(out, c.ax, c.ay, ring_r, ring, 2.0);
        push_ring(out, c.ax, c.ay, ring_r + 8.0 * s, [0.55, 0.72, 1.0, 0.38], 1.2);
        push_ring_ticks(out, c.ax, c.ay, ring_r, [0.85, 0.92, 1.0, 0.88], s);
    }

    if !show_formula {
        return;
    }

    let ring_r = c.radius.max(10.0);
    let callout_w = 300.0 * s;
    let prefer_right = c.ax + ring_r + 18.0 + callout_w < screen.0 - 12.0;
    let callout_left = if prefer_right {
        c.ax + ring_r + 14.0
    } else {
        (c.ax - ring_r - 14.0 - callout_w).max(12.0)
    };
    let callout_top = (c.ay - 20.0 * s).clamp(12.0, (screen.1 - 160.0 * s).max(12.0));
    let short = wrapped_formulas(spec.formula, 40);
    let short: Vec<String> = short.into_iter().take(3).collect();
    push_formula_panel(
        out,
        text,
        if en { "FORMULA" } else { "公式" },
        &short,
        callout_left,
        callout_top,
        s,
        screen,
        1.0,
    );
}
