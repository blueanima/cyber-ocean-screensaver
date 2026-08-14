"""全屏屏保窗口：优先系统原生窗口，其次 Chrome/Firefox kiosk（绝不打开普通网页标签）。"""

from __future__ import annotations

import os
import shutil
import signal
import subprocess
import sys
import tempfile
import threading
from pathlib import Path

_browser_proc: subprocess.Popen[bytes] | None = None
_profile_dir: Path | None = None


def _cache_dir() -> Path:
    cache = Path.home() / ".cache" / "cyber-ocean-screensaver"
    cache.mkdir(parents=True, exist_ok=True)
    return cache


def _win_paths() -> list[tuple[str, list[str]]]:
    local = os.environ.get("LOCALAPPDATA", "")
    pf = os.environ.get("PROGRAMFILES", r"C:\Program Files")
    pf86 = os.environ.get("PROGRAMFILES(X86)", r"C:\Program Files (x86)")
    chrome_flags = _chrome_flags()
    edge_flags = ["--kiosk", "--edge-kiosk-type=fullscreen", "--no-first-run"]
    return [
        (str(Path(pf) / "Google" / "Chrome" / "Application" / "chrome.exe"), chrome_flags),
        (str(Path(local) / "Google" / "Chrome" / "Application" / "chrome.exe"), chrome_flags),
        (str(Path(pf) / "Microsoft" / "Edge" / "Application" / "msedge.exe"), edge_flags),
        (str(Path(pf86) / "Microsoft" / "Edge" / "Application" / "msedge.exe"), edge_flags),
        (str(Path(pf) / "Chromium" / "Application" / "chrome.exe"), chrome_flags),
        (str(Path(pf) / "Mozilla Firefox" / "firefox.exe"), []),
    ]


def _mac_paths() -> list[tuple[str, list[str]]]:
    chrome_flags = _chrome_flags()
    edge_flags = ["--kiosk", "--edge-kiosk-type=fullscreen", "--no-first-run"]
    return [
        ("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome", chrome_flags),
        ("/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge", edge_flags),
        ("/Applications/Chromium.app/Contents/MacOS/Chromium", chrome_flags),
        ("/Applications/Brave Browser.app/Contents/MacOS/Brave Browser", chrome_flags),
        ("/Applications/Firefox.app/Contents/MacOS/firefox", []),
    ]


def _chrome_flags() -> list[str]:
    return [
        "--kiosk",
        "--start-fullscreen",
        "--disable-infobars",
        "--no-first-run",
        "--disable-session-crashed-bubble",
        "--disable-restore-session-state",
        "--no-default-browser-check",
        "--disable-hang-monitor",
        "--hide-scrollbars",
        "--overscroll-history-navigation=0",
        "--disable-pinch",
        "--disable-translate",
        "--autoplay-policy=no-user-gesture-required",
        "--noerrdialogs",
        "--disable-features=Translate,TranslateUI,InfiniteSessionRestore",
        "--class=CyberOcean",
        "--name=CyberOcean",
    ]


def _linux_paths() -> list[tuple[str, list[str]]]:
    chrome_flags = _chrome_flags()
    edge_flags = ["--kiosk", "--edge-kiosk-type=fullscreen", "--no-first-run"]
    names = [
        ("google-chrome-stable", chrome_flags),
        ("google-chrome", chrome_flags),
        ("chromium-browser", chrome_flags),
        ("chromium", chrome_flags),
        ("microsoft-edge-stable", edge_flags),
        ("microsoft-edge", edge_flags),
        ("brave-browser", chrome_flags),
        ("firefox", []),
        ("firefox-esr", []),
    ]
    found: list[tuple[str, list[str]]] = []
    for name, flags in names:
        path = shutil.which(name)
        if path:
            found.append((path, flags))
    return found


