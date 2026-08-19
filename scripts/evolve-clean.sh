#!/usr/bin/env bash
# 短场干净进化：先磷虾对齐（极化门锁），再松散种压过极化。不写回 LIFE。
# 用法：
#   ./scripts/evolve-clean.sh          # 默认 1h：0.4h 磷虾 + 0.6h 松散
#   ./scripts/evolve-clean.sh 1.5
#   TRACK=shrimp ./scripts/evolve-clean.sh 0.5
#   TRACK=loose  ./scripts/evolve-clean.sh 0.5
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIR="$ROOT/.cache/life-obs"
JOURNAL="$DIR/journal.tsv"
LOG="$DIR/evolve-clean.log"
PIDFILE="$DIR/observe-clean.pid"
HOURS_TOTAL="${1:-1}"
TRACK="${TRACK:-seq}"
EVAL_EVERY="${EVAL_EVERY:-360}"
LIMIT_C="${LIMIT_C:-95}"
HOLD_S="${HOLD_S:-30}"
export PATH="${HOME}/.cargo/bin:${PATH}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/native/target}"
export CARGO_NET_OFFLINE="${CARGO_NET_OFFLINE:-1}"
export OBSERVE_LOCK_SPACE=1
export OBSERVE_ALIGN_ONLY=1
export OBSERVE_LOAD_BEST=0
export OBSERVE_EVAL="${OBSERVE_EVAL:-16}"
export OBSERVE_GENS="${OBSERVE_GENS:-10}"
export OBSERVE_SIGMA="${OBSERVE_SIGMA:-0.28}"

mkdir -p "$DIR"
chmod +x "$ROOT/scripts/observe-loop.sh" "$ROOT/scripts/stay-awake.sh" "$ROOT/scripts/thermal-watch.sh"

log() { printf '%s %s\n' "$(date -Iseconds)" "$*" | tee -a "$LOG"; }

write_ctrl() {
  printf 'sigma=%s\ngens=%s\n' "$OBSERVE_SIGMA" "$OBSERVE_GENS" > "$DIR/cma-ctrl.txt"
}

kill_tree() {
  local pid="${1:-}"
  [[ -n "$pid" && "$pid" =~ ^[0-9]+$ ]] || return 0
  local c
  for c in $(pgrep -P "$pid" 2>/dev/null || true); do
    kill_tree "$c"
  done
  kill -TERM "$pid" 2>/dev/null || true
}

kill_observe() {
  set +e
  local opid=""
  if [[ -f "$PIDFILE" ]]; then
    opid="$(tr -d ' \n' < "$PIDFILE")"
  fi
  kill_tree "$opid"
  sleep 1
  kill_tree "$opid"
  sleep 2
  set -e
}

observe_alive() {
  local p
  p="$(tr -d ' \n' < "$PIDFILE" 2>/dev/null || true)"
  [[ -n "$p" ]] && kill -0 "$p" 2>/dev/null
}

stop_all() {
  kill_observe
  if [[ -f "$DIR/thermal-watch.pid" ]]; then
    kill "$(cat "$DIR/thermal-watch.pid")" 2>/dev/null || true
  fi
  sleep 1
}

start_thermal() {
  if [[ -f "$DIR/thermal-watch.pid" ]]; then
    kill "$(cat "$DIR/thermal-watch.pid")" 2>/dev/null || true
  fi
  export OBSERVE_PIDFILE="$PIDFILE"
  export LIMIT_C HOLD_S
  setsid "$ROOT/scripts/thermal-watch.sh" >> "$DIR/thermal-watch.out" 2>&1 &
  disown $! 2>/dev/null || true
}

start_observe() {
  local hours="$1"
  setsid "$ROOT/scripts/observe-loop.sh" "$hours" >> "$DIR/observe-clean.out" 2>&1 &
  echo $! > "$PIDFILE"
  disown $! 2>/dev/null || true
  sleep 5
}

