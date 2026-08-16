#!/usr/bin/env python3
"""Dump heading-aligned silhouettes for polarity audit."""
import math
import os
import struct
import zlib

_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(_ROOT, ".cache", "heading-audit")
os.makedirs(OUT, exist_ok=True)
FACES = [
    -1.583, 2.221, 1.540, 0.125, 1.563, 2.610, 1.977, 1.962,
    0.000, 0.000, 2.287, 1.305, 1.783, 0.000, 2.390, 1.885, 1.666,
]
IDS = [
    "fucan", "youyan", "jichong", "jelly", "nebula", "lantern", "feather",
    "tentacle", "flower6", "wheel", "spiral", "comb", "saweel", "star8",
    "shrimp", "vortex", "angel",
]


def push(out, x, y, u=0.0, v=0.0):
    if math.isfinite(x) and math.isfinite(y):
        out.append((x, y, u, v))


def fill_fucan(t, step=4):
    out = []
    i = 0
    while i < 22000:
        x = float(i % 100)
        y = float(i // 100)
        k = x / 4.0 - 12.5
        e = y / 9.0 + 6.0
        o = math.sqrt(k * k + e * e) / 9.0
        if abs(k) < 1e-9 or o < 1e-9 or abs(math.cos(y / 2.0)) < 0.015:
            i += step
            continue
        ht = 0.5 * math.tan(y / 2.0)
        if not math.isfinite(ht) or abs(ht) > 60.0:
            i += step
            continue
        c = o / 2.0 + e / 2.0 - t / 4.0
        q = (3.0 / k) * (ht + math.cos(y)) + k * (5.0 / o + o * math.sin(y) * math.sin(e + 4.0 * o - t))
        xw = q + 40.0 * math.cos(c)
        yw = q * math.sin(c) - (o * k * k) / 6.0 + 12.0 * e * o
        if math.isfinite(xw) and math.isfinite(yw):
            out.append((200.0 + xw * 0.82, 28.0 + (yw - 50.0) * 0.82, y, k))
        i += step
    return out


def fill_youyan(t, step=4):
    out = []
    i = 0
    while i < 18000:
        x = float(i % 100)
        y = float(i // 100)
        k = x / 4.0 - 12.5
        e = y / 9.0 + 5.0
        o = math.sqrt(k * k + e * e) / 9.0
        if abs(k) < 1e-6:
            i += step
            continue
        q = x + 99.0 + math.tan(1.0 / k) + o * k * (math.cos(e * 9.0) / 4.0 + math.cos(y / 2.0)) * math.sin(o * 4.0 - t)
        c = o * e / 30.0 - t / 8.0
        push(out, q * 0.7 * math.sin(c) + 9.0 * math.cos(y / 19.0 + t) + 200.0, 200.0 + q / 2.0 * math.cos(c), y, k)
        i += step
    return out


def fill_jichong(t, step=3):
    out = []
    i = 0
    while i < 9000:
        x = float(i)
        y = i / 235.0
        e = y / 8.0 - 13.0
        k = (4.0 + math.sin(y * 2.0 - t) * 3.0) * math.cos(x / 29.0)
        if abs(k) < 1e-6:
            i += step
            continue
        d = math.sqrt(k * k + e * e)
        q = 3.0 * math.sin(k * 2.0) + 0.3 / k + math.sin(y / 25.0) * k * (9.0 + 4.0 * math.sin(e * 9.0 - d * 3.0 + t * 2.0))
        push(out, q + 30.0 * math.cos(d - t) + 200.0, 620.0 - q * math.sin(d - t) - d * 39.0, y, k)
        i += step
    return out


def fill_jelly(t, step=4):
    out = []
    i = 0
    while i < 10000:
        x = float(i % 200)
        y = i / 43.0
        k = 5.0 * math.cos(x / 14.0) * math.cos(y / 30.0)
        e = y / 8.0 - 13.0
        d = (k * k + e * e) / 59.0 + 4.0
        a = math.atan2(k, e)
        q = 60.0 - 3.0 * math.sin(a * e) + k * (3.0 + 4.0 / d * math.sin(d * d - t * 2.0))
        c = d / 2.0 + e / 99.0 - t / 18.0
        push(out, q * math.sin(c) + 200.0, (q + d * 9.0) * math.cos(c) + 200.0, e, k)
        i += step
    return out


def fill_nebula(t, step=5):
    out = []
    i = 0
    while i < 20000:
        x = float(i % 200)
        y = i / 200.0
        k = x / 8.0 - 12.5
        e = y / 8.0 - 12.5
        o = (k * k + e * e) / 169.0
        d = 0.5 + 5.0 * math.cos(o)
        push(out, x + d * k * math.sin(d * 2.0 + o + t) + e * math.cos(e + t) + 100.0,
             y / 4.0 - o * 135.0 + d * 6.0 * math.cos(d * 3.0 + o * 9.0 + t) + 275.0, e, k)
        i += step
    return out


def fill_lantern(t, step=4):
    out = []
    i = 0
    while i < 10000:
        x = float(i % 200)
        y = i / 55.0
        k = 9.0 * math.cos(x / 8.0)
        e = y / 8.0 - 12.5
        d = (k * k + e * e) / 99.0 + math.sin(t) / 6.0 + 0.5
        if abs(d) < 1e-6:
            i += step
            continue
        q = 99.0 - e * math.sin(math.atan2(k, e) * 7.0) / d + k * (3.0 + math.cos(d * d - t) * 2.0)
        c = d / 2.0 + e / 69.0 - t / 16.0
        push(out, q * math.sin(c) + 200.0, (q + 19.0 * d) * math.cos(c) + 200.0, e, k)
        i += step
    return out


def fill_feather(t, step=3):
    out = []
    i = 1
    while i <= 9000:
        y = i / 790.0
        k = 6.0 + math.sin(float((int(math.floor(y)) ^ 1))) * 6.0 if y < 5.0 else 4.0 + math.cos(y)
        cs = math.cos(i + t / 4.0)
        d = math.sqrt((k * cs) ** 2 + (y / 3.0 - 13.0) ** 2)
        q = y * k * cs / 5.0 * (2.0 + math.sin(d * 2.0 + y - t * 4.0))
        c = d / 3.0 - t / 2.0 + (i % 2)
        push(out, q + 90.0 * math.cos(c) + 200.0, 400.0 - (q * math.sin(c) + d * 29.0 - 170.0), y, k * cs)
        i += step
    return out


def fill_tentacle(t, step=3):
    out = []
    i = 1
    while i <= 9000:
        y = i / 345.0
        x = y
        if y < 11.0:
            x = 6.0 + math.sin(float((int(math.floor(x)) ^ 8))) * 6.0
        else:
            x = x / 5.0 + math.cos(x / 2.0)
        e = y / 7.0 - 13.0
        k = x * math.cos(i - t / 4.0)
        d = math.sqrt(k * k + e * e) + math.sin(e / 4.0 + t) / 2.0
        if abs(d) < 1e-6:
            i += step
            continue
        q = y * k / d * (3.0 + math.sin(d * 2.0 + y / 2.0 - t * 4.0))
        c = d / 2.0 + 1.0 - t / 2.0
        push(out, q + 60.0 * math.cos(c) + 200.0, 400.0 - (q * math.sin(c) + d * 29.0 - 170.0), y, k)
        i += step
    return out


def fill_spiral(t, step=4):
    out = []
    i = 0
    while i < 12000:
        x = float(i % 120)
        y = float(i // 120)
        k = x / 5.0 - 12.0
        e = y / 8.0 - 8.0
        o = math.sqrt(k * k + e * e) / 8.0
        c = o * 1.15 + t / 5.0
        q = 22.0 + 10.0 * math.sin(e * 0.8 + t) + k * (1.6 + 0.35 * math.sin(3.0 * o - t))
        push(out, q * math.cos(c) + 10.0 * math.sin(e * 2.0 + t) + 200.0, q * math.sin(c) * 0.88 + 200.0, e, k)
        i += step
    return out


def fill_comb(t, step=4):
    out = []
    i = 0
    while i < 10000:
        x = float(i % 180)
        y = i / 50.0
        k = 7.0 * math.cos(x / 10.0) * math.cos(y / 35.0)
        e = y / 8.0 - 12.0
        d = (k * k + e * e) / 70.0 + 3.0
        if abs(d) < 1e-6:
            i += step
            continue
        a = math.atan2(k, e)
        q = 48.0 - 4.0 * math.sin(a * 4.0) + k * (2.2 + 3.0 / d * math.sin(d * d - t))
        c = d / 2.4 + e / 85.0 - t / 14.0
        push(out, q * math.sin(c) + 200.0, (q + 7.0 * d) * math.cos(c) * 0.78 + 12.0 * math.sin(x / 18.0 + t) + 210.0, e, k)
        i += step
    return out


def fill_saweel(t, step=3):
    out = []
    i = 0
    while i < 9000:
        x = float(i)
        y = i / 210.0
        e = y / 9.0 - 12.0
        k = (3.5 + math.sin(y * 1.6 - t) * 2.4) * math.cos(x / 22.0)
        if abs(k) < 1e-6:
            i += step
            continue
        d = math.sqrt(k * k + e * e)
        q = 2.2 * math.sin(k * 3.0) + 0.25 / k + math.sin(y / 18.0) * k * (7.0 + 3.0 * math.sin(e * 6.0 - d * 2.0 + t * 2.0))
        push(out, q + 24.0 * math.cos(d * 0.7 - t) + 200.0, 560.0 - q * math.sin(d * 0.7 - t) - d * 32.0, y, k)
        i += step
    return out


def fill_shrimp(t, step=4):
    out = []
    i = 0
    while i < 14000:
        x = float(i % 100)
        y = float(i // 100)
        k = x / 4.0 - 12.5
        e = y / 8.0 + 3.5
        o = math.sqrt(k * k + e * e) / 8.0
        if abs(k) < 1e-6 or o < 1e-6:
            i += step
            continue
        q = 55.0 + 10.0 * math.sin(k * 0.8) + k * (2.2 + 0.55 * o * math.sin(y * 0.7 - t))
        c = o / 3.2 + e / 22.0 - t / 9.0
        push(out, q * 0.5 * math.sin(c) + 7.0 * math.cos(y / 16.0 + t) + 200.0,
             200.0 + q * 0.38 * math.cos(c) + 6.0 * math.sin(k + t * 0.6), y, k)
        i += step
    return out


def fill_vortex(t, step=4):
    out = []
    i = 0
    while i < 14000:
        x = float(i % 200) - 100.0
        y = (i / 200.0) - 35.0
        r = math.sqrt(x * x + y * y) / 38.0
        th = math.atan2(y, x)
        rbig = 62.0 + 22.0 * math.sin(3.0 * th + t) + 10.0 * math.sin(r * 5.0 - t * 2.0)
        push(out, rbig * math.cos(th + r * 0.45 + t / 7.0) + 200.0,
             rbig * math.sin(th + r * 0.45 + t / 7.0) * 0.9 + 200.0, y, x)
        i += step
    return out


def fill_angel(t, step=4):
    out = []
    i = 0
    while i < 10000:
        x = float(i % 160)
        y = i / 65.0
        k = 6.5 * math.cos(x / 12.0) * math.cos(y / 38.0)
        e = y / 9.0 - 11.0
        d = (k * k + e * e) / 68.0 + 2.4
        if abs(d) < 1e-6:
            i += step
            continue
        a = math.atan2(k, e)
        q = 38.0 - 5.0 * math.sin(a * 3.0) + k * (2.0 + 3.2 / d * math.sin(d * 2.2 - t))
        c = d / 2.1 + e / 88.0 - t / 15.0
        push(out, q * math.sin(c) + 200.0, (q + 6.5 * d) * math.cos(c) + 8.0 * math.sin(e * 0.5 + t) + 205.0, e, k)
        i += step
    return out


FILLS = {
    "fucan": fill_fucan, "youyan": fill_youyan, "jichong": fill_jichong,
    "jelly": fill_jelly, "nebula": fill_nebula, "lantern": fill_lantern,
    "feather": fill_feather, "tentacle": fill_tentacle, "spiral": fill_spiral,
    "comb": fill_comb, "saweel": fill_saweel, "shrimp": fill_shrimp,
    "vortex": fill_vortex, "angel": fill_angel,
}


def write_png(path, w, h, rgb):
    def chunk(tag, data):
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)

    raw = b"".join(b"\x00" + rgb[y * w * 3 : (y + 1) * w * 3] for y in range(h))
    with open(path, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n")
        f.write(chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0)))
        f.write(chunk(b"IDAT", zlib.compress(raw, 6)))
        f.write(chunk(b"IEND", b""))


def xform(pts, face):
    cx = sum(p[0] for p in pts) / len(pts)
    cy = sum(p[1] for p in pts) / len(pts)
    ca, sa = math.cos(-face), math.sin(-face)
    out = []
    for p in pts:
        xy = ((p[0] - cx) * ca - (p[1] - cy) * sa, (p[0] - cx) * sa + (p[1] - cy) * ca)
        out.append(xy + p[2:] if len(p) > 2 else xy)
    return out


def draw_panel(xy, w, h, pad=18):
    xs = [p[0] for p in xy]
    ys = [p[1] for p in xy]
    xs.sort()
    ys.sort()
    n = len(xs)
    minx, maxx = xs[n // 40], xs[n * 39 // 40]
    miny, maxy = ys[n // 40], ys[n * 39 // 40]
    span = max(maxx - minx, maxy - miny, 8.0)
    cx = (minx + maxx) * 0.5
    cy = (miny + maxy) * 0.5
    s = (min(w, h) - 2 * pad) / span
    rgb = bytearray(w * h * 3)
    for i in range(0, w * h * 3, 3):
        rgb[i] = 8
        rgb[i + 1] = 18
        rgb[i + 2] = 36

    def put(ix, iy, r, g, b, a=1.0):
        if 0 <= ix < w and 0 <= iy < h:
            o = (iy * w + ix) * 3
            rgb[o] = int(rgb[o] * (1 - a) + r * a)
            rgb[o + 1] = int(rgb[o + 1] * (1 - a) + g * a)
            rgb[o + 2] = int(rgb[o + 2] * (1 - a) + b * a)

    for x, y in xy:
        ix = int((x - cx) * s + w * 0.5)
        iy = int((y - cy) * s + h * 0.5)
        put(ix, iy, 230, 240, 255, 0.55)
        put(ix + 1, iy, 180, 210, 255, 0.25)
        put(ix, iy + 1, 180, 210, 255, 0.25)

    # heading arrow: +X
    y0 = h // 2
    for x in range(w - 70, w - 18):
        put(x, y0, 255, 80, 70, 0.95)
        put(x, y0 - 1, 255, 80, 70, 0.6)
    for k in range(-7, 8):
        put(w - 18 - abs(k), y0 + k, 255, 80, 70, 0.95)
    return rgb


def main():
    t = 1.2
    pw, ph = 360, 220
    for i, sid in enumerate(IDS):
        if sid not in FILLS:
            continue
        pts = FILLS[sid](t)
        face = FACES[i]
        cur = xform(pts, face)
        flip = xform(pts, face + math.pi)
        a = draw_panel(cur, pw, ph)
        b = draw_panel(flip, pw, ph)
        w, h = pw * 2, ph
        rgb = bytearray(w * h * 3)
        for y in range(ph):
            rgb[y * w * 3 : y * w * 3 + pw * 3] = a[y * pw * 3 : (y + 1) * pw * 3]
            rgb[y * w * 3 + pw * 3 : (y + 1) * w * 3] = b[y * pw * 3 : (y + 1) * pw * 3]
        # divider
        for y in range(ph):
            o = (y * w + pw) * 3
            rgb[o : o + 3] = bytes([40, 80, 120])
        path = os.path.join(OUT, f"{i:02d}-{sid}.png")
        write_png(path, w, h, bytes(rgb))
        print("wrote", path, "n", len(pts))


if __name__ == "__main__":
    main()
