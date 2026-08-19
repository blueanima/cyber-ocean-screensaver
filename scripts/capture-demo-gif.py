#!/usr/bin/env python3
"""把海洋屏保打成 README 用的循环 GIF（firefox 无头截图）。"""
from __future__ import annotations

import os
import shutil
import subprocess
import sys
import tempfile
import time
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs" / "screenshots" / "demo.gif"
PORT = int(os.environ.get("PORT", "8766"))
FRAMES = int(os.environ.get("DEMO_FRAMES", "16"))
WARM0 = int(os.environ.get("DEMO_WARM0", "80"))
WARM_STEP = int(os.environ.get("DEMO_WARM_STEP", "8"))
WIDTH = int(os.environ.get("DEMO_WIDTH", "960"))
HEIGHT = int(os.environ.get("DEMO_HEIGHT", "540"))
SEED = int(os.environ.get("DEMO_SEED", "42"))


def wait_health(url: str, tries: int = 40) -> None:
    last = None
    for _ in range(tries):
        try:
            urllib.request.urlopen(url, timeout=1)
            return
        except Exception as exc:
            last = exc
            time.sleep(0.25)
    raise SystemExit(f"server not ready: {last}")


def main() -> int:
    try:
        from PIL import Image
    except ImportError:
        subprocess.check_call([sys.executable, "-m", "pip", "install", "--user", "pillow"])
        from PIL import Image

    firefox = os.environ.get("FIREFOX", "firefox")
    server = subprocess.Popen(
        [sys.executable, str(ROOT / "main.py"), "--no-browser", "--port", str(PORT)],
        cwd=ROOT,
    )
    profile = Path(tempfile.mkdtemp(prefix="cyber-ocean-ff."))
    work = Path(tempfile.mkdtemp(prefix="cyber-ocean-gif."))
    try:
        wait_health(f"http://127.0.0.1:{PORT}/api/health")
        pngs: list[Path] = []
        for i in range(FRAMES):
            warm = WARM0 + i * WARM_STEP
            png = work / f"f{i:02d}.png"
            url = (
                f"http://127.0.0.1:{PORT}/screensaver"
                f"?wallpaper=1&seed={SEED}&shot=1&warm={warm}"
            )
            subprocess.run(
                [
                    firefox,
                    "--headless",
                    "--profile",
                    str(profile),
                    f"--window-size={WIDTH},{HEIGHT}",
                    "--screenshot",
                    str(png),
                    url,
                ],
                check=True,
                timeout=60,
            )
            if not png.is_file():
                raise SystemExit(f"firefox did not write {png}")
            pngs.append(png)
            print(f"frame {i + 1}/{FRAMES} warm={warm}", flush=True)

        images = []
        for path in pngs:
            im = Image.open(path).convert("P", palette=Image.Palette.ADAPTIVE, colors=64)
            images.append(im)
        OUT.parent.mkdir(parents=True, exist_ok=True)
        images[0].save(
            OUT,
            save_all=True,
            append_images=images[1:],
            duration=90,
            loop=0,
            optimize=True,
        )
        print(f"wrote {OUT} ({OUT.stat().st_size} bytes)")
        return 0
    finally:
        server.terminate()
        try:
            server.wait(timeout=5)
        except subprocess.TimeoutExpired:
            server.kill()
        shutil.rmtree(profile, ignore_errors=True)
        shutil.rmtree(work, ignore_errors=True)


if __name__ == "__main__":
    raise SystemExit(main())
