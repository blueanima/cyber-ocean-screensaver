#!/usr/bin/env python3
"""Watch school path CSVs and keep one live PNG up to date.

    python3 scripts/plot-paths-live.py
    python3 scripts/plot-paths-live.py --once
"""
from __future__ import annotations

import argparse
import csv
import math
import time
from collections import defaultdict
from pathlib import Path

import importlib.util

ROOT = Path(__file__).resolve().parents[1]
DIR = ROOT / ".cache" / "life-obs"
OUT = DIR / "paths-live.png"
STATUS = DIR / "score-status.txt"
JOURNAL = DIR / "journal.tsv"
THERMAL_LOG = DIR / "thermal.log"
THERMAL_CSV = DIR / "thermal.csv"
DEFAULT_GATE = 11.90

spec = importlib.util.spec_from_file_location("plot_gaits", ROOT / "scripts" / "plot-gaits.py")
g = importlib.util.module_from_spec(spec)
spec.loader.exec_module(g)


def hsv(h: float, s: float, v: float) -> tuple[int, int, int]:
    i = int(h * 6.0)
    f = h * 6.0 - i
    p = v * (1.0 - s)
    q = v * (1.0 - f * s)
    t = v * (1.0 - (1.0 - f) * s)
    r, g_, b = [(v, t, p), (q, v, p), (p, v, t), (p, q, v), (t, p, v), (v, p, q)][i % 6]
    return int(r * 255), int(g_ * 255), int(b * 255)


def color_for(i: int) -> tuple[int, int, int]:
    return hsv((i * 0.6180339887) % 1.0, 0.55, 0.95)


DIGITS = {
    "0": ("111", "101", "101", "101", "111"),
    "1": ("010", "110", "010", "010", "111"),
    "2": ("111", "001", "111", "100", "111"),
    "3": ("111", "001", "111", "001", "111"),
    "4": ("101", "101", "111", "001", "001"),
    "5": ("111", "100", "111", "001", "111"),
    "6": ("111", "100", "111", "101", "111"),
    "7": ("111", "001", "001", "001", "001"),
    "8": ("111", "101", "111", "101", "111"),
    "9": ("111", "101", "111", "001", "111"),
    "C": ("111", "100", "100", "100", "111"),
    " ": ("000", "000", "000", "000", "000"),
    ".": ("000", "000", "000", "000", "010"),
    "+": ("000", "010", "111", "010", "000"),
    "-": ("000", "000", "111", "000", "000"),
}


def draw_text(rgb, w, h, x, y, text, col, scale=2):
    x0 = x
    for ch in text:
        glyph = DIGITS.get(ch, DIGITS[" "])
        for r, row in enumerate(glyph):
            for c, bit in enumerate(row):
                if bit != "1":
                    continue
                for dy in range(scale):
                    for dx in range(scale):
                        g.put(rgb, w, h, x0 + c * scale + dx, y + r * scale + dy, col)
        x0 += 4 * scale


def newest_csv() -> Path | None:
    csvs = list(DIR.glob("paths-*.csv"))
    if not csvs:
        return None
    return max(csvs, key=lambda p: p.stat().st_mtime)


def lerp(a: tuple[int, int, int], b: tuple[int, int, int], t: float) -> tuple[int, int, int]:
    t = 0.0 if t < 0 else 1.0 if t > 1 else t
    return (
        int(a[0] + (b[0] - a[0]) * t),
        int(a[1] + (b[1] - a[1]) * t),
        int(a[2] + (b[2] - a[2]) * t),
    )


def load_temps() -> list[tuple[float, float]]:
    src = THERMAL_LOG if THERMAL_LOG.exists() else THERMAL_CSV
    if not src.exists():
        return []
    text = src.read_text(errors="replace")
    mark = text.rfind("---- ")
    chunk = text[mark:] if mark >= 0 else text
    out: list[tuple[float, float]] = []
    for line in chunk.splitlines():
        if "C hold=" in line:
            try:
                ts, rest = line.split(" ", 1)
                t = time.mktime(time.strptime(ts[:19], "%Y-%m-%dT%H:%M:%S"))
                c = float(rest.split("C", 1)[0])
                out.append((t, c))
            except ValueError:
                continue
            continue
        parts = line.strip().split(",")
        if len(parts) < 2 or parts[0] in ("ts", "thermal-watch start") or parts[0].startswith("----"):
            continue
        try:
            t = time.mktime(time.strptime(parts[0][:19], "%Y-%m-%dT%H:%M:%S"))
            out.append((t, float(parts[1])))
        except ValueError:
            continue
    return out