archive_leg() {
  local tag="$1"
  local ts
  ts="$(date +%Y%m%d-%H%M%S)"
  if [[ -f "$JOURNAL" ]]; then
    mv -f "$JOURNAL" "$DIR/journal-${tag}-${ts}.tsv"
  fi
  if [[ -f "$DIR/best.rs" ]]; then
    cp -f "$DIR/best.rs" "$DIR/best-${tag}.rs"
  fi
}

run_leg() {
  local name="$1"
  local hours="$2"
  local ci="$3"
  export OBSERVE_CI="$ci"
  export OBSERVE_LOAD_BEST=0
  write_ctrl
  rm -f "$DIR/thermal-trip.txt"
  log "leg start name=${name} hours=${hours} ci=${ci} sigma=${OBSERVE_SIGMA} gens=${OBSERVE_GENS}"
  start_thermal
  start_observe "$hours"
  local t_cap
  t_cap=$(( $(date +%s) + $(python3 -c "print(int(float('$hours')*3600))") ))
  while [[ $(date +%s) -lt $t_cap ]]; do
    if ! observe_alive; then
      local left
      left=$(python3 -c "print(max(0.0, ($t_cap - $(date +%s))/3600.0))")
      log "observe exited; remaining ${left}h"
      if python3 -c "raise SystemExit(0 if float('${left}')>=0.08 else 1)"; then
        export OBSERVE_LOAD_BEST=1
        start_thermal
        start_observe "$left"
        continue
      fi
      break
    fi
    if [[ -f "$DIR/thermal-trip.txt" ]]; then
      local left
      left=$(python3 -c "print(max(0.08, ($t_cap - $(date +%s))/3600.0))")
      log "thermal trip; restart remaining ${left}h"
      rm -f "$DIR/thermal-trip.txt"
      kill_observe
      export OBSERVE_LOAD_BEST=1
      start_thermal
      start_observe "$left"
    fi
    sleep "$EVAL_EVERY"
    local n polar
    n=$(python3 - "$JOURNAL" <<'PY'
from pathlib import Path
import sys
p = Path(sys.argv[1])
if not p.exists():
    print("0 nan"); raise SystemExit
rows = []
for line in p.read_text().splitlines()[1:]:
    parts = line.split("\t")
    if len(parts) < 17:
        continue
    try:
        rows.append(float(parts[16]))
    except ValueError:
        continue
if not rows:
    print("0 nan"); raise SystemExit
tail = sorted(rows[-24:])
print(f"{len(rows)} {tail[len(tail)//2]:.3f}")
PY
)
    log "watch name=${name} ${n}"
  done
  kill_observe
  archive_leg "$name"
  log "leg done name=${name}"
}

if [[ -f "$JOURNAL" ]]; then
  mv -f "$JOURNAL" "$DIR/journal-pre-clean-$(date +%Y%m%d-%H%M%S).tsv"
fi
if [[ -f "$DIR/best.rs" ]]; then
  mv -f "$DIR/best.rs" "$DIR/best.rs.pre-clean-$(date +%H%M%S)"
fi
: > "$LOG"
chmod +x "$ROOT/scripts/stay-awake.sh"
# observe-loop.sh 自己 apply/restore；包装退出时再 restore 一次。
trap '"$ROOT/scripts/stay-awake.sh" restore || true; stop_all' EXIT

log "evolve-clean start hours=${HOURS_TOTAL} track=${TRACK}"
case "$TRACK" in
  shrimp)
    run_leg shrimp "$HOURS_TOTAL" shrimp
    ;;
  loose)
    run_leg loose "$HOURS_TOTAL" loose
    ;;
  seq|*)
    shrimp_h=$(python3 -c "print(max(0.25, float('$HOURS_TOTAL')*0.40))")
    loose_h=$(python3 -c "print(max(0.30, float('$HOURS_TOTAL')*0.60))")
    run_leg shrimp "$shrimp_h" shrimp
    run_leg loose "$loose_h" loose
    ;;
esac

stop_all
"$ROOT/scripts/stay-awake.sh" restore || true
trap - EXIT
log "evolve-clean done"
echo "evolve-clean exit=0"
