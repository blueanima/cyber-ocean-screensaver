#!/usr/bin/env bash
# 把赛博海洋馆装到当前用户：命令、桌面项、可选 xscreensaver。
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${XDG_BIN_HOME:-$HOME/.local/bin}"
APP="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
PYTHON="${PYTHON:-python3}"

mkdir -p "$BIN" "$APP"

cat > "$BIN/cyber-ocean-screensaver" <<EOF
#!/usr/bin/env bash
exec "$PYTHON" "$ROOT/main.py" --screensaver "\$@"
EOF
chmod +x "$BIN/cyber-ocean-screensaver"

cat > "$BIN/cyber-ocean" <<EOF
#!/usr/bin/env bash
exec "$PYTHON" "$ROOT/main.py" "\$@"
EOF
chmod +x "$BIN/cyber-ocean"

cat > "$APP/cyber-ocean-screensaver.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=赛博海洋馆屏幕保护
Name[en]=Cyber Ocean Screensaver
Comment=公式生物在深海里游动
Exec=$BIN/cyber-ocean-screensaver
Terminal=false
Categories=Screensaver;Graphics;
StartupNotify=false
EOF

echo "已安装："
echo "  $BIN/cyber-ocean-screensaver"
echo "  $APP/cyber-ocean-screensaver.desktop"
echo
echo "运行： cyber-ocean-screensaver"
echo "集合馆： cyber-ocean"
echo
echo "空闲自动启动请加 --no-setup，避免弹出设置窗口："
echo "  hypridle / swayidle timeout 180 '$BIN/cyber-ocean-screensaver --no-setup'"
echo "  xfce4-session：设置 → 会话和启动 → 应用程序自动启动"
echo

if command -v xscreensaver-command >/dev/null 2>&1; then
  CONF="$HOME/.xscreensaver"
  HACK="$BIN/cyber-ocean-screensaver"
  if [[ -f "$CONF" ]] && ! grep -q "cyber-ocean-screensaver" "$CONF";  then
    echo "可把下面这一行加入 $CONF 的 programs 列表："
    echo "  cyber-ocean     $HACK \\n\\"
  fi
fi
