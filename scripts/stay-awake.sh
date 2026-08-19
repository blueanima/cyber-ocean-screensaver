#!/usr/bin/env bash
# 挡住 COSMIC / GNOME 自动休眠。
# systemd-inhibit 挡不住 cosmic-idle：它直接执行 `systemctl suspend`。
# 用法：
#   scripts/stay-awake.sh apply     关掉自动挂起（备份原设置）
#   scripts/stay-awake.sh restore   恢复备份
#   scripts/stay-awake.sh status    看当前是否挡住
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BAK="$ROOT/.cache/life-obs/idle-backup"
COSMIC_IDLE="$HOME/.config/cosmic/com.system76.CosmicIdle/v1"
GSET_KEY="org.gnome.settings-daemon.plugins.power"

apply() {
  mkdir -p "$BAK" "$COSMIC_IDLE"
  for key in suspend_on_ac_time suspend_on_battery_time; do
    if [[ ! -f "$BAK/$key" && ! -f "$BAK/$key.absent" ]]; then
      if [[ -f "$COSMIC_IDLE/$key" ]]; then
        cp -a "$COSMIC_IDLE/$key" "$BAK/$key"
      else
        printf 'ABSENT\n' > "$BAK/$key.absent"
      fi
    fi
    printf 'None\n' > "$COSMIC_IDLE/$key"
  done
  if command -v gsettings >/dev/null 2>&1; then
    if [[ ! -f "$BAK/gsettings-ac-type" ]]; then
      gsettings get "$GSET_KEY" sleep-inactive-ac-type > "$BAK/gsettings-ac-type" || true
      gsettings get "$GSET_KEY" sleep-inactive-battery-type > "$BAK/gsettings-bat-type" || true
    fi
    gsettings set "$GSET_KEY" sleep-inactive-ac-type nothing || true
    gsettings set "$GSET_KEY" sleep-inactive-battery-type nothing || true
  fi
  printf 'stay-awake applied (cosmic-idle suspend=None, gsettings sleep=nothing)\n'
}

restore() {
  [[ -d "$BAK" ]] || { echo "stay-awake: no backup"; return 0; }
  mkdir -p "$COSMIC_IDLE"
  for key in suspend_on_ac_time suspend_on_battery_time; do
    if [[ -f "$BAK/$key.absent" ]]; then
      rm -f "$COSMIC_IDLE/$key"
    elif [[ -f "$BAK/$key" ]]; then
      cp -a "$BAK/$key" "$COSMIC_IDLE/$key"
    fi
  done
  if command -v gsettings >/dev/null 2>&1; then
    if [[ -f "$BAK/gsettings-ac-type" ]]; then
      gsettings set "$GSET_KEY" sleep-inactive-ac-type "$(tr -d "'" < "$BAK/gsettings-ac-type")" || true
    fi
    if [[ -f "$BAK/gsettings-bat-type" ]]; then
      gsettings set "$GSET_KEY" sleep-inactive-battery-type "$(tr -d "'" < "$BAK/gsettings-bat-type")" || true
    fi
  fi
  printf 'stay-awake restored\n'
}

status() {
  echo "cosmic-idle: $(pgrep -a cosmic-idle | grep -v pgrep || echo none)"
  echo "suspend_on_ac_time: $(cat "$COSMIC_IDLE/suspend_on_ac_time" 2>/dev/null || echo '(default, typically ~30min)')"
  echo "suspend_on_battery_time: $(cat "$COSMIC_IDLE/suspend_on_battery_time" 2>/dev/null || echo '(default)')"
  if command -v gsettings >/dev/null 2>&1; then
    echo "gsettings ac: $(gsettings get $GSET_KEY sleep-inactive-ac-type) / $(gsettings get $GSET_KEY sleep-inactive-ac-timeout)s"
  fi
  echo "--- systemd-inhibit --list ---"
  systemd-inhibit --list 2>/dev/null || echo "(no systemd)"
}

case "${1:-status}" in
  apply) apply ;;
  restore) restore ;;
  status) status ;;
  *) echo "usage: $0 apply|restore|status" >&2; exit 2 ;;
esac
