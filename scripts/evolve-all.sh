#!/usr/bin/env bash
# 全种自我进化：17 种轮转，锁 space，动全部运动参数（含 zone）。
# 定时评估；走平则加更激进 CMA 步长并续跑剩余时间。不写回 LIFE。
# 训练进程用 setsid 独立会话，加码时只杀该会话，避免评估包装被一起带走。
#   ./scripts/evolve-all.sh 4
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIR="$ROOT/.cache/life-obs"
JOURNAL="$DIR/journal.tsv"
LOG="$DIR/evolve-all.log"
PIDFILE="$DIR/observe-evolve.pid"
HOURS_TOTAL="${1:-4}"
EVAL_EVERY="${EVAL_EVERY:-900}"
LIMIT_C="${LIMIT_C:-95}"
HOLD_S="${HOLD_S:-30}"
export PATH="${HOME}/.cargo/bin:${PATH}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/native/target}"
export CARGO_NET_OFFLINE="${CARGO_NET_OFFLINE:-1}"
export OBSERVE_CI="${OBSERVE_CI:-all}"
export OBSERVE_LOCK_SPACE=1
export OBSERVE_ALIGN_ONLY=0
export OBSERVE_LOAD_BEST=0
export OBSERVE_EVAL="${OBSERVE_EVAL:-16}"
export OBSERVE_GENS="${OBSERVE_GENS:-12}"
export OBSERVE_SIGMA="${OBSERVE_SIGMA:-0.38}"

mkdir -p "$DIR"
chmod +x "$ROOT/scripts/observe-loop.sh" "$ROOT/scripts/stay-awake.sh" "$ROOT/scripts/thermal-watch.sh"

log() { printf '%s %s\n' "$(date -Iseconds)" "$*" | tee -a "$LOG"; }

write_ctrl() {
  printf 'sigma=%s\ngens=%s\n' "$OBSERVE_SIGMA" "$OBSERVE_GENS" > "$DIR/cma-ctrl.txt"
}

