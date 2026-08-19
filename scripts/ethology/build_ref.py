#!/usr/bin/env python3
"""Build ethology reference stats from downloaded fish-school CSVs.

    python3 scripts/ethology/build_ref.py
"""
from __future__ import annotations

import csv
import json
import math
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DIR = ROOT / "data" / "ethology"
OUT = DIR / "ref.json"

# Puy et al. 2024 Zenodo 10890112
FPS = 50.0
PX_PER_CM = 2745.0 / 100.0
BL_CM = 3.5
BL_PX = BL_CM * PX_PER_CM
STRIDE = 10  # 5 Hz
MIN_SPEED_BL = 1.0


def wrap_pi(a: float) -> float:
    while a > math.pi:
        a -= 2 * math.pi
    while a < -math.pi:
        a += 2 * math.pi
    return a


def pct(xs: list[float], p: float) -> float:
    if not xs:
        return 0.0
    ys = sorted(xs)
    i = min(len(ys) - 1, max(0, int(round((p / 100.0) * (len(ys) - 1)))))
    return ys[i]


def load_xy(path: Path) -> list[list[tuple[float, float]]]:
    with path.open() as f:
        rows = csv.reader(f)
        header = next(rows)
        n = (len(header) - 1) // 2
        frames: list[list[tuple[float, float]]] = []
        for k, row in enumerate(rows):
            if k % STRIDE != 0:
                continue
            pts = []
            ok = True
            for i in range(n):
                try:
                    x = float(row[1 + 2 * i])
                    y = float(row[2 + 2 * i])
                except (ValueError, IndexError):
                    ok = False
                    break
                if not math.isfinite(x) or not math.isfinite(y):
                    ok = False
                    break
                pts.append((x, y))
            if ok:
                frames.append(pts)
    return frames


def analyze(frames: list[list[tuple[float, float]]]) -> dict:
    dt = STRIDE / FPS
    nnds: list[float] = []
    polars: list[float] = []
    yaws: list[float] = []
    speeds: list[float] = []
    sharp = 0
    segs = 0
    prev_h: list[float] | None = None
    for t, pts in enumerate(frames):
        n = len(pts)
        heads = [0.0] * n
        units: list[tuple[float, float]] = []
        if t == 0:
            prev_h = [0.0] * n
            continue
        prev = frames[t - 1]
        sx = sy = 0.0
        moving = 0
        for i in range(n):
            dx = pts[i][0] - prev[i][0]
            dy = pts[i][1] - prev[i][1]
            vpx = math.hypot(dx, dy) / dt
            vbl = vpx / BL_PX
            if vbl < MIN_SPEED_BL:
                heads[i] = prev_h[i] if prev_h else 0.0
                continue
            moving += 1
            speeds.append(vbl)
            h = math.atan2(dy, dx)
            heads[i] = h
            units.append((math.cos(h), math.sin(h)))
            if prev_h is not None:
                yaw = abs(wrap_pi(h - prev_h[i])) / dt
                yaws.append(yaw)
                segs += 1
                if yaw * dt > math.radians(75):
                    sharp += 1
            nn = min(
                math.hypot(pts[i][0] - pts[j][0], pts[i][1] - pts[j][1]) / BL_PX
                for j in range(n)
                if j != i
            )
            nnds.append(nn)
        if units:
            mx = sum(u[0] for u in units) / len(units)
            my = sum(u[1] for u in units) / len(units)
            polars.append(math.hypot(mx, my))
        prev_h = heads
    return {
        "frames": len(frames),
        "nnd_bl": {
            "mean": sum(nnds) / max(len(nnds), 1),
            "p10": pct(nnds, 10),
            "p50": pct(nnds, 50),
            "p90": pct(nnds, 90),
        },
        "polar": {
            "mean": sum(polars) / max(len(polars), 1),
            "p10": pct(polars, 10),
            "p50": pct(polars, 50),
        },
        "yaw_rad_s": {
            "mean": sum(yaws) / max(len(yaws), 1),
            "p50": pct(yaws, 50),
            "p90": pct(yaws, 90),
        },
        "speed_bl_s": {
            "mean": sum(speeds) / max(len(speeds), 1),
            "p50": pct(speeds, 50),
        },
        "sharp_turn_frac": sharp / max(segs, 1),
    }


def main() -> None:
    recs = []
    for p in sorted(DIR.glob("Experimental_school_N=8_recording_*.csv")):
        print(f"analyze {p.name}", flush=True)
        recs.append({"file": p.name, **analyze(load_xy(p))})
    if not recs:
        raise SystemExit("no CSVs in data/ethology")
    keys = ("nnd_bl", "polar", "yaw_rad_s", "speed_bl_s")
    pooled = {}
    for k in keys:
        pooled[k] = {
            m: sum(r[k][m] for r in recs) / len(recs) for m in recs[0][k]
        }
    pooled["sharp_turn_frac"] = sum(r["sharp_turn_frac"] for r in recs) / len(recs)
    out = {
        "source": {
            "citation": "Puy et al. 2024 PNAS 10.1073/pnas.2309733121",
            "zenodo": "10.5281/zenodo.10890112",
            "species": "Hemigrammus rhodostomus",
            "n": 8,
            "fps": FPS,
            "px_per_cm": PX_PER_CM,
            "body_length_cm": BL_CM,
            "notes": "NND in body lengths; inactive <1 BL/s dropped; 5 Hz subsample.",
        },
        "recordings": recs,
        "pooled": pooled,
        "targets": {
            "nnd_bl": pooled["nnd_bl"]["p50"],
            "nnd_bl_lo": 1.0,
            "nnd_bl_hi": 4.0,
            "polar": pooled["polar"]["p50"],
            "yaw_rad_s": pooled["yaw_rad_s"]["p50"],
            "sharp_turn_frac": pooled["sharp_turn_frac"],
        },
    }
    OUT.write_text(json.dumps(out, indent=2) + "\n")
    t = out["targets"]
    print(
        f"wrote {OUT}  nnd_p50={t['nnd_bl']:.2f} BL  polar={t['polar']:.2f}  "
        f"yaw={t['yaw_rad_s']:.2f} rad/s  sharp={t['sharp_turn_frac']:.4f}"
    )


if __name__ == "__main__":
    main()
