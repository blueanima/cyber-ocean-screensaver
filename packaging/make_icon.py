#!/usr/bin/env python3
"""Write a 256×256 PNG icon (stdlib only)."""
from __future__ import annotations

import math
import struct
import zlib
from pathlib import Path


def _chunk(tag: bytes, data: bytes) -> bytes:
    crc = zlib.crc32(tag + data) & 0xFFFFFFFF
    return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", crc)


def write_icon(path: Path, size: int = 256) -> None:
    rows = []
    cx = cy = (size - 1) / 2
    r_out = size * 0.46
    for y in range(size):
        raw = bytearray(1 + size * 4)
        raw[0] = 0
        for x in range(size):
            dx, dy = x - cx, y - cy
            d = math.hypot(dx, dy)
            t = max(0.0, 1.0 - d / r_out)
            glow = math.exp(-((d - r_out * 0.35) ** 2) / (size * 1.8))
            dots = 0.0
            for i in range(9):
                ang = i * (math.pi * 2 / 9) + 0.4
                px = cx + math.cos(ang) * r_out * 0.28
                py = cy + math.sin(ang) * r_out * 0.22
                dots += math.exp(-((x - px) ** 2 + (y - py) ** 2) / 28)
            a = int(min(255, 255 * (t ** 0.45)))
            g = int(min(255, 40 + 180 * t + 70 * glow + 90 * dots))
            b = int(min(255, 50 + 160 * t + 80 * glow))
            r = int(min(255, 8 + 30 * t + 40 * dots))
            o = 1 + x * 4
            raw[o : o + 4] = bytes((r, g, b, a))
        rows.append(bytes(raw))
    compressed = zlib.compress(b"".join(rows), 9)
    png = b"\x89PNG\r\n\x1a\n"
    png += _chunk(b"IHDR", struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0))
    png += _chunk(b"IDAT", compressed)
    png += _chunk(b"IEND", b"")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(png)


if __name__ == "__main__":
    out = Path(__file__).with_name("cyber-ocean.png")
    write_icon(out)
    print(out)