def _chrome_profile_flag(binary: str) -> list[str]:
    global _profile_dir
    name = Path(binary).name.lower()
    if "firefox" in name:
        return []
    if _profile_dir is None:
        _profile_dir = Path(tempfile.mkdtemp(prefix="profile-", dir=str(_cache_dir())))
    return [f"--user-data-dir={_profile_dir}"]


def _firefox_cmd(binary: str, url: str) -> list[str]:
    profile = _cache_dir() / "firefox-kiosk"
    profile.mkdir(parents=True, exist_ok=True)
    user_js = profile / "user.js"
    user_js.write_text(
        "\n".join(
            [
                'user_pref("full-screen-api.ignore-widgets", true);',
                'user_pref("browser.fullscreen.autohide", true);',
                'user_pref("browser.tabs.warnOnClose", false);',
                'user_pref("toolkit.legacyUserProfileCustomizations.stylesheets", true);',
            ]
        )
        + "\n",
        encoding="utf-8",
    )
    chrome_dir = profile / "chrome"
    chrome_dir.mkdir(exist_ok=True)
    (chrome_dir / "userChrome.css").write_text(
        "#navigator-toolbox, #TabsToolbar, #nav-bar, #PersonalToolbar { display: none !important; }\n",
        encoding="utf-8",
    )
    return [
        binary,
        "--kiosk",
        "--no-remote",
        "--new-instance",
        "--profile",
        str(profile),
        url,
    ]


def iter_browsers() -> list[tuple[str, list[str]]]:
    system = sys.platform
    if system == "win32":
        return [(p, f) for p, f in _win_paths() if Path(p).is_file()]
    if system == "darwin":
        return [(p, f) for p, f in _mac_paths() if Path(p).is_file()]
    return _linux_paths()


def _python_candidates() -> list[str]:
    out: list[str] = []
    for raw in (sys.executable, "/usr/bin/python3"):
        if raw and Path(raw).is_file() and raw not in out:
            out.append(raw)
    return out


def launch_native_view(url: str) -> bool:
    """系统全屏窗口（无地址栏、无标签）。成功则记录进程并返回 True。"""
    global _browser_proc
    script = Path(__file__).resolve().parent / "native_saver.py"
    if not script.is_file():
        return False
    for py in _python_candidates():
        try:
            proc = subprocess.Popen(
                [py, str(script), url],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                start_new_session=True,
            )
        except OSError:
            continue
        try:
            proc.wait(timeout=0.9)
        except subprocess.TimeoutExpired:
            _browser_proc = proc
            print("全屏屏保窗口已打开（铺满屏幕）")
            return True
        # 立刻退出：这个 Python 加载不了 GTK/WebKit，试下一个
    return False


def launch_kiosk(url: str) -> bool:
    """铺满屏幕。优先原生窗口，绝不退回普通浏览器标签。"""
    global _browser_proc
    if launch_native_view(url):
        return True

    extra = [
        "--disable-translate",
        "--autoplay-policy=no-user-gesture-required",
        "--noerrdialogs",
        "--disable-features=Translate,InfiniteSessionRestore",
    ]
    for binary, flags in iter_browsers():
        name = Path(binary).name.lower()
        if "firefox" in name:
            cmd = _firefox_cmd(binary, url)
        else:
            cmd = [binary, *flags, *_chrome_profile_flag(binary), *extra, f"--app={url}"]
        try:
            _browser_proc = subprocess.Popen(
                cmd,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                start_new_session=True,
            )
            print(f"全屏 kiosk：{binary}")
            return True
        except OSError as exc:
            print(f"无法启动 {binary}：{exc}")
            _browser_proc = None
    return False


def watch_kiosk_then(callback) -> None:
    """屏保窗口结束后回调（例如关掉本地服务）。"""

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
        os.killpg(proc.pid, signal.SIGTERM)
    except Exception:
        try:
            proc.terminate()
        except Exception:
            pass
    try:
        proc.wait(timeout=2)
    except Exception:
        try:
            os.killpg(proc.pid, signal.SIGKILL)
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
