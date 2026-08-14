#!/usr/bin/env bash
# 安装 macOS 应用程序：~/Applications/CyberOcean Screensaver.app
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEST="${HOME}/Applications/CyberOcean Screensaver.app"
MACOS="$DEST/Contents/MacOS"
PYTHON="${PYTHON:-/usr/bin/python3}"

mkdir -p "$MACOS" "$DEST/Contents/Resources"

cat > "$DEST/Contents/Info.plist" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>CyberOcean Screensaver</string>
  <key>CFBundleDisplayName</key><string>赛博海洋馆</string>
  <key>CFBundleIdentifier</key><string>local.cyber.ocean.screensaver</string>
  <key>CFBundleVersion</key><string>1.0</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleExecutable</key><string>CyberOcean</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
EOF

cat > "$MACOS/CyberOcean" <<EOF
#!/bin/bash
ROOT="$ROOT"
PYTHON="$PYTHON"
if ! command -v "\$PYTHON" >/dev/null 2>&1; then
  PYTHON="python3"
fi
exec "\$PYTHON" "\$ROOT/main.py" --screensaver
EOF
chmod +x "$MACOS/CyberOcean"

echo "已安装：$DEST"
echo
echo "打开屏保："
echo "  open \"$DEST\""
echo
echo "也可在「系统设置 → 桌面与程序坞 → 快捷键 / 触发角」里"
echo "用 Automator 把本应用设为触发角操作。"
echo
echo "需要系统级 .saver 模块时，请先安装 Chrome 或 Edge，再运行本应用（全屏 kiosk）。"
echo "壁纸模式（不退出）： python3 \"$ROOT/main.py\" --wallpaper"
