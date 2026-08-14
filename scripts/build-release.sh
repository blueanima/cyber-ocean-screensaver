#!/usr/bin/env bash
# 打出成品：AppImage、便携 zip、离线 HTML。
# 环境变量：
#   VERSION=1.0.2          成品文件名版本（不设则读 VERSION 文件；已打过标签则自动升补丁号）
#   SYSTEM_PYTHON=1        不捆绑 CPython，AppImage 使用系统 python3
#   GITHUB_MIRROR=https://ghfast.top/  强制镜像前缀
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIST="$ROOT/dist"
CACHE="$ROOT/.cache/build"
chmod +x "$ROOT/scripts/bump-version.sh"
if [[ -z "${VERSION:-}" ]]; then
  VERSION="$("$ROOT/scripts/bump-version.sh")"
else
  VERSION="${VERSION#v}"
fi
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64|amd64) ARCH=x86_64 ;;
  aarch64|arm64) ARCH=aarch64 ;;
esac

PY_TAG="20260310"
PY_VER="3.12.13"
if [[ "$ARCH" == "aarch64" ]]; then
  PY_FILE="cpython-${PY_VER}+${PY_TAG}-aarch64-unknown-linux-gnu-install_only_stripped.tar.gz"
else
  PY_FILE="cpython-${PY_VER}+${PY_TAG}-x86_64-unknown-linux-gnu-install_only_stripped.tar.gz"
fi
PY_REPO="astral-sh/python-build-standalone"
PY_URL="https://github.com/${PY_REPO}/releases/download/${PY_TAG}/${PY_FILE}"
TOOL_NAME="appimagetool-${ARCH}.AppImage"
TOOL_URL="https://github.com/AppImage/appimagetool/releases/download/continuous/${TOOL_NAME}"

# GitHub 直连常超时：先官方，再 gh api，再镜像。
MIRRORS=(
  ""
  "https://ghfast.top/"
  "https://ghproxy.net/"
)
if [[ -n "${GITHUB_MIRROR:-}" ]]; then
  MIRRORS=("${GITHUB_MIRROR}" "${MIRRORS[@]}")
fi

download() {
  local url="$1" dest="$2"
  local tmp="${dest}.part"
  mkdir -p "$(dirname "$dest")"
  rm -f "$tmp"
  if curl -fL --retry 3 --connect-timeout 20 --speed-time 45 --speed-limit 1000 -o "$tmp" "$url"; then
    mv "$tmp" "$dest"
    return 0
  fi
  rm -f "$tmp"
  return 1
}

download_github() {
  local repo="$1" tag="$2" name="$3" dest="$4" url="$5"
  echo "downloading ${name}"
  if command -v gh >/dev/null 2>&1; then
    local id
    id="$(gh api "repos/${repo}/releases/tags/${tag}" --jq ".assets[] | select(.name==\"${name}\") | .id" 2>/dev/null | head -1 || true)"
    if [[ -n "${id}" ]]; then
      if gh api -H "Accept: application/octet-stream" "repos/${repo}/releases/assets/${id}" > "${dest}.part" 2>/dev/null; then
        if [[ -s "${dest}.part" ]]; then
          mv "${dest}.part" "$dest"
          return 0
        fi
      fi
      rm -f "${dest}.part"
    fi
  fi
  local prefix
  for prefix in "${MIRRORS[@]}"; do
    if download "${prefix}${url}" "$dest"; then
      return 0
    fi
  done
  return 1
}

mkdir -p "$DIST" "$CACHE"
rm -rf "$DIST/AppDir" "$DIST/CyberOcean-portable"
python3 "$ROOT/packaging/make_icon.py"
ICON="$ROOT/packaging/cyber-ocean.png"

echo "==> portable zip"
PORT="$DIST/CyberOcean-portable"
mkdir -p "$PORT/fucan"
cp -a "$ROOT/main.py" "$ROOT/LICENSE" "$ROOT/README.md" "$ROOT/requirements.txt" "$PORT/"
cp -a "$ROOT/fucan/"*.py "$PORT/fucan/"
CJK_FONT=""
for cand in \
  "$ROOT/fonts/DroidSansFallbackFull.ttf" \
  /usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf \
  /usr/share/fonts/truetype/droid/DroidSansFallback.ttf
