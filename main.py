#!/usr/bin/env python3
"""赛博海洋馆屏幕保护 / 北斗浮蚕集合馆

用法：
  python3 main.py
  python3 main.py --screensaver
  python3 main.py --wallpaper
  python3 main.py --config
  python3 main.py --export beidou_fucan.svg
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from fucan.gallery import INDEX_HTML
from fucan.kiosk import show_config_dialog
from fucan.server import run_server
from fucan.settings import native_argv_from_ns
from fucan.svg_render import rows_to_svg


def _windows_scr_mode(argv: list[str]) -> str | None:
    """解析 Windows 屏保参数：/s 运行、/c 设置、/p 预览。"""
    if len(argv) < 2:
        return None
    raw = " ".join(argv[1:]).strip().lower().replace("-", "/")
    if raw.startswith("/c") or raw == "c":
        return "config"
    if raw.startswith("/p") or raw == "p":
        return "preview"
    if "/s" in raw.split() or raw == "/s" or raw.startswith("/s"):
        return "run"
    return None


def main(argv: list[str] | None = None) -> None:
    argv = list(sys.argv if argv is None else argv)
    frozen = getattr(sys, "frozen", False)
    scr = None
    if frozen and len(argv) == 1:
        scr = "run"
    elif frozen or sys.platform == "win32":
        scr = _windows_scr_mode(argv)
    if scr == "config":
        show_config_dialog()
        return
    if scr == "preview":
        return
    if scr == "run":
        from fucan.kiosk import launch_native_gpu

        launch_native_gpu(["--screensaver"])
        run_server(
            host="127.0.0.1",
            port=8765,
            open_browser=True,
            path="/screensaver",
            allow_quit=True,
            kiosk=True,
        )
        return

    parser = argparse.ArgumentParser(description="赛博海洋馆屏幕保护 · 数字生命集合馆")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8765)
    parser.add_argument("--no-browser", action="store_true")
    parser.add_argument("--screensaver", action="store_true", help="全屏屏保：键鼠退出")
    parser.add_argument("--wallpaper", action="store_true", help="全屏壁纸模式：不因键鼠退出")
    parser.add_argument("--config", action="store_true", help="打开画质 / 密度设置")
    parser.add_argument("--no-setup", action="store_true", help="跳过启动前的设置窗口（空闲屏保用）")
    parser.add_argument(
        "--quality",
        choices=("low", "medium", "high", "ultra"),
        help="画质预设：low / medium / high / ultra",
    )
    parser.add_argument("--step", type=int, help="粒子密度，1 最密，越大越疏")
    parser.add_argument("--fps", type=float, help="帧率上限")
    parser.add_argument("--count", type=int, help="生物数量 1–17")
    parser.add_argument("--no-legend", dest="legend", action="store_false", default=None, help="关闭左上图例")
    parser.add_argument("--no-formula", dest="formula", action="store_false", default=None, help="关闭生物公式")
    parser.add_argument("--legend-stride", type=int, help="图例抽样，1 为全点")
    parser.add_argument("--point-size", type=float, help="点大小倍率，默认 1.0")
    parser.add_argument("--no-vsync", dest="vsync", action="store_false", default=None, help="关闭垂直同步")
    parser.add_argument("--seed", type=int, help="随机种子")
    parser.add_argument("--lang", choices=("zh", "en"), help="界面语言：zh 中文 / en English")
    parser.add_argument("--export", type=Path, help="导出 SVG 后退出")
    parser.add_argument("--t", type=float, default=1.2, help="导出时的时间参数 t")
    parser.add_argument(
        "--write-html",
        type=Path,
        help="写出可双击打开的 HTML（不依赖服务器）",
    )
    parser.add_argument(
        "--write-screensaver",
        type=Path,
        help="写出屏保用 HTML（可供 Lively 等加载）",
    )
    args = parser.parse_args(argv[1:])

    if args.config:
        show_config_dialog()
        return

    native_flags = native_argv_from_ns(args)

    if args.write_html:
        args.write_html.write_text(INDEX_HTML, encoding="utf-8")
        print(f"已写出：{args.write_html.resolve()}")
        print("可用浏览器直接打开该 HTML 文件。")
        return

    if args.write_screensaver:
        html = INDEX_HTML.replace("<body>", '<body data-mode="saver">', 1)
        if args.wallpaper:
            html = INDEX_HTML.replace("<body>", '<body data-mode="wallpaper">', 1)
        args.write_screensaver.write_text(html, encoding="utf-8")
        print(f"已写出：{args.write_screensaver.resolve()}")
        return

    if args.export:
        args.export.write_text(rows_to_svg(t=args.t), encoding="utf-8")
        print(f"已保存：{args.export.resolve()}")
        return

    path = "/"
    allow_quit = False
    kiosk = False
    if args.screensaver or args.wallpaper:
        if not args.no_setup:
            status = show_config_dialog(startable=True)
            if status != "start":
                return
        from fucan.kiosk import launch_native_gpu

        mode = "--wallpaper" if args.wallpaper else "--screensaver"
        launch_native_gpu([mode, *native_flags])
        path = "/screensaver?wallpaper=1" if args.wallpaper else "/screensaver"
        allow_quit = not args.wallpaper
        kiosk = True

    run_server(
        host=args.host,
        port=args.port,
        open_browser=not args.no_browser,
        path=path,
        allow_quit=allow_quit,
        kiosk=kiosk,
    )


if __name__ == "__main__":
    main()
