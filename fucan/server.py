"""本地 HTTP 服务：北斗浮蚕海报展示 / 赛博海洋馆屏保。"""

from __future__ import annotations

import importlib
import socket
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlparse

from . import gallery, web_ui
from .math_model import BeidouParams
from .svg_render import rows_to_svg

ALLOW_QUIT = False
HTTPD: ThreadingHTTPServer | None = None


def _request_shutdown() -> None:
    time.sleep(0.12)
    server = HTTPD
    if server is None:
        return
    try:
        server.shutdown()
    except Exception:
        pass


class FucanHandler(BaseHTTPRequestHandler):
    server_version = "BeidouFucan/2.2"

    def log_message(self, fmt: str, *args) -> None:
        if self.path.startswith("/api/"):
            return
        super().log_message(fmt, *args)

    def _send(self, code: int, body: bytes, content_type: str) -> None:
        self.send_response(code)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(body)

    def _quit_if_allowed(self, path: str) -> bool:
        if path != "/api/quit":
            return False
        if not ALLOW_QUIT:
            self._send(403, b'{"ok":false}', "application/json")
            return True
        self._send(200, b'{"ok":true}', "application/json")
        threading.Thread(target=_request_shutdown, daemon=True).start()
        return True

    def do_POST(self) -> None:  # noqa: N802
        parsed = urlparse(self.path)
        if self._quit_if_allowed(parsed.path):
            return
        self._send(404, b"Not Found", "text/plain; charset=utf-8")

    def do_GET(self) -> None:  # noqa: N802
        parsed = urlparse(self.path)
        path = parsed.path
        query = parse_qs(parsed.query)

        if self._quit_if_allowed(path):
            return

        if path in ("/", "/index.html", "/gallery"):
            importlib.reload(gallery)
            self._send(200, gallery.INDEX_HTML.encode("utf-8"), "text/html; charset=utf-8")
            return

        if path in ("/screensaver", "/screensaver.html"):
            importlib.reload(gallery)
            html = gallery.INDEX_HTML.replace("<body>", '<body data-mode="saver">', 1)
            self._send(200, html.encode("utf-8"), "text/html; charset=utf-8")
            return

        if path in ("/lesson", "/fucan"):
            importlib.reload(web_ui)
            self._send(200, web_ui.INDEX_HTML.encode("utf-8"), "text/html; charset=utf-8")
            return

        if path == "/api/svg":
            try:
                t = float(query.get("t", ["0"])[0])
            except ValueError:
                t = 0.0
            svg = rows_to_svg(t=t)
            self._send(200, svg.encode("utf-8"), "image/svg+xml; charset=utf-8")
            return

        if path == "/api/health":
            self._send(200, b'{"ok":true}', "application/json")
            return

        self._send(404, b"Not Found", "text/plain; charset=utf-8")


def find_free_port(preferred: int = 8765) -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        try:
            s.bind(("127.0.0.1", preferred))
            return preferred
        except OSError:
            s.bind(("127.0.0.1", 0))
            return int(s.getsockname()[1])


def run_server(
    host: str = "127.0.0.1",
    port: int | None = None,
    open_browser: bool = True,
    path: str = "/",
    allow_quit: bool = False,
    kiosk: bool = False,
) -> None:
    global ALLOW_QUIT, HTTPD
    from .kiosk import launch_kiosk, stop_kiosk

    ALLOW_QUIT = allow_quit
    port = find_free_port(port or 8765)
    HTTPD = ThreadingHTTPServer((host, port), FucanHandler)
    url = f"http://{host}:{port}{path}"
    print("=" * 48, flush=True)
    if kiosk or path.startswith("/screensaver"):
        print("  赛博海洋馆 · 屏幕保护", flush=True)
        print(f"  {url}", flush=True)
        print("  移动鼠标或按任意键退出", flush=True)
    else:
        print("  数字海洋生命 · 集合馆", flush=True)
        print(f"  请在浏览器打开：{url}", flush=True)
        print("  分步讲解：{}/lesson".format(url.rstrip("/")), flush=True)
        print("  屏保预览：{}/screensaver".format(url.rstrip("/")), flush=True)
    print("  按 Ctrl+C 结束", flush=True)
    print("=" * 48, flush=True)

    if kiosk:
        threading.Timer(0.45, lambda: _open_kiosk_or_tab(url, open_browser)).start()
    elif open_browser:
        import webbrowser

        threading.Timer(0.6, lambda: webbrowser.open(url)).start()

    try:
        HTTPD.serve_forever()
    except KeyboardInterrupt:
        print("\n已关闭。")
    finally:
        stop_kiosk()
        try:
            HTTPD.server_close()
        except Exception:
            pass
        HTTPD = None
        ALLOW_QUIT = False


def _open_kiosk_or_tab(url: str, fallback_tab: bool) -> None:
    from .kiosk import launch_kiosk, watch_kiosk_then
    import webbrowser

    if launch_kiosk(url):
        watch_kiosk_then(_request_shutdown)
        return
    print("未找到 Chrome / Edge / Firefox，改为普通标签页。可按 F11 全屏。")
    if fallback_tab:
        webbrowser.open(url)


_ = BeidouParams
