#!/usr/bin/env bash
# 8 小时磷虾场：极化卡住则加强 slip/zone 并重启剩余时间。
# 不写回 LIFE。不加载压扁 yaw 的 best.rs。
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIR="$ROOT/.cache/life-obs"
JOURNAL="$DIR/journal.tsv"
LOG="$DIR/polar-advance.log"
PIDFILE="$DIR/observe-shrimp8.pid"
ADVANCE_MARK="$DIR/polar-advance.done"
HOURS_TOTAL="${1:-8}"
LIMIT_C="${LIMIT_C:-95}"
HOLD_S="${HOLD_S:-30}"
export PATH="${HOME}/.cargo/bin:${PATH}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/native/target}"
export CARGO_NET_OFFLINE="${CARGO_NET_OFFLINE:-1}"
export OBSERVE_CI=14
export OBSERVE_LOCK_SPACE=1
export OBSERVE_ALIGN_ONLY=1
export OBSERVE_LOAD_BEST=0
export OBSERVE_EVAL="${OBSERVE_EVAL:-20}"
export OBSERVE_GENS="${OBSERVE_GENS:-12}"

mkdir -p "$DIR"
chmod +x "$ROOT/scripts/observe-loop.sh" "$ROOT/scripts/stay-awake.sh" "$ROOT/scripts/thermal-watch.sh"

log() { printf '%s %s\n' "$(date -Iseconds)" "$*" | tee -a "$LOG"; }

median_polar() {
  python3 - "$JOURNAL" <<'PY'
import sys
from pathlib import Path
p = Path(sys.argv[1])
if not p.exists():
    print("nan"); raise SystemExit
rows = []
for line in p.read_text().splitlines()[1:]:
    parts = line.split("\t")
    if len(parts) < 17:
        continue
    try:
        rows.append((float(parts[1]), float(parts[16])))
    except ValueError:
        continue
if len(rows) < 12:
    print("nan"); raise SystemExit
tail = [r[1] for r in rows[-80:]]
tail.sort()
print(f"{tail[len(tail)//2]:.4f}")
PY
}

elapsed_h() {
  python3 - "$JOURNAL" <<'PY'
import sys
from pathlib import Path
p = Path(sys.argv[1])
if not p.exists():
    print("0"); raise SystemExit
last = None
for line in p.read_text().splitlines()[1:]:
    parts = line.split("\t")
    if len(parts) < 2:
        continue
    try:
        last = float(parts[1])
    except ValueError:
        continue
print(f"{last or 0:.4f}")
PY
}

stop_observe() {
  if [[ -f "$PIDFILE" ]]; then
    kill "$(cat "$PIDFILE")" 2>/dev/null || true
  fi
  pkill -f 'observe_record_optimize_loop' 2>/dev/null || true
  pkill -f 'observe-loop.sh' 2>/dev/null || true
  sleep 2
}

start_thermal() {
  pkill -f 'scripts/thermal-watch.sh' 2>/dev/null || true
  export OBSERVE_PIDFILE="$PIDFILE"
  export LIMIT_C HOLD_S
  nohup "$ROOT/scripts/thermal-watch.sh" >> "$DIR/thermal-watch.out" 2>&1 &
}

start_observe() {
  local hours="$1"
  (
    echo $$ > "$PIDFILE"
    cd "$ROOT"
    exec ./scripts/observe-loop.sh "$hours"
  ) >> "$DIR/observe-shrimp8.out" 2>&1 &
  sleep 3
}

t_end=$(( $(date +%s) + $(python3 -c "print(int(float('$HOURS_TOTAL')*3600))") ))
if [[ -f "$JOURNAL" ]]; then
  mv -f "$JOURNAL" "$DIR/journal-shrimp-prev-$(date +%Y%m%d-%H%M%S).tsv"
fi
if [[ -f "$DIR/best.rs" ]]; then
  mv -f "$DIR/best.rs" "$DIR/best.rs.pre-shrimp8-$(date +%H%M%S)"
fi
rm -f "$DIR/thermal-trip.txt" "$ADVANCE_MARK"
log "start shrimp8 hours=${HOURS_TOTAL} slip=${OBSERVE_SLIP:-LIFE} zone=${OBSERVE_ZONE:-LIFE}"
start_thermal
start_observe "$HOURS_TOTAL"

advanced=0
while [[ $(date +%s) -lt $t_end ]]; do
  if [[ -f "$DIR/thermal-trip.txt" ]]; then
    left=$(python3 -c "print(max(0.05, ($t_end - $(date +%s))/3600.0))")
    log "thermal trip; restart remaining ${left}h"
    rm -f "$DIR/thermal-trip.txt"
    stop_observe
    start_thermal
    start_observe "$left"
  fi
  if [[ $advanced -eq 0 ]] && [[ ! -f "$ADVANCE_MARK" ]]; then
    eh=$(elapsed_h)
    pol=$(median_polar)
    log "watch t=${eh}h polar_med=${pol}"
    ok=$(python3 -c "print(int(float('${eh:-0}')>=0.50 and float('${pol:-1}')==float('${pol:-1}') and float('${pol:-1}')<0.70))" 2>/dev/null || echo 0)
    if [[ "$ok" == "1" ]]; then
      left=$(python3 -c "print(max(0.08, ($t_end - $(date +%s))/3600.0))")
      log "polar stuck ${pol}; boost slip=1.85 zone=2.50 remaining ${left}h"
      export OBSERVE_SLIP=1.85
      export OBSERVE_ZONE=2.50
      date -Iseconds > "$ADVANCE_MARK"
      advanced=1
      if [[ -f "$JOURNAL" ]]; then
        mv -f "$JOURNAL" "$DIR/journal-shrimp-preboost-$(date +%H%M%S).tsv"
      fi
      stop_observe
      start_observe "$left"
    fi
  fi
  sleep 120
done
stop_observe
pkill -f 'scripts/thermal-watch.sh' 2>/dev/null || true
"$ROOT/scripts/stay-awake.sh" restore || true
log "shrimp8 wall-clock done"