def smooth_temps(samples: list[tuple[float, float]], win: int = 11) -> list[tuple[float, float]]:
    if len(samples) < 3:
        return samples
    half = max(1, win // 2)
    out: list[tuple[float, float]] = []
    for i in range(len(samples)):
        lo = max(0, i - half)
        hi = min(len(samples), i + half + 1)
        avg = sum(s[1] for s in samples[lo:hi]) / (hi - lo)
        out.append((samples[i][0], avg))
    return out


def temp_color(c: float) -> tuple[int, int, int]:
    stops = (
        (50.0, (28, 48, 92)),
        (65.0, (36, 150, 170)),
        (75.0, (70, 210, 150)),
        (85.0, (230, 200, 70)),
        (92.0, (255, 140, 50)),
        (100.0, (255, 64, 64)),
    )
    if c <= stops[0][0]:
        return stops[0][1]
    for (c0, col0), (c1, col1) in zip(stops, stops[1:]):
        if c <= c1:
            return lerp(col0, col1, (c - c0) / (c1 - c0))
    return stops[-1][1]


def load_scores() -> list[tuple[int, float, float]]:
    if not JOURNAL.exists():
        return []
    out: list[tuple[int, float, float]] = []
    with JOURNAL.open() as f:
        for line in f:
            if line.startswith("cycle") or not line.strip():
                continue
            parts = line.split("\t")
            if len(parts) < 3:
                continue
            try:
                out.append((int(parts[0]), float(parts[1]), float(parts[2])))
            except ValueError:
                continue
    return out


def current_run(rows: list[tuple[int, float, float]]) -> list[tuple[int, float, float]]:
    start = 0
    for i, (cycle, _, _) in enumerate(rows):
        if cycle == 0:
            start = i
    return rows[start:]


def write_status(rows: list[tuple[int, float, float]], temps: list[tuple[float, float]]) -> str:
    run = current_run(rows)
    if not run:
        line = "score: no journal yet"
    else:
        cycle, elapsed, score = run[-1]
        champ = max(r[2] for r in run)
        gate = run[0][2] if run else DEFAULT_GATE
        delta = champ - gate
        sign = "+" if delta >= 0 else "-"
        temp = f"{temps[-1][1]:.0f}C" if temps else "?"
        line = (
            f"{time.strftime('%Y-%m-%dT%H:%M:%S')}  "
            f"cycle={cycle}  t={elapsed:.3f}h  "
            f"score={score:.3f}  champ={champ:.3f}  "
            f"gate={gate:.2f}  d={sign}{abs(delta):.3f}  {temp}"
        )
    STATUS.write_text(line + "\n")
    return line


def score_color(s: float) -> tuple[int, int, int]:
    return score_color_vs(s, DEFAULT_GATE)


def score_color_vs(s: float, gate: float) -> tuple[int, int, int]:
    if s > gate:
        return (120, 230, 140)
    if s >= gate - 0.01:
        return (230, 210, 90)
    return (80, 160, 200)


def score_trend(run: list[tuple[int, float, float]], n_bin: int) -> tuple[list[float], list[float], list[float]]:
    """Time-binned mean score, then a short moving average. Trend only, not per-cycle jitter."""
    t_end = run[-1][1]
    window = 8.0 / 60.0
    run = [r for r in run if r[1] >= t_end - window] or run
    t0, t1 = run[0][1], run[-1][1]
    span = max(t1 - t0, 1e-6)
    bins: list[list[float]] = [[] for _ in range(n_bin)]
    champs: list[list[float]] = [[] for _ in range(n_bin)]
    seen = run[0][2]
    for _cycle, elapsed, score in run:
        seen = max(seen, score)
        i = int((elapsed - t0) / span * (n_bin - 1))
        i = 0 if i < 0 else n_bin - 1 if i >= n_bin else i
        bins[i].append(score)
        champs[i].append(seen)
    xs: list[float] = []
    ys: list[float] = []
    hs: list[float] = []
    for i, bucket in enumerate(bins):
        if not bucket:
            continue
        xs.append(i / max(n_bin - 1, 1))
        ys.append(sum(bucket) / len(bucket))
        hs.append(max(champs[i]))
    if len(ys) < 5:
        return xs, ys, hs
    half = max(3, len(ys) // 40)
    sm: list[float] = []
    for i in range(len(ys)):
        lo = max(0, i - half)
        hi = min(len(ys), i + half + 1)
        sm.append(sum(ys[lo:hi]) / (hi - lo))
    return xs, sm, hs


def trend_color(cur: float, prev: float, gate: float) -> tuple[int, int, int]:
    if cur > prev + 1e-4:
        return (120, 230, 140)
    if cur < prev - 1e-4:
        return (255, 140, 90)
    return score_color_vs(cur, gate)


def draw_score(rgb, w, h, y0, y1, rows: list[tuple[int, float, float]]) -> None:
    pad = 28
    g.line(rgb, w, h, pad, y0, w - pad, y0, (40, 60, 90))
    run = current_run(rows)
    if not run:
        draw_text(rgb, w, h, pad, y0 + 8, "NO SCORE", (120, 140, 160), 2)
        return
    inner_w = w - 2 * pad
    inner_h = y1 - y0 - 22
    n_bin = max(int(inner_w), 8)
    xs, ys, hs = score_trend(run, n_bin)
    if not ys:
        return
    now = ys[-1]
    champ = max(r[2] for r in current_run(rows))
    gate = current_run(rows)[0][2]
    smin, smax = min(ys), max(ys)
    pad_y = max(0.012, (smax - smin) * 0.45)
    lo = smin - pad_y
    hi = smax + pad_y
    if hi - lo < 0.04:
        mid = 0.5 * (lo + hi)
        lo, hi = mid - 0.02, mid + 0.02

    def tx(u: float) -> float:
        return pad + u * inner_w

    def ty(s: float) -> float:
        u = (s - lo) / (hi - lo)
        return y1 - 10 - u * inner_h

    if lo <= gate <= hi:
        g.line(rgb, w, h, pad, ty(gate), w - pad, ty(gate), (90, 80, 30))

    def fat(x0, y_a, x1, y_b, c):
        g.line(rgb, w, h, x0, y_a, x1, y_b, c)
        g.line(rgb, w, h, x0, y_a + 1, x1, y_b + 1, c)
        g.line(rgb, w, h, x0 + 1, y_a, x1 + 1, y_b, c)
        g.line(rgb, w, h, x0, y_a - 1, x1, y_b - 1, c)
        g.line(rgb, w, h, x0, y_a + 2, x1, y_b + 2, c)

    for i in range(1, len(xs)):
        fat(tx(xs[i - 1]), ty(ys[i - 1]), tx(xs[i]), ty(ys[i]), trend_color(ys[i], ys[i - 1], gate))
    label = f"{now:.3f}"
    draw_text(rgb, w, h, w - pad - 4 * 6 * 2 - 4, y0 + 6, label, trend_color(now, ys[0], gate), 2)
    if champ > now + 1e-4:
        draw_text(rgb, w, h, pad, y0 + 6, f"{champ:.3f}", (120, 230, 140), 2)


def draw_temp(rgb, w, h, y0, y1, samples: list[tuple[float, float]]) -> None:
    pad = 28
    g.line(rgb, w, h, pad, y0, w - pad, y0, (40, 60, 90))
    if not samples:
        draw_text(rgb, w, h, pad, y0 + 8, "NO TEMP", (120, 140, 160), 2)
        return
    t_end = samples[-1][0]
    samples = [s for s in samples if s[0] >= t_end - 8 * 60.0]
    samples = smooth_temps(samples, 11)
    t0, t1 = samples[0][0], samples[-1][0]
    span = max(t1 - t0, 30.0)
    cs = [s[1] for s in samples]
    cmin, cmax = min(cs), max(cs)
    lo = cmin - 6.0
    hi = cmax + 6.0
    if hi - lo < 18.0:
        mid = 0.5 * (lo + hi)
        lo, hi = mid - 9.0, mid + 9.0
    lo = max(35.0, lo)
    hi = min(110.0, hi)
    now = samples[-1][1]
    inner_w = w - 2 * pad
    inner_h = y1 - y0 - 22

    def tx(t):
        return pad + (t - t0) / span * inner_w

    def ty(c):
        u = (c - lo) / (hi - lo)
        return y1 - 10 - u * inner_h

    # 每个像素一列，避免 1Hz 锯齿叠成竖线
    n_bin = max(int(inner_w), 8)
    bins: list[list[float]] = [[] for _ in range(n_bin)]
    for t, c in samples:
        i = int((t - t0) / span * (n_bin - 1))
        i = 0 if i < 0 else n_bin - 1 if i >= n_bin else i
        bins[i].append(c)
    xs: list[float] = []
    ys: list[float] = []
    vs: list[float] = []
    for i, bucket in enumerate(bins):
        if not bucket:
            continue
        c = sum(bucket) / len(bucket)
        xs.append(pad + i / max(n_bin - 1, 1) * inner_w)
        ys.append(ty(c))
        vs.append(c)

    base = y1 - 10
    for i in range(len(xs)):
        x = int(xs[i])
        y = int(ys[i])
        fill = temp_color(vs[i])
        fill = (8 + fill[0] // 6, 10 + fill[1] // 6, 16 + fill[2] // 6)
        y_top = y if y < base else int(base)
        y_bot = int(base)
        if y_top > y_bot:
            y_top, y_bot = y_bot, y_top
        for yy in range(y_top, y_bot):
            g.put(rgb, w, h, x, yy, fill)

    for mark in (70, 85, 95):
        if mark < lo or mark > hi:
            continue
        yy = ty(mark)
        col = (90, 42, 42) if mark == 95 else (28, 42, 64)
        g.line(rgb, w, h, pad, yy, w - pad, yy, col)
        draw_text(rgb, w, h, 4, int(yy) - 5, str(mark), (90, 110, 130), 1)

    def fat(x0, y_a, x1, y_b, c):
        g.line(rgb, w, h, x0, y_a, x1, y_b, c)
        g.line(rgb, w, h, x0, y_a + 1, x1, y_b + 1, c)
        g.line(rgb, w, h, x0 + 1, y_a, x1 + 1, y_b, c)
        g.line(rgb, w, h, x0, y_a - 1, x1, y_b - 1, c)

    for i in range(1, len(xs)):
        fat(xs[i - 1], ys[i - 1], xs[i], ys[i], temp_color(vs[i]))
    draw_text(rgb, w, h, w - pad - 52, y0 + 6, f"{now:3.0f}C", temp_color(now), 3)


def load_trails(csv_path: Path) -> dict[int, list[tuple[float, float, float, str]]]:
    trails: dict[int, list[tuple[float, float, float, str]]] = defaultdict(list)
    with csv_path.open() as f:
        for row in csv.DictReader(f):
            try:
                trails[int(row["i"])].append(
                    (float(row["x"]), float(row["y"]), float(row["rot"]), row["id"])
                )
            except (TypeError, ValueError, KeyError):
                continue
    return trails


def recent_path_csvs(n: int = 3) -> list[Path]:
    csvs = [p for p in DIR.glob("paths-*.csv") if p.stat().st_size > 2000]
    csvs.sort(key=lambda p: p.stat().st_mtime)
    return csvs[-n:]


def plot_school(csv_path: Path, png_path: Path, w=900, h=920) -> bool:
    layers = []
    for p in recent_path_csvs(3):
        tr = load_trails(p)
        if sum(len(v) for v in tr.values()) >= 80:
            layers.append(tr)
    if not layers:
        trails = load_trails(csv_path)
        if sum(len(v) for v in trails.values()) < 80:
            return False
        layers = [trails]
    rgb = bytearray([8, 14, 28]) * (w * h)
    pad = 28
    school_h = 520
    score_h = 680

    def to_px(x: float, y: float) -> tuple[float, float]:
        px = pad + x * (w - 2 * pad)
        py = pad + y * (school_h - 2 * pad)
        return px, py

    g.line(rgb, w, h, *to_px(0, 0), *to_px(1, 0), (40, 60, 90))
    g.line(rgb, w, h, *to_px(0, 0), *to_px(0, 1), (40, 60, 90))
    g.line(rgb, w, h, *to_px(1, 1), *to_px(1, 0), (40, 60, 90))
    g.line(rgb, w, h, *to_px(1, 1), *to_px(0, 1), (40, 60, 90))

    def fat_line(x0, y0, x1, y1, c, width=2):
        g.line(rgb, w, h, x0, y0, x1, y1, c)
        if width >= 2:
            g.line(rgb, w, h, x0, y0 + 1, x1, y1 + 1, c)
            g.line(rgb, w, h, x0 + 1, y0, x1 + 1, y1, c)
        if width >= 3:
            g.line(rgb, w, h, x0, y0 - 1, x1, y1 - 1, c)
            g.line(rgb, w, h, x0 - 1, y0, x1 - 1, y1, c)

    def chaikin(xy: list[tuple[float, float]]) -> list[tuple[float, float]]:
        pts = xy
        for _ in range(2):
            if len(pts) < 3:
                break
            out = [pts[0]]
            for a, b in zip(pts, pts[1:]):
                out.append((0.75 * a[0] + 0.25 * b[0], 0.75 * a[1] + 0.25 * b[1]))
                out.append((0.25 * a[0] + 0.75 * b[0], 0.25 * a[1] + 0.75 * b[1]))
            out.append(pts[-1])
            pts = out
        return pts

    def drop_jitter(xy: list[tuple[float, float]]) -> list[tuple[float, float]]:
        if not xy:
            return xy
        out = [xy[0]]
        for p in xy[1:]:
            if math.hypot(p[0] - out[-1][0], p[1] - out[-1][1]) >= 0.0007:
                out.append(p)
        if out[-1] != xy[-1]:
            out.append(xy[-1])
        return out

    for li, trails in enumerate(layers):
        newest = li == len(layers) - 1
        layer_fade = 0.22 + 0.78 * (li / max(len(layers) - 1, 1))
        width = 3 if newest else 2
        for i, pts in trails.items():
            col = color_for(i)
            raw = [(p[0], p[1]) for p in pts]
            travel = 0.0
            for a, b in zip(raw, raw[1:]):
                travel += math.hypot(b[0] - a[0], b[1] - a[1])
            if travel >= 0.012:
                sm = chaikin(drop_jitter(raw))
                n = max(len(sm) - 1, 1)
                for k in range(1, len(sm)):
                    u = (k / n) ** 0.65
                    c = lerp((12, 18, 32), col, u * layer_fade)
                    fat_line(*to_px(*sm[k - 1]), *to_px(*sm[k]), c, width)
            if not newest:
                continue
            x, y, rot, _name = pts[-1]
            px, py = to_px(x, y)
            fx, fy = math.cos(rot), -math.sin(rot)
            g.line(rgb, w, h, px, py, px + fx * 11, py + fy * 11, (255, 220, 120))
            for dx, dy in ((0, 0), (1, 0), (0, 1), (-1, 0), (0, -1)):
                g.put(rgb, w, h, int(px) + dx, int(py) + dy, (255, 255, 255))

    scores = load_scores()
    temps = load_temps()
    draw_score(rgb, w, h, school_h, score_h, scores)
    draw_temp(rgb, w, h, score_h, h, temps)
    print(write_status(scores, temps), flush=True)

    tmp = png_path.with_suffix(".tmp.png")
    g.write_png(tmp, w, h, rgb)
    tmp.replace(png_path)
    return True


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--once", action="store_true")
    ap.add_argument("--interval", type=float, default=2.0)
    args = ap.parse_args()
    DIR.mkdir(parents=True, exist_ok=True)
    last = None
    while True:
        src = newest_csv()
        tlog = THERMAL_LOG.stat().st_mtime if THERMAL_LOG.exists() else 0
        jlog = JOURNAL.stat().st_mtime if JOURNAL.exists() else 0
        key = (src, src.stat().st_mtime if src else 0, tlog, jlog)
        if src and key != last:
            try:
                if plot_school(src, OUT):
                    last = key
                    print(f"live {src.name} + score + temp -> {OUT.name}", flush=True)
            except (OSError, ValueError, TypeError) as e:
                print(f"skip {src}: {e}", flush=True)
        if args.once:
            break
        time.sleep(args.interval)


if __name__ == "__main__":
    main()