do
  if [[ -f "$cand" ]]; then
    CJK_FONT="$cand"
    break
  fi
done
if [[ -n "$CJK_FONT" ]]; then
  mkdir -p "$PORT/fonts"
  cp -a "$CJK_FONT" "$PORT/fonts/DroidSansFallbackFull.ttf"
fi
python3 "$ROOT/main.py" --write-screensaver "$PORT/screensaver.html"
python3 "$ROOT/main.py" --write-screensaver "$PORT/wallpaper.html" --wallpaper
NATIVE_BIN="$ROOT/native/target/release/cyber-ocean-native"
if command -v cargo >/dev/null 2>&1; then
  echo "==> rust wgpu screensaver"
  export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/native/target}"
  ( cd "$ROOT/native" && CARGO_NET_OFFLINE=1 cargo build --release )
fi
if [[ -x "$NATIVE_BIN" ]]; then
  cp -a "$NATIVE_BIN" "$PORT/cyber-ocean-native"
fi
cat > "$PORT/run-screensaver.sh" <<'EOF'
#!/usr/bin/env bash
cd "$(dirname "$0")"
if [[ -x ./cyber-ocean-native ]]; then
  exec ./cyber-ocean-native --screensaver "$@"
fi
exec python3 main.py --screensaver "$@"
EOF
cat > "$PORT/run-gallery.sh" <<'EOF'
#!/usr/bin/env bash
cd "$(dirname "$0")"
exec python3 main.py "$@"
EOF
cat > "$PORT/run-screensaver.bat" <<'EOF'
@echo off
cd /d "%~dp0"
python main.py --screensaver %*
EOF
cat > "$PORT/run-gallery.bat" <<'EOF'
@echo off
cd /d "%~dp0"
python main.py %*
EOF
chmod +x "$PORT"/run-*.sh
cat > "$PORT/HOW-TO-RUN.txt" <<EOF
赛博海洋馆 / Cyber Ocean  ${VERSION}

Linux / macOS:
  ./run-screensaver.sh
  ./run-gallery.sh

Windows:
  run-screensaver.bat
  run-gallery.bat

需要本机已安装 Python 3.10+，以及 Chrome / Edge / Firefox（全屏）。
若目录里有 cyber-ocean-native，屏保会走 wgpu 原生窗口，不需要浏览器。
需要纯网页时，用浏览器打开 screensaver.html 或 wallpaper.html。
EOF
( cd "$DIST" && zip -qr "CyberOcean-portable-${VERSION}.zip" CyberOcean-portable )
cp "$PORT/screensaver.html" "$DIST/screensaver.html"
cp "$PORT/wallpaper.html" "$DIST/wallpaper.html"

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "AppImage 仅在 Linux 上构建。便携包已写出。"
  ls -lh "$DIST"
  exit 0
fi

APPDIR="$DIST/AppDir"
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/share/cyber-ocean/fucan" \
         "$APPDIR/usr/share/applications" \
         "$APPDIR/usr/share/icons/hicolor/256x256/apps" \
         "$APPDIR/usr/bin"

BUNDLE_PYTHON=1
if [[ "${SYSTEM_PYTHON:-0}" == "1" ]]; then
  BUNDLE_PYTHON=0
fi

