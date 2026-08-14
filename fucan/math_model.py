"""北斗浮蚕公式（与海报一致）+ 粒子采样。"""

from __future__ import annotations

import math
from dataclasses import dataclass
from typing import List, Optional, Tuple

Point = Tuple[float, float]


@dataclass
class BeidouParams:
    t: float = 0.0
    n_points: int = 40000


# 固定视野：对准粒子最密的躯干段（避免每帧乱跳）
VIEW_X = (-90.0, 90.0)
VIEW_Y = (50.0, 500.0)


def point_at(x: float, y: float, t: float) -> Optional[Point]:
    k = x / 4.0 - 12.5
    e = y / 9.0 + 6.0
    o = math.hypot(k, e) / 9.0
    if abs(k) < 1e-9 or o < 1e-9:
        return None
    if abs(math.cos(y / 2.0)) < 0.015:
        return None
    try:
        half_tan = 0.5 * math.tan(y / 2.0)
        if not math.isfinite(half_tan) or abs(half_tan) > 60:
            return None
        c = o / 2.0 + e / 2.0 - t / 4.0
        q = (3.0 / k) * (half_tan + math.cos(y)) + k * (
            5.0 / o + o * math.sin(y) * math.sin(e + 4.0 * o - t)
        )
        px = q + 40.0 * math.cos(c)
        py = q * math.sin(c) - (o * k * k) / 6.0 + 12.0 * e * o
    except (ValueError, OverflowError, ZeroDivisionError):
        return None
    if not (math.isfinite(px) and math.isfinite(py)):
        return None
    return px, py


def sample_points(t: float, n_points: int = 40000) -> List[Point]:
    """与 Matlab scatter 一致：i=0:N-1, x=mod(i,100), y=floor(i/100)。"""
    pts: List[Point] = []
    for i in range(n_points):
        x = float(i % 100)
        y = float(i // 100)
        p = point_at(x, y, t)
        if p is not None:
            pts.append(p)
    return pts


FORMULA_PLAIN = [
    "c = o/2 + e/2 − t/4 ,  k = x/4 − 12.5 ,  e = y/9 + 6 ,  o = √(k²+e²)/9",
    "q = (3/k)(½ tan(y/2) + cos y) + k(5/o + o·sin y·sin(e+4o−t))",
    "⟨ q + 40 cos(c) ,  q sin c − o k²/6 + 12 e o ⟩",
]
