#!/usr/bin/env python3
"""Rasterize gait CSV traces to PNG (trajectory + heading ticks)."""
from __future__ import annotations

import csv
import math
import struct
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / ".cache" / "gait-obs"
OUT = SRC


def write_png(path: Path, w: int, h: int, rgb: bytearray) -> None:
    def chunk(tag: bytes, data: bytes) -> bytes:
        crc = zlib.crc32(tag + data) & 0xFFFFFFFF
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", crc)

    raw = bytearray()
    for y in range(h):
        raw.append(0)
        i = y * w * 3
        raw.extend(rgb[i : i + w * 3])
    path.write_bytes(
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(bytes(raw), 9))
        + chunk(b"IEND", b"")
    )


def put(rgb: bytearray, w: int, h: int, x: int, y: int, c: tuple[int, int, int]) -> None:
    if 0 <= x < w and 0 <= y < h:
        i = (y * w + x) * 3
        rgb[i : i + 3] = bytes(c)


def line(rgb, w, h, x0, y0, x1, y1, c):
    n = max(abs(int(x1) - int(x0)), abs(int(y1) - int(y0)), 1)
    for i in range(n + 1):
        t = i / n
        put(rgb, w, h, int(x0 + (x1 - x0) * t), int(y0 + (y1 - y0) * t), c)


def plot_one(csv_path: Path, png_path: Path, w=420, h=420) -> None:
    pts = []
    with csv_path.open() as f:
        for row in csv.DictReader(f):
            pts.append((float(row["x"]), float(row["y"]), float(row["rot"])))
    if len(pts) < 4:
        return
    xs = [p[0] for p in pts]
    ys = [p[1] for p in pts]
    xmin, xmax = min(xs), max(xs)
    ymin, ymax = min(ys), max(ys)
    span = max(xmax - xmin, ymax - ymin, 0.02)
    pad = span * 0.18
    xmin -= pad
    ymin -= pad
    span += pad * 2
    rgb = bytearray([10, 18, 36]) * (w * h)

    def to_px(x, y):
        px = 20 + (x - xmin) / span * (w - 40)
        py = h - 20 - (y - ymin) / span * (h - 40)
        return px, py

    for i in range(1, len(pts)):
        x0, y0 = to_px(pts[i - 1][0], pts[i - 1][1])
        x1, y1 = to_px(pts[i][0], pts[i][1])
        t = i / len(pts)
        c = (int(80 + 140 * t), int(180 + 40 * t), 255)
        line(rgb, w, h, x0, y0, x1, y1, c)
    step = max(len(pts) // 14, 1)
    for i in range(0, len(pts), step):
        x, y, rot = pts[i]
        px, py = to_px(x, y)
        fx, fy = math.cos(rot), -math.sin(rot)
        line(rgb, w, h, px, py, px + fx * 16, py + fy * 16, (255, 210, 90))
    sx, sy = to_px(pts[0][0], pts[0][1])
    ex, ey = to_px(pts[-1][0], pts[-1][1])
    line(rgb, w, h, sx - 3, sy - 3, sx + 3, sy + 3, (80, 255, 140))
    line(rgb, w, h, sx - 3, sy + 3, sx + 3, sy - 3, (80, 255, 140))
    line(rgb, w, h, ex - 4, ey, ex + 4, ey, (255, 90, 90))
    line(rgb, w, h, ex, ey - 4, ex, ey + 4, (255, 90, 90))
    write_png(png_path, w, h, rgb)


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    for csv_path in sorted(SRC.glob("*.csv")):
        plot_one(csv_path, OUT / f"{csv_path.stem}.png")
        print(csv_path.stem)


if __name__ == "__main__":
    main()