if [[ "$BUNDLE_PYTHON" == "1" ]]; then
  echo "==> download portable CPython"
  PY_TGZ="$CACHE/$PY_FILE"
  if [[ -f "$PY_TGZ" ]] && gzip -t "$PY_TGZ" 2>/dev/null; then
    echo "using cached $PY_FILE"
  else
    rm -f "$PY_TGZ"
    download_github "$PY_REPO" "$PY_TAG" "$PY_FILE" "$PY_TGZ" "$PY_URL"
    gzip -t "$PY_TGZ"
  fi
  tar -xzf "$PY_TGZ" -C "$APPDIR/usr"
  if [[ -d "$APPDIR/usr/python" ]]; then
    :
  elif [[ -d "$APPDIR/usr/usr" ]]; then
    mv "$APPDIR/usr/usr" "$APPDIR/usr/python"
  else
    echo "CPython tarball layout unexpected" >&2
    ls -la "$APPDIR/usr" >&2
    exit 1
  fi
  ln -sfn ../python/bin/python3 "$APPDIR/usr/bin/python3"
else
  echo "==> SYSTEM_PYTHON=1，AppImage 将调用系统 python3"
fi

cp -a "$ROOT/main.py" "$ROOT/LICENSE" "$APPDIR/usr/share/cyber-ocean/"
cp -a "$ROOT/fucan/"*.py "$APPDIR/usr/share/cyber-ocean/fucan/"
if [[ -n "${CJK_FONT:-}" ]]; then
  mkdir -p "$APPDIR/usr/share/cyber-ocean/fonts"
  cp -a "$CJK_FONT" "$APPDIR/usr/share/cyber-ocean/fonts/DroidSansFallbackFull.ttf"
fi
install -m 0755 "$ROOT/packaging/AppRun" "$APPDIR/AppRun"
install -m 0644 "$ROOT/packaging/cyber-ocean.desktop" "$APPDIR/cyber-ocean.desktop"
install -m 0644 "$ROOT/packaging/cyber-ocean.desktop" "$APPDIR/usr/share/applications/cyber-ocean.desktop"
install -m 0644 "$ICON" "$APPDIR/cyber-ocean.png"
install -m 0644 "$ICON" "$APPDIR/.DirIcon"
install -m 0644 "$ICON" "$APPDIR/usr/share/icons/hicolor/256x256/apps/cyber-ocean.png"
cat > "$APPDIR/usr/bin/cyber-ocean" <<'EOF'
#!/bin/bash
HERE="$(dirname "$(readlink -f "$0")")/../.."
exec "$HERE/AppRun" "$@"
EOF
chmod +x "$APPDIR/usr/bin/cyber-ocean"
if [[ -x "$NATIVE_BIN" ]]; then
  install -m 0755 "$NATIVE_BIN" "$APPDIR/usr/bin/cyber-ocean-native"
fi

echo "==> appimagetool"
TOOL="$CACHE/$TOOL_NAME"
if [[ ! -x "$TOOL" ]]; then
  download_github "AppImage/appimagetool" "continuous" "$TOOL_NAME" "$TOOL" "$TOOL_URL"
  chmod +x "$TOOL"
fi
OUT_APP="$DIST/CyberOcean-${VERSION}-${ARCH}.AppImage"
RUNTIME="$CACHE/runtime-${ARCH}"
if [[ ! -s "$RUNTIME" ]]; then
  RUNTIME_URL="https://github.com/AppImage/type2-runtime/releases/download/continuous/runtime-${ARCH}"
  download_github "AppImage/type2-runtime" "continuous" "runtime-${ARCH}" "$RUNTIME" "$RUNTIME_URL"
fi
export ARCH
export APPIMAGE_EXTRACT_AND_RUN=1
"$TOOL" --appimage-extract-and-run --runtime-file "$RUNTIME" --no-appstream "$APPDIR" "$OUT_APP"
chmod +x "$OUT_APP"

( cd "$DIST" && sha256sum "CyberOcean-portable-${VERSION}.zip" "CyberOcean-${VERSION}-${ARCH}.AppImage" screensaver.html wallpaper.html > SHA256SUMS )

echo
echo "成品目录 $DIST"
ls -lh "$DIST"/*.AppImage "$DIST"/*.zip "$DIST"/*.html "$DIST"/SHA256SUMS
echo
echo "运行 AppImage："
echo "  $OUT_APP"
echo "  $OUT_APP --gallery"
echo "  $OUT_APP --wallpaper"
