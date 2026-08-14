"""跨平台全屏浏览器启动：Chrome / Edge / Chromium / Firefox kiosk。"""

from __future__ import annotations

import os
import shutil
import subprocess
import sys
import tempfile
import threading
from pathlib import Path

_browser_proc: subprocess.Popen[bytes] | None = None
_profile_dir: Path | None = None


def _win_paths() -> list[tuple[str, list[str]]]:
    local = os.environ.get("LOCALAPPDATA", "")
    pf = os.environ.get("PROGRAMFILES", r"C:\Program Files")
    pf86 = os.environ.get("PROGRAMFILES(X86)", r"C:\Program Files (x86)")
    chrome_flags = ["--kiosk", "--disable-infobars", "--no-first-run", "--disable-session-crashed-bubble"]
    edge_flags = ["--kiosk", "--edge-kiosk-type=fullscreen", "--no-first-run"]
    return [
        (str(Path(pf) / "Google" / "Chrome" / "Application" / "chrome.exe"), chrome_flags),
        (str(Path(local) / "Google" / "Chrome" / "Application" / "chrome.exe"), chrome_flags),
        (str(Path(pf) / "Microsoft" / "Edge" / "Application" / "msedge.exe"), edge_flags),
        (str(Path(pf86) / "Microsoft" / "Edge" / "Application" / "msedge.exe"), edge_flags),
        (str(Path(pf) / "Chromium" / "Application" / "chrome.exe"), chrome_flags),
        (str(Path(pf) / "Mozilla Firefox" / "firefox.exe"), ["--kiosk"]),
    ]


def _mac_paths() -> list[tuple[str, list[str]]]:
    chrome_flags = ["--kiosk", "--disable-infobars", "--no-first-run", "--disable-session-crashed-bubble"]
    edge_flags = ["--kiosk", "--edge-kiosk-type=fullscreen", "--no-first-run"]
    return [
        ("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome", chrome_flags),
        ("/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge", edge_flags),
        ("/Applications/Chromium.app/Contents/MacOS/Chromium", chrome_flags),
        ("/Applications/Brave Browser.app/Contents/MacOS/Brave Browser", chrome_flags),
        ("/Applications/Firefox.app/Contents/MacOS/firefox", ["--kiosk"]),
    ]


def _linux_paths() -> list[tuple[str, list[str]]]:
    chrome_flags = ["--kiosk", "--disable-infobars", "--no-first-run", "--disable-session-crashed-bubble"]
    edge_flags = ["--kiosk", "--edge-kiosk-type=fullscreen", "--no-first-run"]
    names = [
        ("google-chrome-stable", chrome_flags),
        ("google-chrome", chrome_flags),
        ("chromium-browser", chrome_flags),
        ("chromium", chrome_flags),
        ("microsoft-edge-stable", edge_flags),
        ("microsoft-edge", edge_flags),
        ("brave-browser", chrome_flags),
        ("firefox", ["--kiosk"]),
        ("firefox-esr", ["--kiosk"]),
    ]
    found: list[tuple[str, list[str]]] = []
    for name, flags in names:
        path = shutil.which(name)
        if path:
            found.append((path, flags))
    return found


def _profile_flag(binary: str) -> list[str]:
    global _profile_dir
    name = Path(binary).name.lower()
    if "firefox" in name:
        return []
    if _profile_dir is None:
        cache = Path.home() / ".cache" / "cyber-ocean-screensaver"
        cache.mkdir(parents=True, exist_ok=True)
        _profile_dir = Path(tempfile.mkdtemp(prefix="profile-", dir=str(cache)))
    return [f"--user-data-dir={_profile_dir}"]


def iter_browsers() -> list[tuple[str, list[str]]]:
    system = sys.platform
    if system == "win32":
        return [(p, f) for p, f in _win_paths() if Path(p).is_file()]
    if system == "darwin":
        return [(p, f) for p, f in _mac_paths() if Path(p).is_file()]
    return _linux_paths()


def launch_kiosk(url: str) -> bool:
    """全屏打开海洋馆。成功启动浏览器进程则返回 True。"""
    global _browser_proc
    extra = [
        "--disable-translate",
        "--autoplay-policy=no-user-gesture-required",
        "--noerrdialogs",
        "--disable-features=Translate,InfiniteSessionRestore",
    ]
    for binary, flags in iter_browsers():
        cmd = [binary, *flags]
        if "firefox" not in Path(binary).name.lower():
            cmd.extend(_profile_flag(binary))
            cmd.extend(extra)
        cmd.append(url)
        try:
            _browser_proc = subprocess.Popen(
                cmd,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                start_new_session=True,
            )
            print(f"全屏浏览器：{binary}")
            return True
        except OSError as exc:
            print(f"无法启动 {binary}：{exc}")
            _browser_proc = None
    return False


def watch_kiosk_then(callback) -> None:
    """浏览器主进程结束后回调（例如关掉本地服务）。"""

    def _run() -> None:
        proc = _browser_proc
        if proc is None:
            return
        try:
            proc.wait()
        except Exception:
            return
        callback()

    threading.Thread(target=_run, daemon=True).start()


def stop_kiosk() -> None:
    global _browser_proc
    proc = _browser_proc
    _browser_proc = None
    if proc is None:
        return
    if proc.poll() is not None:
        return
    try:
        proc.terminate()
        proc.wait(timeout=2)
    except Exception:
        try:
            proc.kill()
        except Exception:
            pass


def show_config_dialog() -> None:
    """Windows 屏保“设置”对话框。"""
    try:
        import tkinter as tk
        from tkinter import messagebox

        root = tk.Tk()
        root.withdraw()
        root.attributes("-topmost", True)
        messagebox.showinfo(
            "赛博海洋馆",
            "无需额外设置。\n\n作为屏幕保护运行时，移动鼠标或按任意键即可退出。",
        )
        root.destroy()
    except Exception:
        print("赛博海洋馆：无需设置。移动鼠标或按任意键退出。")
