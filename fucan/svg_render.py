"""导出 SVG：用大量半透明圆点模拟 Matlab scatter。"""

from __future__ import annotations

from .math_model import FORMULA_PLAIN, VIEW_X, VIEW_Y, sample_points


def rows_to_svg(
    t: float = 0.0,
    width: int = 720,
    height: int = 1280,
    title: str = "北斗浮蚕 Matlab",
    n_points: int = 28000,
) -> str:
    pts = sample_points(t, n_points=n_points)
    minx, maxx = VIEW_X
    miny, maxy = VIEW_Y
    dx = maxx - minx
    dy = maxy - miny
    top, bottom, side = 0.10, 0.18, 0.08

    def mx(x: float) -> float:
        return (side + (x - minx) / dx * (1 - 2 * side)) * width

    def my(y: float) -> float:
        return (top + (maxy - y) / dy * (1 - top - bottom)) * height

    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" '
        f'viewBox="0 0 {width} {height}">',
        f'<rect width="100%" height="100%" fill="#050505"/>',
        f'<text x="{width/2}" y="{height*0.055}" text-anchor="middle" '
        f'fill="#ffffff" font-size="{int(width*0.055)}" font-family="sans-serif">'
        f"{title}</text>",
        '<g fill="#ffffff" fill-opacity="0.22">',
    ]

    r = max(0.7, width / 900)
    # 抽样绘制，避免 SVG 过大
    step = max(1, len(pts) // 12000)
    for i in range(0, len(pts), step):
        x, y = pts[i]
        if x < minx or x > maxx or y < miny or y > maxy:
            continue
        parts.append(f'<circle cx="{mx(x):.1f}" cy="{my(y):.1f}" r="{r:.2f}"/>')

    parts.append("</g>")
    fy = height * 0.86
    for line in FORMULA_PLAIN:
        parts.append(
            f'<text x="{width/2}" y="{fy:.1f}" text-anchor="middle" fill="#f2fff2" '
            f'font-size="{max(11, int(width*0.022))}" font-family="Georgia, serif">{line}</text>'
        )
        fy += height * 0.035
    parts.append("</svg>")
    return "\n".join(parts)