eval_journal() {
  python3 - "$JOURNAL" <<'PY'
import sys
from collections import defaultdict
from pathlib import Path
p = Path(sys.argv[1])
if not p.exists():
    print("wait 0 nan nan nan 0 0 0")
    raise SystemExit
rows = []
by_sp = defaultdict(list)
for line in p.read_text().splitlines()[1:]:
    parts = line.split("\t")
    if len(parts) < 17:
        continue
    try:
        t = float(parts[1])
        mix = float(parts[2])
        spec = parts[14]
        spec_sc = float(parts[15])
        polar = float(parts[16])
    except ValueError:
        continue
    rows.append((t, mix, spec, spec_sc, polar))
    by_sp[spec].append((t, spec_sc, polar))
if len(rows) < 24:
    print("wait", len(rows), "nan nan nan 0 0 0")
    raise SystemExit
t, mix, _, _, _ = rows[-1]
champ = max(r[1] for r in rows)
start_mix = rows[0][1]
recent = [r[1] for r in rows if r[0] >= t - 0.25] or [mix]
prev = [r[1] for r in rows if t - 0.55 <= r[0] < t - 0.25] or recent
rmax = max(recent)
pmax = max(prev)
n_sp = len(by_sp)
risers = 0
for sp, xs in by_sp.items():
    if len(xs) < 4:
        continue
    early = sorted(x[1] for x in xs[: max(2, len(xs)//4)])
    late = sorted(x[1] for x in xs[-8:])
    if late[len(late)//2] > early[len(early)//2] + 0.04:
        risers += 1
print(f"go {t:.4f} {mix:.3f} {champ:.3f} {start_mix:.3f} {rmax:.3f} {pmax:.3f} {n_sp} {risers} {len(rows)}")
PY
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
  "$ROOT/scripts/stay-awake.sh" restore || true
}

start_thermal() {
  if [[ -f "$DIR/thermal-watch.pid" ]]; then
    kill "$(cat "$DIR/thermal-watch.pid")" 2>/dev/null || true
  fi
  export OBSERVE_PIDFILE="$PIDFILE"
  export LIMIT_C HOLD_S
  # 独立会话，避免热跳闸时误伤评估包装。
  setsid "$ROOT/scripts/thermal-watch.sh" >> "$DIR/thermal-watch.out" 2>&1 &
  disown $! 2>/dev/null || true
}

start_observe() {
  local hours="$1"
  # 新会话启动，加码杀训练时不会把评估循环一起带走。
  setsid "$ROOT/scripts/observe-loop.sh" "$hours" >> "$DIR/observe-evolve.out" 2>&1 &
  echo $! > "$PIDFILE"
  disown $! 2>/dev/null || true
  sleep 5
}

if [[ -f "$JOURNAL" ]]; then
  mv -f "$JOURNAL" "$DIR/journal-pre-evolve-$(date +%Y%m%d-%H%M%S).tsv"
fi
if [[ -f "$DIR/best.rs" ]]; then
  mv -f "$DIR/best.rs" "$DIR/best.rs.pre-evolve-$(date +%H%M%S)"
fi
rm -f "$DIR/thermal-trip.txt"
: > "$LOG"
log "evolve-all start hours=${HOURS_TOTAL} ci=${OBSERVE_CI} lock_space=1 align_only=0 sigma=${OBSERVE_SIGMA} gens=${OBSERVE_GENS}"
write_ctrl
start_thermal
start_observe "$HOURS_TOTAL"

t0=$(date +%s)
t_cap=$(( t0 + $(python3 -c "print(int(float('$HOURS_TOTAL')*3600))") ))
boost=0
last_boost=0
prev_champ=""

while [[ $(date +%s) -lt $t_cap ]]; do
  if ! observe_alive; then
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
    left=$(python3 -c "print(max(0.08, ($t_cap - $(date +%s))/3600.0))")
    log "thermal trip; restart remaining ${left}h"
    rm -f "$DIR/thermal-trip.txt"
    kill_observe
    export OBSERVE_LOAD_BEST=1
    start_thermal
    start_observe "$left"
  fi
  sleep "$EVAL_EVERY"
  read -r tag eh mix champ start_mix rmax pmax n_sp risers n <<<"$(eval_journal)"
  wall=$(python3 -c "print(f'{($t_cap - $(date +%s))/3600.0:.2f}')")
  log "eval tag=${tag} t=${eh}h wall_left=${wall}h n=${n} mix=${mix} champ=${champ} start=${start_mix} recent_max=${rmax} prev_max=${pmax} species=${n_sp} risers=${risers} sigma=${OBSERVE_SIGMA} gens=${OBSERVE_GENS} boost=${boost}"
  if [[ "$tag" != "go" ]]; then
    continue
  fi
  now=$(date +%s)
  decision=$(python3 -c "
eh=float('${eh}')
champ=float('${champ}')
rmax=float('${rmax}')
pmax=float('${pmax}')
risers=int('${risers}')
boost=int('${boost}')
prev='${prev_champ}'
prev_c=float(prev) if prev not in ('', 'nan') else None
since_boost=($now - $last_boost)/3600.0 if $last_boost else 99
if eh < 0.40:
    print('continue')
elif boost < 3 and since_boost >= 0.40 and rmax <= pmax + 0.03 and (prev_c is None or champ <= prev_c + 0.02) and risers <= 3:
    print('boost')
else:
    print('continue')
")
  log "decision=${decision}"
  prev_champ="$champ"
  if [[ "$decision" == "boost" ]]; then
    boost=$((boost + 1))
    last_boost=$now
    prev_sigma="$OBSERVE_SIGMA"
    case "$boost" in
      1)
        if python3 -c "raise SystemExit(0 if float('${prev_sigma}') >= 0.50 else 1)"; then
          export OBSERVE_SIGMA=0.68 OBSERVE_GENS=16
        else
          export OBSERVE_SIGMA=0.52 OBSERVE_GENS=16
        fi
        ;;
      2) export OBSERVE_SIGMA=0.68 OBSERVE_GENS=16 ;;
      *) export OBSERVE_SIGMA=0.72 OBSERVE_GENS=18 ;;
    esac
    log "stuck; live-correct boost=${boost} sigma=${OBSERVE_SIGMA} gens=${OBSERVE_GENS} (no restart)"
    write_ctrl
  fi
done

stop_all
log "evolve-all done boost=${boost} sigma=${OBSERVE_SIGMA}"
echo "evolve-all exit=0"
