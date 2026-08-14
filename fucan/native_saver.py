#!/usr/bin/env python3
"""全屏原生窗口（GTK + WebKitGTK），铺满屏幕，不是浏览器页面。"""

from __future__ import annotations

import sys
import time
from ctypes import CDLL, CFUNCTYPE, c_int, c_void_p
from urllib.parse import urlparse
from urllib.request import urlopen

KEEP: list[object] = []
GDK_BLANK_CURSOR = -2


def _load_webkit() -> CDLL:
    last: OSError | None = None
    for name in ("libwebkit2gtk-4.1.so.0", "libwebkit2gtk-4.0.so.0"):
        try:
            return CDLL(name)
        except OSError as exc:
            last = exc
    raise last or OSError("WebKitGTK not found")


def main(argv: list[str]) -> int:
    if argv[1:] == ["--self-test"]:
        gtk = CDLL("libgtk-3.so.0")
        wk = _load_webkit()
        gtk.gtk_init(None, None)
        gtk.gtk_window_new.restype = c_void_p
        wk.webkit_web_view_new.restype = c_void_p
        if not gtk.gtk_window_new(0) or not wk.webkit_web_view_new():
            print("native saver: widget create failed", file=sys.stderr)
            return 1
        print("native saver: ok")
        return 0

    url = next((a for a in argv[1:] if a.startswith("http://") or a.startswith("https://")), "")
    if not url:
        print("usage: native_saver.py URL", file=sys.stderr)
        return 2

    gtk = CDLL("libgtk-3.so.0")
    gdk = CDLL("libgdk-3.so.0")
    glib = CDLL("libglib-2.0.so.0")
    gobj = CDLL("libgobject-2.0.so.0")
    wk = _load_webkit()

    gtk.gtk_init(None, None)
    gtk.gtk_window_new.restype = c_void_p
    gtk.gtk_widget_get_window.restype = c_void_p
    wk.webkit_web_view_new.restype = c_void_p
    gdk.gdk_display_get_default.restype = c_void_p
    gdk.gdk_screen_get_default.restype = c_void_p
    gdk.gdk_cursor_new_for_display.restype = c_void_p
    gdk.gdk_screen_get_n_monitors.restype = c_int
    gdk.gdk_screen_get_n_monitors.argtypes = [c_void_p]
    gobj.g_signal_connect_data.restype = c_void_p

    CloseCb = CFUNCTYPE(None, c_void_p, c_void_p)
    DeleteCb = CFUNCTYPE(c_int, c_void_p, c_void_p, c_void_p)
    MenuCb = CFUNCTYPE(c_int, c_void_p, c_void_p, c_void_p, c_void_p, c_void_p)
    RealizeCb = CFUNCTYPE(None, c_void_p, c_void_p)
    PollCb = CFUNCTYPE(c_int, c_void_p)

    def on_delete(_w, _e, _d) -> int:
        gtk.gtk_main_quit()
        return 0

    def on_menu(_v, _m, _e, _h, _d) -> int:
        return 1

    def on_realize(widget, _d) -> None:
        display = gdk.gdk_display_get_default()
        if not display:
            return
        cursor = gdk.gdk_cursor_new_for_display(display, GDK_BLANK_CURSOR)
        gdk_win = gtk.gtk_widget_get_window(widget)
        if gdk_win and cursor:
            gdk.gdk_window_set_cursor(gdk_win, cursor)

    close_cb = CloseCb(lambda _v, _d: gtk.gtk_main_quit())
    delete_cb = DeleteCb(on_delete)
    menu_cb = MenuCb(on_menu)
    realize_cb = RealizeCb(on_realize)
    KEEP.extend((close_cb, delete_cb, menu_cb, realize_cb))

    uri = url.encode("utf-8")
    win = gtk.gtk_window_new(0)
    gtk.gtk_window_set_title(win, b"Cyber Ocean")
    gtk.gtk_window_set_decorated(win, 0)
    gtk.gtk_window_set_keep_above(win, 1)
    gtk.gtk_window_set_skip_taskbar_hint(win, 1)
    gtk.gtk_window_set_skip_pager_hint(win, 1)
    gtk.gtk_window_stick(win)
    view = wk.webkit_web_view_new()
    settings = None
    get_settings = getattr(wk, "webkit_web_view_get_settings", None)
    if get_settings:
        get_settings.restype = c_void_p
        settings = get_settings(view)
    if settings:
        for name, val in (
            ("webkit_settings_set_hardware_acceleration_policy", 1),
            ("webkit_settings_set_enable_webaudio", 0),
            ("webkit_settings_set_enable_media", 0),
            ("webkit_settings_set_enable_media_stream", 0),
            ("webkit_settings_set_enable_html5_database", 0),
            ("webkit_settings_set_enable_offline_web_application_cache", 0),
            ("webkit_settings_set_enable_page_cache", 0),
            ("webkit_settings_set_enable_smooth_scrolling", 0),
        ):
            fn = getattr(wk, name, None)
            if fn:
                try:
                    fn(settings, val)
                except Exception:
                    pass
    gtk.gtk_container_add(win, view)
    wk.webkit_web_view_load_uri(view, uri)
    gobj.g_signal_connect_data(view, b"close", close_cb, None, None, 0)
    gobj.g_signal_connect_data(view, b"context-menu", menu_cb, None, None, 0)
    gobj.g_signal_connect_data(win, b"delete-event", delete_cb, None, None, 0)
    gobj.g_signal_connect_data(win, b"realize", realize_cb, None, None, 0)
    gtk.gtk_widget_show_all(win)
    gtk.gtk_window_fullscreen(win)

    parsed = urlparse(url)
    health = f"{parsed.scheme}://{parsed.netloc}/api/health"
    fails = [0]
    started = time.time()

    def poll(_d) -> int:
        if time.time() - started < 2.5:
            return 1
        try:
            urlopen(health, timeout=0.35)
            fails[0] = 0
            return 1
        except Exception:
            fails[0] += 1
            if fails[0] >= 5:
                gtk.gtk_main_quit()
                return 0
            return 1

    poll_cb = PollCb(poll)
    KEEP.append(poll_cb)
    glib.g_timeout_add(1200, poll_cb, None)
    gtk.gtk_main()
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv))
    except OSError as exc:
        print(f"native saver: {exc}", file=sys.stderr)
        raise SystemExit(1)
