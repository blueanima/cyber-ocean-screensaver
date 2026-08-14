"""原生屏保画质 / 密度等参数。与 native 读取同一份 JSON。"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any

QUALITY: dict[str, dict[str, Any]] = {
    "low": {"step": 4, "fps": 24, "legend_stride": 4, "count": 12},
    "medium": {"step": 2, "fps": 30, "legend_stride": 2, "count": 17},
    "high": {"step": 1, "fps": 30, "legend_stride": 1, "count": 17},
    "ultra": {"step": 1, "fps": 60, "legend_stride": 1, "count": 17},
}

DEFAULTS: dict[str, Any] = {
    "quality": "high",
    "step": 1,
    "fps": 30,
    "count": 17,
    "legend": True,
    "formula": True,
    "legend_stride": 1,
    "point_size": 1.0,
    "vsync": True,
    "lang": "zh",
}

UI: dict[str, dict[str, Any]] = {
    "zh": {
        "window": "赛博海洋馆 - 设置",
        "title": "赛博海洋馆 - 设置",
        "lang": "语言",
        "quality": "画质预设",
        "step": "粒子密度 (1 最密)",
        "fps": "帧率上限",
        "count": "生物数量 1-17",
        "legend": "显示左上图例",
        "formula": "显示生物公式",
        "legend_stride": "图例抽样 (1 全点)",
        "point_size": "点大小倍率",
        "vsync": "垂直同步",
        "hint_start": "点「开始」进入海洋。运行中右键可再打开本窗口。",
        "hint_apply": "点「应用」立即生效。运行中可再次右键打开。",
        "reset": "恢复默认",
        "cancel": "取消",
        "start": "开始",
        "close": "关闭",
        "apply": "应用",
        "qualities": {
            "low": "低",
            "medium": "中",
            "high": "高",
            "ultra": "最高",
        },
    },
    "en": {
        "window": "Cyber Ocean - Settings",
        "title": "Cyber Ocean - Settings",
        "lang": "Language",
        "quality": "Quality",
        "step": "Density (1 = finest)",
        "fps": "Max frame rate",
        "count": "Creatures (1-17)",
        "legend": "Show legend",
        "formula": "Show formulas",
        "legend_stride": "Legend detail (1 = full)",
        "point_size": "Point size",
        "vsync": "Vertical sync",
        "hint_start": "Click Start to enter the ocean. Right-click later to open this again.",
        "hint_apply": "Click Apply to save. Right-click the ocean to open this again.",
        "reset": "Reset",
        "cancel": "Cancel",
        "start": "Start",
        "close": "Close",
        "apply": "Apply",
        "qualities": {
            "low": "Low",
            "medium": "Medium",
            "high": "High",
            "ultra": "Ultra",
        },
    },
}

LANG_CHOICES = ("zh", "en")


def lang_option_labels(ui_lang: str) -> dict[str, str]:
    if ui_lang == "en":
        return {"zh": "Chinese", "en": "English"}
    return {"zh": "中文", "en": "English"}


def default_lang() -> str:
    loc = (
        os.environ.get("CYBER_OCEAN_LANG")
        or os.environ.get("LC_ALL")
        or os.environ.get("LC_MESSAGES")
        or os.environ.get("LANG")
        or ""
    ).lower()
    if loc.startswith("en"):
        return "en"
    return "zh"


def config_path() -> Path:
    env = os.environ.get("CYBER_OCEAN_CONFIG")
    if env:
        p = Path(env).expanduser()
        if p.suffix:
            return p
        return p / "settings.json"
    if sys.platform == "win32":
        base = Path(os.environ.get("APPDATA", Path.home() / "AppData" / "Roaming"))
        return base / "cyber-ocean" / "settings.json"
    xdg = os.environ.get("XDG_CONFIG_HOME")
    base = Path(xdg) if xdg else Path.home() / ".config"
    return base / "cyber-ocean" / "settings.json"


def load() -> dict[str, Any]:
    data = dict(DEFAULTS)
    data["lang"] = default_lang()
    q = QUALITY.get(str(data["quality"]), QUALITY["high"])
    data.update(q)
    data["quality"] = "high"
    path = config_path()
    if not path.is_file():
        return data
    try:
        raw = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return data
    if not isinstance(raw, dict):
        return data
    if "quality" in raw and str(raw["quality"]) in QUALITY:
        data["quality"] = str(raw["quality"])
        data.update(QUALITY[data["quality"]])
    if "lang" in raw and str(raw["lang"]) in LANG_CHOICES:
        data["lang"] = str(raw["lang"])
    for key in DEFAULTS:
        if key in raw and key not in ("quality", "lang"):
            data[key] = raw[key]
    return data


def save(data: dict[str, Any]) -> Path:
    path = config_path()
    path.parent.mkdir(parents=True, exist_ok=True)
    out = {k: data[k] for k in DEFAULTS if k in data}
    path.write_text(json.dumps(out, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    return path


def native_argv_from_ns(ns: Any) -> list[str]:
    """把 argparse 里显式给出的项转成 native CLI（配置文件由 native 自己读）。"""
    out: list[str] = []
    mapping = (
        ("quality", "--quality"),
        ("step", "--step"),
        ("fps", "--fps"),
        ("count", "--count"),
        ("legend_stride", "--legend-stride"),
        ("point_size", "--point-size"),
        ("seed", "--seed"),
        ("lang", "--lang"),
    )
    for attr, flag in mapping:
        val = getattr(ns, attr, None)
        if val is not None:
            out += [flag, str(val)]
    legend = getattr(ns, "legend", None)
    if legend is False:
        out.append("--no-legend")
    elif legend is True:
        out.append("--legend")
    formula = getattr(ns, "formula", None)
    if formula is False:
        out.append("--no-formula")
    elif formula is True:
        out.append("--formula")
    vsync = getattr(ns, "vsync", None)
    if vsync is False:
        out.append("--no-vsync")
    elif vsync is True:
        out.append("--vsync")
    return out


def _project_root() -> Path:
    return Path(__file__).resolve().parent.parent


def _cjk_font_files() -> list[Path]:
    names = (
        "DroidSansFallbackFull.ttf",
        "DroidSansFallback.ttf",
        "NotoSansSC-Regular.otf",
        "NotoSansCJKsc-Regular.otf",
    )
    dirs = [
        _project_root() / "fonts",
        Path(os.environ.get("APPDIR", "")) / "usr" / "share" / "fonts" / "cyber-ocean",
        Path("/usr/share/fonts/truetype/droid"),
        Path("/usr/share/fonts/opentype/noto"),
        Path("/usr/share/fonts/noto-cjk"),
        Path("/usr/share/fonts/truetype/wqy"),
        Path(os.environ.get("WINDIR", r"C:\Windows")) / "Fonts",
        Path("/System/Library/Fonts"),
        Path("/System/Library/Fonts/Supplemental"),
    ]
    out: list[Path] = []
    seen: set[Path] = set()
    for folder in dirs:
        if not folder or not folder.is_dir():
            continue
        for name in names:
            p = folder / name
            if p.is_file():
                key = p.resolve()
                if key not in seen:
                    seen.add(key)
                    out.append(p)
    extras = (
        Path("/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf"),
        Path("/usr/share/fonts/truetype/wqy/wqy-microhei.ttc"),
    )
    for p in extras:
        if p.is_file():
            key = p.resolve()
            if key not in seen:
                seen.add(key)
                out.append(p)
    return out


def _register_x11_cjk_font() -> None:
    """Tk 9 便携构建只用 X11 XLFD，看不到 fontconfig 里的 Noto CJK。"""
    if sys.platform != "linux":
        return
    files = [p for p in _cjk_font_files() if p.suffix.lower() == ".ttf"]
    if not files:
        return
    cache = Path(os.environ.get("XDG_CACHE_HOME", Path.home() / ".cache")) / "cyber-ocean" / "xfonts"
    try:
        cache.mkdir(parents=True, exist_ok=True)
    except OSError:
        return
    src = files[0]
    dest = cache / "DroidSansFallbackFull.ttf"
    try:
        if not dest.is_file() or dest.stat().st_size != src.stat().st_size:
            shutil.copy2(src, dest)
        # Tk 9 只用 XLFD：仅注册 Unicode 时，拉丁字母会落到希腊文编码的字体上。
        encodings = ("iso10646-1", "iso8859-1", "iso8859-15", "ascii-0")
        lines = [str(len(encodings))]
        for enc in encodings:
            lines.append(f"{dest.name} -misc-cyberocean-medium-r-normal--0-0-0-0-p-0-{enc}")
        listing = "\n".join(lines) + "\n"
        (cache / "fonts.scale").write_text(listing, encoding="ascii")
        (cache / "fonts.dir").write_text(listing, encoding="ascii")
    except OSError:
        return
    for args in (("xset", "+fp", str(cache)), ("xset", "fp", "rehash")):
        try:
            subprocess.run(args, check=False, capture_output=True, timeout=2)
        except (OSError, subprocess.TimeoutExpired):
            return


def _font_has_cjk(root, family: str) -> bool:
    import tkinter.font as tkfont

    font = tkfont.Font(root=root, family=family, size=16)
    w_hai = font.measure("海")
    w_guan = font.measure("馆")
    w_set = font.measure("设置")
    w_i = max(font.measure("i"), 1)
    if w_hai < w_i * 1.5 or w_guan < w_i * 1.5:
        return False
    if abs(w_hai - w_guan) > max(6, w_hai * 0.35):
        return False
    if w_set < w_hai * 1.5:
        return False
    return True


def _pick_latin_family(root) -> str:
    import tkinter.font as tkfont

    families = {name.lower(): name for name in tkfont.families(root)}
    for key in ("nimbus sans l", "helvetica", "courier", "fixed"):
        name = families.get(key)
        if name:
            return name
    return "fixed"


def _pick_ui_family(root, lang: str = "zh") -> str:
    import tkinter.font as tkfont

    families = {name.lower(): name for name in tkfont.families(root)}
    if lang == "en":
        return _pick_latin_family(root)
    preferred = (
        "song ti",
        "fangsong ti",
        "wenquanyi micro hei",
        "wenquanyi zen hei",
        "noto sans cjk sc",
        "noto sans sc",
        "source han sans sc",
        "droid sans fallback",
        "microsoft yahei ui",
        "microsoft yahei",
        "pingfang sc",
        "cyberocean",
    )
    for key in preferred:
        name = families.get(key)
        if name and _font_has_cjk(root, name):
            return name
    for key, name in families.items():
        if any(tag in key for tag in ("cjk", "droid", "uming", "ukai", "wqy", "yahei", "pingfang")):
            if _font_has_cjk(root, name):
                return name
    return "song ti"


def _ui_scale(root) -> float:
    """只用于窗口边距，不再改 tk scaling（X11 核心字体会被拉变形）。"""
    env = (
        os.environ.get("CYBER_OCEAN_UI_SCALE")
        or os.environ.get("GDK_SCALE")
        or os.environ.get("QT_SCALE_FACTOR")
    )
    if env:
        try:
            return max(0.9, min(1.4, float(env)))
        except ValueError:
            pass
    try:
        sw = max(640, int(root.winfo_screenwidth()))
        sh = max(480, int(root.winfo_screenheight()))
    except Exception:
        sw, sh = 1920, 1080
    fit = min(sw / 1920.0, sh / 1080.0)
    return max(0.9, min(1.25, fit))


def _is_bitmap_family(family: str) -> bool:
    key = family.lower()
    return any(tag in key for tag in ("song ti", "fangsong", "mincho", "fixed", "clearlyu"))


def _ui_font_px(width: int, family: str) -> int:
    """X11 点阵宋体只有 16 / 24 像素；可缩放字体按窗口宽度选字号。"""
    width = max(320, int(width))
    if _is_bitmap_family(family):
        return 24 if width >= 680 else 16
    if width >= 920:
        return 18
    if width >= 640:
        return 16
    return 14


def _break_text(font: Any, text: str, max_px: int) -> str:
    """ttk.Checkbutton 没有 wraplength，按像素宽度插入换行。"""
    text = str(text)
    if max_px < 48 or not text:
        return text
    try:
        if font.measure(text) <= max_px:
            return text
    except Exception:
        return text
    lines: list[str] = []
    buf = ""
    for ch in text:
        trial = buf + ch
        try:
            too_wide = bool(buf) and font.measure(trial) > max_px
        except Exception:
            too_wide = len(trial) > 16
        if too_wide:
            lines.append(buf)
            buf = "" if ch == " " else ch
        else:
            buf = trial
    if buf:
        lines.append(buf)
    return "\n".join(lines) if lines else text


def _prepare_dialog(root, lang: str = "zh") -> tuple[float, dict[str, Any]]:
    import tkinter.font as tkfont
    from tkinter import ttk

    try:
        root.tk.call("encoding", "system", "utf-8")
    except Exception:
        pass
    scale = _ui_scale(root)
    try:
        root.tk.call("tk", "scaling", 1.333333)
    except Exception:
        pass
    family = _pick_ui_family(root, lang)
    latin_family = _pick_latin_family(root)
    try:
        start_w = int(root.winfo_screenwidth() * 0.36)
    except Exception:
        start_w = 560
    px = _ui_font_px(start_w, family)
    title_px = 24 if _is_bitmap_family(family) else px + 4
    small_px = 16 if _is_bitmap_family(family) else max(11, px - 2)
    body = tkfont.Font(root=root, family=family, size=-px)
    title = tkfont.Font(root=root, family=family, size=-title_px, weight="bold")
    small = tkfont.Font(root=root, family=family, size=-small_px)
    latin = tkfont.Font(root=root, family=latin_family, size=-px)
    latin_small = tkfont.Font(root=root, family=latin_family, size=-small_px)
    for name in (
        "TkDefaultFont",
        "TkTextFont",
        "TkMenuFont",
        "TkHeadingFont",
        "TkCaptionFont",
        "TkSmallCaptionFont",
        "TkIconFont",
        "TkTooltipFont",
    ):
        try:
            tkfont.nametofont(name).configure(family=family, size=-px)
        except Exception:
            pass
    try:
        tkfont.nametofont("TkFixedFont").configure(family=latin_family, size=-px)
    except Exception:
        pass
    style = ttk.Style(root)
    try:
        style.theme_use("clam")
    except Exception:
        pass
    pad_y = 8
    style.configure(".", font=body, padding=4)
    style.configure("TFrame", background="#0a1628")
    style.configure("TLabel", font=body, background="#0a1628", foreground="#e8f0ff")
    style.configure("Hint.TLabel", font=body, background="#0a1628", foreground="#9ab4d4")
    style.configure("Path.TLabel", font=latin_small, background="#0a1628", foreground="#7a90b0")
    style.configure("Title.TLabel", font=title, background="#0a1628", foreground="#ffffff")
    style.configure("TCheckbutton", font=body, background="#0a1628", foreground="#e8f0ff")
    style.configure(
        "TButton",
        font=body,
        padding=(px + 6, max(12, px - 2)),
        foreground="#e8f0ff",
        background="#1a2e4a",
        bordercolor="#8ab4ff",
        lightcolor="#8ab4ff",
        darkcolor="#06101c",
        focusthickness=1,
        focuscolor="#c8dcff",
    )
    style.map(
        "TButton",
        foreground=[("active", "#06101c")],
        background=[("active", "#e8f0ff")],
    )
    style.configure("TCombobox", font=body, padding=6)
    style.configure("TSpinbox", font=latin, padding=6)
    style.layout(
        "HudLabel.TCheckbutton",
        [
            (
                "Checkbutton.padding",
                {"sticky": "nswe", "children": [("Checkbutton.label", {"sticky": "w"})]},
            )
        ],
    )
    style.layout("HudTitle.TCheckbutton", style.layout("HudLabel.TCheckbutton"))
    style.layout("HudHint.TCheckbutton", style.layout("HudLabel.TCheckbutton"))
    for name, font, fg in (
        ("HudLabel.TCheckbutton", body, "#e8f0ff"),
        ("HudTitle.TCheckbutton", title, "#ffffff"),
        ("HudHint.TCheckbutton", body, "#9ab4d4"),
    ):
        style.configure(name, font=font, background="#0a1628", foreground=fg, padding=0)
        style.map(
            name,
            background=[("active", "#0a1628"), ("pressed", "#0a1628"), ("selected", "#0a1628")],
            foreground=[("active", fg), ("pressed", fg), ("selected", fg)],
        )
    root.option_add("*TCombobox*Listbox.font", body)
    root.configure(bg="#0a1628", highlightthickness=2, highlightbackground="#8ab4ff", highlightcolor="#c8dcff")
    return scale, {
        "body": body,
        "title": title,
        "small": small,
        "latin": latin,
        "latin_small": latin_small,
        "pad_y": pad_y,
        "style": style,
        "family": family,
        "px": px,
    }


def show_config_dialog(*, startable: bool = False) -> str:
    """画质 / 密度 / 数量设置。返回 start / saved / cancel。"""
    try:
        import tkinter as tk
        from tkinter import ttk
    except Exception:
        data = load()
        print(f"当前设置：{data}")
        print(f"编辑 {config_path()} 后重启屏保。")
        return "start" if startable else "cancel"

    data = load()
    result = {"status": "cancel"}
    _register_x11_cjk_font()
    root = tk.Tk()
    root.attributes("-topmost", True)
    raw_lang = str(data.get("lang", "zh"))
    if raw_lang not in LANG_CHOICES:
        raw_lang = default_lang()
    _scale, fonts = _prepare_dialog(root, raw_lang)
    pad = {"padx": 8, "pady": 5}
    frm = ttk.Frame(root, padding=12)
    frm.grid(sticky="nsew")
    root.columnconfigure(0, weight=1)
    root.rowconfigure(0, weight=1)
    frm.columnconfigure(0, weight=1, minsize=168)
    frm.columnconfigure(1, weight=1, minsize=140)

    style = fonts["style"]
    bg = "#0a1628"
    dummy = tk.BooleanVar(value=False)

    def mk_label(kind: str = "HudLabel.TCheckbutton") -> Any:
        w = ttk.Checkbutton(frm, style=kind, takefocus=False, variable=dummy)
        w.bind("<Button-1>", lambda _e: "break")
        w.bind("<space>", lambda _e: "break")
        return w

    lang = tk.StringVar(value=raw_lang)
    quality = tk.StringVar(value=str(data.get("quality", "high")))
    step = tk.IntVar(value=int(data.get("step", 1)))
    fps = tk.IntVar(value=int(data.get("fps", 30)))
    count = tk.IntVar(value=int(data.get("count", 17)))
    legend = tk.BooleanVar(value=bool(data.get("legend", True)))
    formula = tk.BooleanVar(value=bool(data.get("formula", True)))
    legend_stride = tk.IntVar(value=int(data.get("legend_stride", 1)))
    point_size = tk.DoubleVar(value=float(data.get("point_size", 1.0)))
    vsync = tk.BooleanVar(value=bool(data.get("vsync", True)))

    def ui() -> dict[str, Any]:
        key = lang.get() if lang.get() in LANG_CHOICES else "zh"
        return UI[key]

    def apply_quality(*_args: object) -> None:
        preset = QUALITY.get(quality.get())
        if not preset:
            return
        step.set(int(preset["step"]))
        fps.set(int(preset["fps"]))
        count.set(int(preset["count"]))
        legend_stride.set(int(preset["legend_stride"]))

    def collect() -> dict[str, Any]:
        out = dict(DEFAULTS)
        out.update(
            {
                "lang": lang.get() if lang.get() in LANG_CHOICES else "zh",
                "quality": quality.get(),
                "step": int(step.get()),
                "fps": int(fps.get()),
                "count": int(count.get()),
                "legend": bool(legend.get()),
                "formula": bool(formula.get()),
                "legend_stride": int(legend_stride.get()),
                "point_size": float(point_size.get()),
                "vsync": bool(vsync.get()),
            }
        )
        return out

    def quality_key_from_label(label: str) -> str:
        qualities = ui()["qualities"]
        for key, text in qualities.items():
            if text == label:
                return key
        return quality.get()

    title_lbl = mk_label("HudTitle.TCheckbutton")
    title_lbl.grid(row=0, column=0, columnspan=2, sticky="ew", **pad)
    lang_lbl = mk_label()
    lang_lbl.grid(row=1, column=0, sticky="ew", **pad)
    lang_box = ttk.Combobox(
        frm,
        values=[lang_option_labels(raw_lang)[k] for k in LANG_CHOICES],
        state="readonly",
        width=16,
    )
    lang_box.grid(row=1, column=1, sticky="ew", **pad)
    quality_lbl = mk_label()
    quality_lbl.grid(row=2, column=0, sticky="ew", **pad)
    qbox = ttk.Combobox(frm, state="readonly", width=16)
    qbox.grid(row=2, column=1, sticky="ew", **pad)

    spin_kw = {"width": 16}
    step_lbl = mk_label()
    step_lbl.grid(row=3, column=0, sticky="ew", **pad)
    ttk.Spinbox(frm, from_=1, to=12, textvariable=step, **spin_kw).grid(
        row=3, column=1, sticky="ew", **pad
    )
    fps_lbl = mk_label()
    fps_lbl.grid(row=4, column=0, sticky="ew", **pad)
    ttk.Spinbox(frm, from_=10, to=120, textvariable=fps, **spin_kw).grid(
        row=4, column=1, sticky="ew", **pad
    )
    count_lbl = mk_label()
    count_lbl.grid(row=5, column=0, sticky="ew", **pad)
    ttk.Spinbox(frm, from_=1, to=17, textvariable=count, **spin_kw).grid(
        row=5, column=1, sticky="ew", **pad
    )
    legend_chk = ttk.Checkbutton(frm, variable=legend)
    legend_chk.grid(row=6, column=0, columnspan=2, sticky="w", **pad)
    formula_chk = ttk.Checkbutton(frm, variable=formula)
    formula_chk.grid(row=7, column=0, columnspan=2, sticky="w", **pad)
    stride_lbl = mk_label()
    stride_lbl.grid(row=8, column=0, sticky="ew", **pad)
    ttk.Spinbox(frm, from_=1, to=16, textvariable=legend_stride, **spin_kw).grid(
        row=8, column=1, sticky="ew", **pad
    )
    size_lbl = mk_label()
    size_lbl.grid(row=9, column=0, sticky="ew", **pad)
    ttk.Spinbox(
        frm, from_=0.4, to=3.0, increment=0.1, textvariable=point_size, **spin_kw
    ).grid(row=9, column=1, sticky="ew", **pad)
    vsync_chk = ttk.Checkbutton(frm, variable=vsync)
    vsync_chk.grid(row=10, column=0, columnspan=2, sticky="w", **pad)
    hint_lbl = mk_label("HudHint.TCheckbutton")
    hint_lbl.grid(row=11, column=0, columnspan=2, sticky="ew", **pad)
    path_lbl = tk.Label(
        frm,
        text=str(config_path()),
        font=fonts["latin_small"],
        bg=bg,
        fg="#7a90b0",
        anchor="w",
        justify="left",
    )
    path_lbl.grid(row=12, column=0, columnspan=2, sticky="w", **pad)

    btns = ttk.Frame(frm)
    btns.grid(row=13, column=0, columnspan=2, sticky="e", **pad)

    reset_btn = ttk.Button(btns)
    reset_btn.pack(side="left", padx=6, ipadx=4, ipady=2)
    extra_btn = ttk.Button(btns)
    extra_btn.pack(side="left", padx=6, ipadx=4, ipady=2)
    primary_btn = ttk.Button(btns)
    primary_btn.pack(side="left", padx=6, ipadx=4, ipady=2)

    ui_state = {"px": int(fonts["px"]), "family": str(fonts["family"]), "width": 0}

    def apply_fonts(family: str, px: int) -> None:
        title_px = 24 if _is_bitmap_family(family) else px + 4
        small_px = 16 if _is_bitmap_family(family) else max(11, px - 2)
        fonts["body"].configure(family=family, size=-px)
        fonts["title"].configure(family=family, size=-title_px)
        fonts["small"].configure(family=family, size=-small_px)
        fonts["latin"].configure(size=-px)
        fonts["latin_small"].configure(size=-small_px)
        style.configure("TCheckbutton", font=fonts["body"])
        style.configure("HudLabel.TCheckbutton", font=fonts["body"])
        style.configure("HudTitle.TCheckbutton", font=fonts["title"])
        style.configure("HudHint.TCheckbutton", font=fonts["body"])
        style.configure(
            "TButton",
            font=fonts["body"],
            padding=(px + 6, max(12, px - 2)),
        )
        style.configure("TCombobox", font=fonts["body"])
        style.configure("TSpinbox", font=fonts["latin"])
        path_lbl.configure(font=fonts["latin_small"])
        ui_state["px"] = px
        ui_state["family"] = family

    raw_texts: dict[str, str] = {}

    def apply_wrap(width: int) -> None:
        wrap_full = max(220, width - 48)
        wrap_col = max(140, int(width * 0.48) - 20)
        try:
            path_lbl.configure(wraplength=wrap_full)
        except tk.TclError:
            pass
        pairs = (
            ("title", title_lbl, fonts["title"], wrap_full),
            ("hint", hint_lbl, fonts["body"], wrap_full),
            ("lang", lang_lbl, fonts["body"], wrap_col),
            ("quality", quality_lbl, fonts["body"], wrap_col),
            ("step", step_lbl, fonts["body"], wrap_col),
            ("fps", fps_lbl, fonts["body"], wrap_col),
            ("count", count_lbl, fonts["body"], wrap_col),
            ("stride", stride_lbl, fonts["body"], wrap_col),
            ("size", size_lbl, fonts["body"], wrap_col),
            ("legend", legend_chk, fonts["body"], wrap_full),
            ("formula", formula_chk, fonts["body"], wrap_full),
            ("vsync", vsync_chk, fonts["body"], wrap_full),
        )
        for key, widget, font, limit in pairs:
            text = raw_texts.get(key)
            if text is None:
                continue
            widget.configure(text=_break_text(font, text, limit))

    def refresh_texts() -> None:
        texts = ui()
        current = lang.get() if lang.get() in LANG_CHOICES else "zh"
        family = _pick_ui_family(root, current)
        width = max(ui_state["width"], int(root.winfo_width() or 560))
        apply_fonts(family, _ui_font_px(width, family))
        root.title(str(texts["window"]))
        raw_texts["title"] = str(texts["title"])
        raw_texts["lang"] = str(texts["lang"])
        raw_texts["quality"] = str(texts["quality"])
        raw_texts["step"] = str(texts["step"])
        raw_texts["fps"] = str(texts["fps"])
        raw_texts["count"] = str(texts["count"])
        raw_texts["legend"] = str(texts["legend"])
        raw_texts["formula"] = str(texts["formula"])
        raw_texts["stride"] = str(texts["legend_stride"])
        raw_texts["size"] = str(texts["point_size"])
        raw_texts["vsync"] = str(texts["vsync"])
        raw_texts["hint"] = str(texts["hint_start"] if startable else texts["hint_apply"])
        labels = lang_option_labels(current)
        lang_box.configure(values=[labels[k] for k in LANG_CHOICES])
        lang_box.set(labels.get(current, labels["zh"]))
        qualities = texts["qualities"]
        qbox.configure(values=[qualities[k] for k in QUALITY])
        qbox.set(qualities.get(quality.get(), quality.get()))
        reset_btn.configure(text=str(texts["reset"]))
        if startable:
            extra_btn.configure(text=str(texts["cancel"]))
            primary_btn.configure(text=str(texts["start"]))
        else:
            extra_btn.configure(text=str(texts["close"]))
            primary_btn.configure(text=str(texts["apply"]))
        apply_wrap(max(ui_state["width"], width))

    def on_lang_selected(_event: object = None) -> None:
        selected = lang_box.get()
        for ui_lang in (lang.get(), "zh", "en"):
            labels = lang_option_labels(ui_lang if ui_lang in LANG_CHOICES else "zh")
            for key, text in labels.items():
                if text == selected:
                    lang.set(key)
                    refresh_texts()
                    return
        refresh_texts()

    def on_quality_selected(_event: object = None) -> None:
        quality.set(quality_key_from_label(qbox.get()))
        apply_quality()

    def do_reset() -> None:
        quality.set("high")
        apply_quality()
        legend.set(True)
        formula.set(True)
        point_size.set(1.0)
        vsync.set(True)
        refresh_texts()

    def do_apply() -> None:
        path = save(collect())
        print(f"已保存设置：{path}")
        result["status"] = "saved"

    def do_start() -> None:
        do_apply()
        result["status"] = "start"
        root.destroy()

    def do_close() -> None:
        root.destroy()

    lang_box.bind("<<ComboboxSelected>>", on_lang_selected)
    qbox.bind("<<ComboboxSelected>>", on_quality_selected)
    reset_btn.configure(command=do_reset)
    extra_btn.configure(command=do_close)
    primary_btn.configure(command=do_start if startable else do_apply)
    refresh_texts()

    def fit_window() -> None:
        root.update_idletasks()
        sw = max(640, int(root.winfo_screenwidth()))
        sh = max(480, int(root.winfo_screenheight()))
        req_w = max(frm.winfo_reqwidth() + 28, 560)
        req_h = max(frm.winfo_reqheight() + 36, 540)
        w = min(req_w, int(sw * 0.92))
        h = min(req_h, int(sh * 0.90))
        x = max(0, (sw - w) // 2)
        y = max(0, (sh - h) // 6)
        root.geometry(f"{w}x{h}+{x}+{y}")
        root.minsize(min(480, w), min(420, h))
        ui_state["width"] = w
        apply_wrap(w)

    def on_configure(event: object) -> None:
        if getattr(event, "widget", None) is not root:
            return
        width = int(getattr(event, "width", 0) or 0)
        if width < 80:
            return
        current = lang.get() if lang.get() in LANG_CHOICES else "zh"
        family = _pick_ui_family(root, current)
        px = _ui_font_px(width, family)
        if px != ui_state["px"] or family != ui_state["family"]:
            apply_fonts(family, px)
        ui_state["width"] = width
        apply_wrap(width)

    root.protocol("WM_DELETE_WINDOW", do_close)
    root.resizable(True, True)
    root.update_idletasks()
    fit_window()
    root.bind("<Configure>", on_configure)
    try:
        root.lift()
        root.focus_force()
    except Exception:
        pass
    root.mainloop()
    return result["status"]


if __name__ == "__main__":
    startable = "--startable" in sys.argv
    status = show_config_dialog(startable=startable)
    sys.exit(0 if status in ("start", "saved") else 1)
