#!/usr/bin/env python3
"""Head/tail contact sheet. Kinds match native formulas::HeadingKind.

    python3 scripts/heading-audit/heads_tails.py
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import dump as d
import ttfdraw

T = 1.2
FACES = [
    -1.583, 2.221, 1.540, 0.125, 1.563,
    2.473 + 0.114 * T,
    1.977, 1.962, 0.0, 0.0, 2.287, 1.305, 1.783, 0.0, 2.390, 1.885, 1.666,
]
NAMES = [
    "０１ 北斗浮蚕", "０２ 蚰蜒", "０３ 脊虫", "０４ 小水母", "０５ 星云水母",
    "０６ 花水母", "０７ 羽鳃", "０８ 触须虫", "０９ 六瓣花", "１０ 轮虫花",
    "１１ 螺灯", "１２ 栉水母", "１３ 锯鳗", "１４ 八腕星", "１５ 磷虾",
    "１６ 涡虫", "１７ 海天使",
]
RADIAL = {"flower6", "wheel", "star8"}
# 与 native/src/formulas.rs HeadingKind 对齐
KIND = {
    "fucan": "spine_no_legs",
    "youyan": "spine_no_legs",
    "jichong": "spine",
    "jelly": "bell",
    "nebula": "bell",
    "lantern": "bell",
    "feather": "spine",
    "tentacle": "spine",
    "flower6": "radial",
    "wheel": "radial",
    "spiral": "spine",
    "comb": "bell",
    "saweel": "spine",
    "star8": "radial",
    "shrimp": "spine",
    "vortex": "spine",
    "angel": "torso",
}


def fill_flower6(t, step=3):
    out = []
    copies, ang = 6, math.pi / 3.0
    i = 1
    while i <= 5000:
        k = float(i % 25) - 12.0
        e = i / 800.0
        dd = 7.0 * math.cos(math.sqrt(k * k + e * e) / 3.0 + t / 2.0)
        bx = k * 4.0 + dd * k * math.sin(dd + e / 9.0 + t)
        by = e * 2.0 - dd * 9.0 - dd * 9.0 * math.cos(dd + t)
        for j in range(copies):
            a = j * ang
            ca, sa = math.cos(a), math.sin(a)
            d.push(out, ca * bx - sa * by + 200.0, sa * bx + ca * by + 200.0)
        i += step
    return out


def fill_wheel(t, step=3):
    out = []
    copies, ang = 14, math.pi / 7.0
    i = 1
    while i <= 3500:
        k = float(i % 50) - 25.0
        e = i / 1100.0
        dd = 5.0 * math.cos(math.sqrt(k * k + e * e) - t + (i % 2))
        if abs(dd) < 0.12:
            i += step
            continue
        bx = k + k * dd / 6.0 * math.sin(dd + e / 3.0 + t)
        by = 90.0 + e * dd - e / dd * 2.0 * math.cos(dd + t)
        for j in range(copies):
            a = j * ang
            ca, sa = math.cos(a), math.sin(a)
            d.push(out, ca * bx - sa * by + 200.0, sa * bx + ca * by + 200.0)
        i += step
    return out


def fill_star8(t, step=3):
    out = []
    copies, ang = 8, math.pi / 4.0
    i = 1
    while i <= 4000:
        k = float(i % 20) - 10.0
        e = i / 900.0
        dd = 6.0 * math.cos(math.sqrt(k * k + e * e) / 4.0 + t / 3.0)
        bx = 5.0 * k + dd * k * math.sin(dd + t)
        by = 2.5 * e - 8.0 * dd * math.cos(dd + e / 8.0 + t)
        for j in range(copies):
            a = j * ang
            ca, sa = math.cos(a), math.sin(a)
            d.push(out, ca * bx - sa * by + 200.0, sa * bx + ca * by + 200.0)
        i += step
    return out


d.FILLS["flower6"] = fill_flower6
d.FILLS["wheel"] = fill_wheel
d.FILLS["star8"] = fill_star8


def _pct(vals, p):
    s = sorted(vals)
    i = int((len(s) - 1) * p)
    return s[max(0, min(len(s) - 1, i))]


def _grid_density(xy, cells=56):
    xs = [p[0] for p in xy]
    ys = [p[1] for p in xy]
    minx, maxx = _pct(xs, 0.01), _pct(xs, 0.99)
    miny, maxy = _pct(ys, 0.01), _pct(ys, 0.99)
    sx = max(maxx - minx, 1e-6) / (cells - 1)
    sy = max(maxy - miny, 1e-6) / (cells - 1)
    grid = [[0] * cells for _ in range(cells)]
    idx = []
    for x, y in xy:
        ix = min(cells - 1, max(0, int((x - minx) / sx)))
        iy = min(cells - 1, max(0, int((y - miny) / sy)))
        grid[iy][ix] += 1
        idx.append((ix, iy))
    return [grid[iy][ix] for ix, iy in idx]


def _pca_dir(xy):
    n = len(xy)
    mx = sum(p[0] for p in xy) / n
    my = sum(p[1] for p in xy) / n
    xx = xy_ = yy = 0.0
    for x, y in xy:
        dx, dy = x - mx, y - my
        xx += dx * dx
        xy_ += dx * dy
        yy += dy * dy
    ang = 0.5 * math.atan2(2.0 * xy_, xx - yy)
    ca, sa = math.cos(ang), math.sin(ang)
    v_par = v_perp = 0.0
    for x, y in xy:
        dx, dy = x - mx, y - my
        v_par += (dx * ca + dy * sa) ** 2
        v_perp += (-dx * sa + dy * ca) ** 2
    if v_perp > v_par:
        ca, sa = -sa, ca
    return mx, my, ca, sa


def _orient_ridge(ridge):
    if len(ridge) < 2:
        return ridge
    if ridge[-1][0] < ridge[0][0]:
        ridge = list(reversed(ridge))
    elif abs(ridge[-1][0] - ridge[0][0]) < 8.0 and ridge[-1][1] > ridge[0][1]:
        ridge = list(reversed(ridge))
    return ridge


def _snap_to_ridge(spine, xy, half=14.0):
    """把先验脊椎垂直方向吸到点最密的那条脊上。"""
    if len(spine) < 3:
        return spine
    cells = 72
    xs = [p[0] for p in xy]
    ys = [p[1] for p in xy]
    minx, maxx = min(xs), max(xs)
    miny, maxy = min(ys), max(ys)
    sx = max(maxx - minx, 1e-6) / (cells - 1)
    sy = max(maxy - miny, 1e-6) / (cells - 1)
    g = [[0] * cells for _ in range(cells)]
    for x, y in xy:
        ix = min(cells - 1, max(0, int((x - minx) / sx)))
        iy = min(cells - 1, max(0, int((y - miny) / sy)))
        g[iy][ix] += 1
    out = []
    for i, (x, y) in enumerate(spine):
        if i == 0:
            tx, ty = spine[1][0] - x, spine[1][1] - y
        elif i == len(spine) - 1:
            tx, ty = x - spine[i - 1][0], y - spine[i - 1][1]
        else:
            tx, ty = spine[i + 1][0] - spine[i - 1][0], spine[i + 1][1] - spine[i - 1][1]
        L = math.hypot(tx, ty) or 1.0
        nx, ny = -ty / L, tx / L
        best = (-1, x, y)
        for k in range(-12, 13):
            px = x + nx * (k / 12.0) * half
            py = y + ny * (k / 12.0) * half
            ix = min(cells - 1, max(0, int((px - minx) / sx)))
            iy = min(cells - 1, max(0, int((py - miny) / sy)))
            d = g[iy][ix]
            if d > best[0]:
                best = (d, px, py)
        out.append((best[1], best[2]))
    sm = []
    for i in range(len(out)):
        lo, hi = max(0, i - 5), min(len(out), i + 6)
        chunk = out[lo:hi]
        sm.append((sum(p[0] for p in chunk) / len(chunk), sum(p[1] for p in chunk) / len(chunk)))
    return sm


def youyan_spine(t, face):
    """蚰蜒去掉附肢后的身体曲线，就是中间那根脊椎。"""
    rows = {}
    raw = []
    i = 0
    while i < 18000:
        x = float(i % 100)
        y = float(i // 100)
        k = x / 4.0 - 12.5
        e = y / 9.0 + 5.0
        o = math.sqrt(k * k + e * e) / 9.0
        if abs(k) < 1e-6:
            i += 4
            continue
        osc = o * k * (math.cos(e * 9.0) / 4.0 + math.cos(y / 2.0)) * math.sin(o * 4.0 - t)
        q = x + 99.0 + math.tan(1.0 / k) + osc
        q0 = x + 99.0 + osc
        c = o * e / 30.0 - t / 8.0
        px = q * 0.7 * math.sin(c) + 9.0 * math.cos(y / 19.0 + t) + 200.0
        py = 200.0 + q / 2.0 * math.cos(c)
        px0 = q0 * 0.7 * math.sin(c) + 9.0 * math.cos(y / 19.0 + t) + 200.0
        py0 = 200.0 + q0 / 2.0 * math.cos(c)
        if math.isfinite(px) and math.isfinite(py):
            raw.append((px, py))
        if math.isfinite(px0) and math.isfinite(py0):
            rows.setdefault(int(y), []).append((px0, py0))
        i += 4
    q0_pts = [p for recs in rows.values() for p in recs]
    spine = []
    for y in sorted(rows):
        recs = rows[y]
        xs = sorted(p[0] for p in recs)
        ys = sorted(p[1] for p in recs)
        spine.append((xs[len(xs) // 2], ys[len(ys) // 2]))
    # 与身体同一套朝向变换
    cx = sum(p[0] for p in raw) / len(raw)
    cy = sum(p[1] for p in raw) / len(raw)
    ca, sa = math.cos(-face), math.sin(-face)

    def xf(p):
        return ((p[0] - cx) * ca - (p[1] - cy) * sa, (p[0] - cx) * sa + (p[1] - cy) * ca)

    xf_spine = [xf(p) for p in spine]
    sm = []
    for i in range(len(xf_spine)):
        lo, hi = max(0, i - 3), min(len(xf_spine), i + 4)
        chunk = xf_spine[lo:hi]
        sm.append((sum(p[0] for p in chunk) / len(chunk), sum(p[1] for p in chunk) / len(chunk)))
    body = [xf(p) for p in q0_pts]
    return _orient_ridge(_snap_to_ridge(sm, body, half=8.0))


def _robust_center(recs, keep=0.55):
    pts = list(recs)
    floor = max(6, int(len(recs) * keep))
    while len(pts) > floor:
        mx = sorted(p[0] for p in pts)[len(pts) // 2]
        my = sorted(p[1] for p in pts)[len(pts) // 2]
        pts.sort(key=lambda p: (p[0] - mx) ** 2 + (p[1] - my) ** 2)
        pts.pop()
    return (
        sorted(p[0] for p in pts)[len(pts) // 2],
        sorted(p[1] for p in pts)[len(pts) // 2],
    )


def _xform_pts(pts, raw, face):
    cx = sum(p[0] for p in raw) / len(raw)
    cy = sum(p[1] for p in raw) / len(raw)
    ca, sa = math.cos(-face), math.sin(-face)
    return [((p[0] - cx) * ca - (p[1] - cy) * sa, (p[0] - cx) * sa + (p[1] - cy) * ca) for p in pts]


def _k0_jelly(t):
    out = []
    i = 0
    while i < 10000:
        y = i / 43.0
        e = y / 8.0 - 13.0
        d = (e * e) / 59.0 + 4.0
        a = math.atan2(0.0, e)
        q = 60.0 - 3.0 * math.sin(a * e)
        c = d / 2.0 + e / 99.0 - t / 18.0
        out.append((q * math.sin(c) + 200.0, (q + d * 9.0) * math.cos(c) + 200.0))
        i += 12
    return out


def _k0_comb(t):
    out = []
    i = 0
    while i < 10000:
        y = i / 50.0
        e = y / 8.0 - 12.0
        d = (e * e) / 70.0 + 3.0
        a = math.atan2(0.0, e)
        q = 48.0 - 4.0 * math.sin(a * 4.0)
        c = d / 2.4 + e / 85.0 - t / 14.0
        out.append((q * math.sin(c) + 200.0, (q + 7.0 * d) * math.cos(c) * 0.78 + 210.0))
        i += 12
    return out


def _k0_lantern(t):
    out = []
    i = 0
    while i < 10000:
        y = i / 55.0
        e = y / 8.0 - 12.5
        d = (e * e) / 99.0 + math.sin(t) / 6.0 + 0.5
        if abs(d) < 1e-6:
            i += 12
            continue
        a = math.atan2(0.0, e)
        q = 99.0 - e * math.sin(a * 7.0) / d
        c = d / 2.0 + e / 69.0 - t / 16.0
        out.append((q * math.sin(c) + 200.0, (q + 19.0 * d) * math.cos(c) + 200.0))
        i += 12
    return out


def _k0_angel(t):
    out = []
    i = 0
    while i < 10000:
        y = i / 65.0
        e = y / 9.0 - 11.0
        d = (e * e) / 68.0 + 2.4
        a = math.atan2(0.0, e)
        q = 38.0 - 5.0 * math.sin(a * 3.0)
        c = d / 2.1 + e / 88.0 - t / 15.0
        out.append(
            (
                q * math.sin(c) + 200.0,
                (q + 6.5 * d) * math.cos(c) + 8.0 * math.sin(e * 0.5 + t) + 205.0,
            )
        )
        i += 12
    return out


def _path_len(poly, a, b):
    if a > b:
        a, b = b, a
    s = 0.0
    for i in range(a + 1, b + 1):
        s += math.hypot(poly[i][0] - poly[i - 1][0], poly[i][1] - poly[i - 1][1])
    return s


def _extend_tips(ridge, head_xy, tail_xy, head_at="east"):
    """把头尾从中线内部推到点云真正的两端。"""
    if len(ridge) < 2:
        return ridge
    ridge = list(ridge)

    def outward(i_from, i_to):
        dx = ridge[i_to][0] - ridge[i_from][0]
        dy = ridge[i_to][1] - ridge[i_from][1]
        L = math.hypot(dx, dy) or 1.0
        return dx / L, dy / L

    def push(end, ux, uy, cloud):
        xs = [p[0] for p in cloud]
        ys = [p[1] for p in cloud]
        span = max(max(xs) - min(xs), max(ys) - min(ys), 8.0)
        tube = 0.14 * span
        best, best_t = end, 0.0
        for p in cloud:
            t = (p[0] - end[0]) * ux + (p[1] - end[1]) * uy
            s = abs(-(p[0] - end[0]) * uy + (p[1] - end[1]) * ux)
            if s <= tube and t > best_t:
                best_t = t
                best = p
        return best

    ridge[0] = push(ridge[0], *outward(1, 0), tail_xy)
    if head_at == "down":
        ridge[-1] = push(ridge[-1], 0.0, 1.0, head_xy)
    elif head_at == "up":
        ridge[-1] = push(ridge[-1], 0.0, -1.0, head_xy)
    else:
        ridge[-1] = push(ridge[-1], 1.0, 0.0, head_xy)
    return ridge


def polar_midline(k0, raw, face, head_at):
    """水母/栉水母/海天使：k=0 才是身体中线。头在指定一端，尾在中线另一端。"""
    poly = _xform_pts(k0, raw, face)
    if head_at == "down":
        ih = max(range(len(poly)), key=lambda i: poly[i][1])
    elif head_at == "up":
        ih = min(range(len(poly)), key=lambda i: poly[i][1])
    else:
        ih = max(range(len(poly)), key=lambda i: poly[i][0])
    i0, i1 = 0, len(poly) - 1
    it = i1 if _path_len(poly, ih, i1) >= _path_len(poly, ih, i0) else i0
    lo, hi = (ih, it) if ih < it else (it, ih)
    ridge = poly[lo : hi + 1]
    if abs(ridge[-1][0] - poly[ih][0]) + abs(ridge[-1][1] - poly[ih][1]) > 1e-6:
        ridge = list(reversed(ridge))
    body = _xform_pts(raw, raw, face)
    if head_at == "down":
        return _extend_tips(ridge, body, ridge, head_at)
    return _extend_tips(ridge, poly, ridge, head_at)


def param_spine(tagged, face, near_midline=True):
    """按身体参数一节一节取中线上的点，连成真实脊椎。"""
    raw = [(p[0], p[1]) for p in tagged]
    us = [p[2] for p in tagged if len(p) > 2]
    if len(us) < 30:
        return spine_curve(d.xform(raw, face))
    umin, umax = _pct(us, 0.03), _pct(us, 0.97)
    if umax - umin < 1e-6:
        return spine_curve(d.xform(raw, face))
    nb = 64
    bins = [[] for _ in range(nb)]
    for p in tagged:
        b = int((p[2] - umin) / (umax - umin) * (nb - 1e-9))
        b = max(0, min(nb - 1, b))
        bins[b].append(p)
    spine = []
    for recs in bins:
        if len(recs) < 6:
            continue
        if len(recs[0]) > 3:
            mag = [abs(p[3]) for p in recs]
            if near_midline:
                thr = _pct(mag, 0.28)
                core = [(p[0], p[1]) for p in recs if abs(p[3]) <= max(thr, 1e-9)]
            else:
                thr = _pct(mag, 0.62)
                core = [(p[0], p[1]) for p in recs if abs(p[3]) >= thr]
            if len(core) < 4:
                core = [(p[0], p[1]) for p in recs]
        else:
            core = [(p[0], p[1]) for p in recs]
        spine.append(_robust_center(core, keep=0.7))
    if len(spine) < 3:
        return spine_curve(d.xform(raw, face))
    cx = sum(p[0] for p in raw) / len(raw)
    cy = sum(p[1] for p in raw) / len(raw)
    ca, sa = math.cos(-face), math.sin(-face)

    def xf(p):
        return ((p[0] - cx) * ca - (p[1] - cy) * sa, (p[0] - cx) * sa + (p[1] - cy) * ca)

    xf_spine = [xf(p) for p in spine]
    sm = []
    for i in range(len(xf_spine)):
        lo, hi = max(0, i - 3), min(len(xf_spine), i + 4)
        chunk = xf_spine[lo:hi]
        sm.append((sum(p[0] for p in chunk) / len(chunk), sum(p[1] for p in chunk) / len(chunk)))
    return _orient_ridge(sm)


def spine_curve(xy):
    """备用：沿最密脊走。"""
    dens = _grid_density(xy)
    thr = _pct(dens, 0.70)
    core = [p for p, den in zip(xy, dens) if den >= thr]
    if len(core) < 40:
        core = list(xy)
    mx, my, ca, sa = _pca_dir(core)
    items = []
    for x, y in core:
        tt = (x - mx) * ca + (y - my) * sa
        s = -(x - mx) * sa + (y - my) * ca
        items.append((tt, s))
    items.sort()
    n = len(items)
    win = max(16, n // 16)
    step = max(1, n // 32)
    ridge = []
    for i in range(0, n, step):
        lo = max(0, i - win)
        hi = min(n, i + win + 1)
        chunk = items[lo:hi]
        ts = sorted(c[0] for c in chunk)
        ss = sorted(c[1] for c in chunk)
        tm, sm = ts[len(ts) // 2], ss[len(ss) // 2]
        ridge.append((mx + ca * tm - sa * sm, my + sa * tm + ca * sm))
    return _orient_ridge(ridge)


def draw_creature(xy, w, h, radial, ridge=None):
    rgb = bytearray(w * h * 3)
    for i in range(0, w * h * 3, 3):
        rgb[i], rgb[i + 1], rgb[i + 2] = 6, 14, 28

    def put(ix, iy, r, g, b, a=1.0):
        if 0 <= ix < w and 0 <= iy < h:
            o = (iy * w + ix) * 3
            rgb[o] = int(rgb[o] * (1 - a) + r * a)
            rgb[o + 1] = int(rgb[o + 1] * (1 - a) + g * a)
            rgb[o + 2] = int(rgb[o + 2] * (1 - a) + b * a)

    xs = sorted(p[0] for p in xy)
    ys = sorted(p[1] for p in xy)
    n = len(xs)
    minx, maxx = xs[n // 40], xs[n * 39 // 40]
    miny, maxy = ys[n // 40], ys[n * 39 // 40]
    span = max(maxx - minx, maxy - miny, 8.0)
    vx, vy = (minx + maxx) * 0.5, (miny + maxy) * 0.5
    pad = 44
    sc = (min(w, h) - 2 * pad) / span

    def to_px(x, y):
        return int((x - vx) * sc + w * 0.5), int((y - vy) * sc + h * 0.5)

    if radial:
        for x, y in xy:
            ix, iy = to_px(x, y)
            put(ix, iy, 210, 220, 235, 0.5)
        ttfdraw.draw_text(put, 12, h - 10, "无头", (170, 190, 210), 18)
        return rgb

    if ridge is None:
        ridge = spine_curve(xy)
    hx, hy = ridge[-1]
    tx, ty = ridge[0]
    # 头尾附近半径：脊椎长度的一小段
    span2 = math.hypot(hx - tx, hy - ty) or 1.0
    end_r = 0.10 * span2
    tube = 0.045 * span2

    def dist_to_poly(x, y):
        best = 1e18
        for i in range(len(ridge) - 1):
            ax, ay = ridge[i]
            bx, by = ridge[i + 1]
            vx, vy = bx - ax, by - ay
            ll = vx * vx + vy * vy or 1.0
            u = max(0.0, min(1.0, ((x - ax) * vx + (y - ay) * vy) / ll))
            dx, dy = ax + u * vx - x, ay + u * vy - y
            best = min(best, dx * dx + dy * dy)
        return math.sqrt(best)

    for x, y in xy:
        ix, iy = to_px(x, y)
        dh = math.hypot(x - hx, y - hy)
        dt = math.hypot(x - tx, y - ty)
        on_spine = dist_to_poly(x, y) <= tube
        if dh <= end_r and on_spine:
            put(ix, iy, 255, 70, 55, 0.85)
        elif dt <= end_r and on_spine:
            put(ix, iy, 40, 200, 230, 0.8)
        else:
            put(ix, iy, 200, 210, 225, 0.40)

    # 中线：沿脊椎折线
    for i in range(len(ridge) - 1):
        x0, y0 = to_px(*ridge[i])
        x1, y1 = to_px(*ridge[i + 1])
        steps = max(abs(x1 - x0), abs(y1 - y0), 1)
        for k in range(steps + 1):
            u = k / steps
            put(int(x0 + (x1 - x0) * u), int(y0 + (y1 - y0) * u), 255, 220, 90, 0.45)

    pxh, pyh = to_px(hx, hy)
    pxt, pyt = to_px(tx, ty)
    ttfdraw.draw_text(put, pxh + 6, pyh - 2, "头", (255, 90, 70), 20)
    ttfdraw.draw_text(put, pxt - 22, pyt + 18, "尾", (50, 210, 240), 20)
    return rgb


def main():
    cols, rows = 5, 4
    pw, ph = 360, 260
    header = 48
    W, H = pw * cols, header + ph * rows
    canvas = bytearray(W * H * 3)
    for i in range(0, W * H * 3, 3):
        canvas[i], canvas[i + 1], canvas[i + 2] = 4, 10, 22

    def put(ix, iy, r, g, b, a=1.0):
        if 0 <= ix < W and 0 <= iy < H:
            o = (iy * W + ix) * 3
            canvas[o] = int(canvas[o] * (1 - a) + r * a)
            canvas[o + 1] = int(canvas[o + 1] * (1 - a) + g * a)
            canvas[o + 2] = int(canvas[o + 2] * (1 - a) + b * a)

    ttfdraw.draw_text(put, 16, 32, "黄线沿身体脊椎，头尾在脊椎两端。红是头，青是尾", (235, 238, 245), 18)

    for i, sid in enumerate(d.IDS):
        r, c = divmod(i, cols)
        pts = d.FILLS[sid](T)
        face = FACES[i]
        xy = [(p[0], p[1]) for p in d.xform(pts, face)]
        kind = KIND.get(sid, "spine")
        if kind == "radial":
            ridge = None
        elif kind == "bell":
            k0 = {"jelly": _k0_jelly, "lantern": _k0_lantern, "comb": _k0_comb}.get(sid)
            ridge = (
                polar_midline(k0(T), [(p[0], p[1]) for p in pts], face, "down")
                if k0
                else param_spine(pts, face)
            )
        elif kind == "torso":
            ridge = polar_midline(_k0_angel(T), [(p[0], p[1]) for p in pts], face, "east")
        elif kind == "spine_no_legs":
            ridge = youyan_spine(T, face) if sid == "youyan" else param_spine(pts, face, near_midline=False)
        else:
            ridge = param_spine(pts, face)
        panel = draw_creature(xy, pw, ph - 32, sid in RADIAL, ridge)
        y0 = header + r * ph
        x0 = c * pw
        for y in range(ph - 32):
            src = y * pw * 3
            dst = ((y0 + 28 + y) * W + x0) * 3
            canvas[dst : dst + pw * 3] = panel[src : src + pw * 3]
        for y in range(28):
            for x in range(pw):
                put(x0 + x, y0 + y, 10, 22, 40, 1.0)
        ttfdraw.draw_text(put, x0 + 8, y0 + 22, NAMES[i], (235, 238, 245), 18)
        for y in range(28, ph):
            put(x0, y0 + y, 30, 60, 90, 1.0)

    r, c = divmod(17, cols)
    x0, y0 = c * pw, header + r * ph
    ttfdraw.draw_text(put, x0 + 16, y0 + 50, "请判断每只", (240, 240, 245), 20)
    ttfdraw.draw_text(put, x0 + 16, y0 + 88, "红端是头", (255, 90, 70), 20)
    ttfdraw.draw_text(put, x0 + 16, y0 + 122, "青端是尾", (50, 210, 240), 20)
    ttfdraw.draw_text(put, x0 + 16, y0 + 156, "黄线是脊椎", (255, 220, 80), 20)
    ttfdraw.draw_text(put, x0 + 16, y0 + 198, "花、轮、星无头", (180, 190, 200), 18)

    path = os.path.join(d.OUT, "heads-tails.png")
    d.write_png(path, W, H, bytes(canvas))
    print("wrote", path, W, H)


if __name__ == "__main__":
    main()
