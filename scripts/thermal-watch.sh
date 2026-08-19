#!/usr/bin/env bash
# 看 CPU 温度：连续 LIMIT 度以上 HOLD 秒才停观察循环。
# 用法：
#   ./scripts/thermal-watch.sh
#   OBSERVE_PIDFILE=... LIMIT_C=95 HOLD_S=30 ./scripts/thermal-watch.sh
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIR="$ROOT/.cache/life-obs"
mkdir -p "$DIR"
LOG="$DIR/thermal.log"
TRIP="$DIR/thermal-trip.txt"
PIDFILE="$DIR/thermal-watch.pid"
OBS_PIDFILE="${OBSERVE_PIDFILE:-$DIR/observe-1h.pid}"
LIMIT_C="${LIMIT_C:-95}"
HOLD_S="${HOLD_S:-30}"
LIMIT_MC=$((LIMIT_C * 1000))

echo $$ > "$PIDFILE"
echo "thermal-watch start limit=${LIMIT_C}C hold=${HOLD_S}s pid=$$" | tee -a "$LOG"

cpu_max_mc() {
  local t max=0
  for f in \
    /sys/class/thermal/thermal_zone8/temp \
    /sys/class/thermal/thermal_zone9/temp \
    /sys/class/hwmon/hwmon7/temp*_input \
    /sys/class/hwmon/hwmon6/temp1_input
  do
    [ -r "$f" ] || continue
    t=$(cat "$f" 2>/dev/null || echo 0)
    t=${t:-0}
    if [ "$t" -gt "$max" ] 2>/dev/null; then max=$t; fi
  done
  echo "$max"
}

hot_s=0
while true; do
  mc=$(cpu_max_mc)
  c=$((mc / 1000))
  ts=$(date -Iseconds)
  if [ "$mc" -ge "$LIMIT_MC" ]; then
    hot_s=$((hot_s + 1))
  else
    hot_s=0
  fi
  echo "$ts ${c}C hold=${hot_s}/${HOLD_S}s" >> "$LOG"
  echo "$ts,$c,$hot_s" >> "$DIR/thermal.csv"
  if [ "$hot_s" -ge "$HOLD_S" ]; then
    echo "THERMAL_TRIP_CPU {\"celsius\":$c,\"seconds\":$hot_s,\"limit\":$LIMIT_C}" | tee -a "$LOG"
    echo "$ts TRIP ${c}C for ${hot_s}s — stopping observe-loop" | tee "$TRIP"
    obs=$(cat "$OBS_PIDFILE" 2>/dev/null || true)
    if [[ -n "${obs:-}" ]]; then
      kill "$obs" 2>/dev/null || true
      sleep 1
      kill -9 "$obs" 2>/dev/null || true
    fi
    exit 2
  fi
  sleep 1
done
