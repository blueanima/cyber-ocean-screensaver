#!/usr/bin/env bash
# 磷虾重训：每 EVAL_EVERY 秒看极化/分数，决定停或续。
# 不写回 LIFE，不加载旧 best.rs。
#   ./scripts/retrain-eval.sh          # 默认最多 2h，涨则最多续到 4h
#   ./scripts/retrain-eval.sh 2
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIR="$ROOT/.cache/life-obs"
JOURNAL="$DIR/journal.tsv"
LOG="$DIR/retrain-eval.log"
PIDFILE="$DIR/observe-retrain.pid"
HOURS_BLOCK="${1:-2}"
HOURS_MAX="${HOURS_MAX:-4}"
EVAL_EVERY="${EVAL_EVERY:-720}"
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
export OBSERVE_SIGMA="${OBSERVE_SIGMA:-0.38}"

mkdir -p "$DIR"
chmod +x "$ROOT/scripts/observe-loop.sh" "$ROOT/scripts/stay-awake.sh" "$ROOT/scripts/thermal-watch.sh"

log() { printf '%s %s\n' "$(date -Iseconds)" "$*" | tee -a "$LOG"; }

eval_journal() {
  python3 - "$JOURNAL" <<'PY'
import sys
from pathlib import Path
p = Path(sys.argv[1])
if not p.exists():
    print("wait nan nan nan nan 0")
    raise SystemExit
rows = []
for line in p.read_text().splitlines()[1:]:
    parts = line.split("\t")
    if len(parts) < 17:
        continue
    try:
        rows.append((float(parts[1]), float(parts[2]), float(parts[16]), float(parts[12])))
    except ValueError:
        continue
if len(rows) < 8:
    print("wait nan nan nan nan", len(rows))
    raise SystemExit
t, sc, pol, nnd = rows[-1]
start_sc, start_pol = rows[0][1], rows[0][2]
tail = [r[2] for r in rows[-80:]]
mid = [r[2] for r in rows[max(0, len(rows)//2 - 40): len(rows)//2 + 40]] or tail
tail.sort()
mid.sort()
pmed = tail[len(tail)//2]
pmid = mid[len(mid)//2]
champ = max(r[1] for r in rows)
print(f"go {t:.4f} {sc:.3f} {pmed:.4f} {nnd:.2f} {len(rows)} {start_sc:.3f} {start_pol:.4f} {champ:.3f} {pmid:.4f}")
PY
}

stop_all() {
  if [[ -f "$PIDFILE" ]]; then
    kill "$(cat "$PIDFILE")" 2>/dev/null || true
  fi
  pkill -f 'observe_record_optimize_loop' 2>/dev/null || true
  pkill -f 'observe-loop.sh' 2>/dev/null || true
  pkill -f 'scripts/thermal-watch.sh' 2>/dev/null || true
  sleep 2
  "$ROOT/scripts/stay-awake.sh" restore || true
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
  ) >> "$DIR/observe-retrain.out" 2>&1 &
  sleep 4
}

ATTACH="${ATTACH:-0}"
if [[ "$ATTACH" != "1" ]]; then
  if [[ -f "$JOURNAL" ]]; then
    mv -f "$JOURNAL" "$DIR/journal-pre-retrain-$(date +%Y%m%d-%H%M%S).tsv"
  fi
  if [[ -f "$DIR/best.rs" ]]; then
    mv -f "$DIR/best.rs" "$DIR/best.rs.pre-retrain-$(date +%H%M%S)"
  fi
  rm -f "$DIR/thermal-trip.txt"
  : > "$LOG"
  log "retrain start block=${HOURS_BLOCK}h max=${HOURS_MAX}h eval_every=${EVAL_EVERY}s load_best=0"
  start_thermal
  start_observe "$HOURS_BLOCK"
else
  log "attach existing observe; max=${HOURS_MAX}h eval_every=${EVAL_EVERY}s"
  if ! pgrep -f 'scripts/thermal-watch.sh' >/dev/null; then
    start_thermal
  fi
fi

t0=$(date +%s)
t_cap=$(( t0 + $(python3 -c "print(int(float('$HOURS_MAX')*3600))") ))
extended=0
prev_pmed=""
next_sleep="$EVAL_EVERY"
if [[ "$ATTACH" == "1" ]]; then
  next_sleep=5
fi

while [[ $(date +%s) -lt $t_cap ]]; do
  if ! pgrep -f 'observe_record_optimize_loop' >/dev/null; then
    read -r tag eh sc pmed nnd n start_sc start_pol champ pmid <<<"$(eval_journal)"
    log "observe exited t=${eh}h polar_med=${pmed} champ=${champ}"
    left=$(python3 -c "print(max(0.0, ($t_cap - $(date +%s))/3600.0))")
    if [[ "$tag" == "go" ]]; then
      more=$(python3 -c "print(int(float('${pmed}')>=0.70 and float('${pmed}')<0.76 and float('${left}')>=0.25))")
      if [[ "$more" == "1" ]]; then
        log "polar still climbing; restart remaining ${left}h"
        start_thermal
        start_observe "$left"
        continue
      fi
    fi
    break
  fi
  if [[ -f "$DIR/thermal-trip.txt" ]]; then
    left=$(python3 -c "print(max(0.08, ($t_cap - $(date +%s))/3600.0))")
    log "thermal trip; restart remaining ${left}h"
    rm -f "$DIR/thermal-trip.txt"
    pkill -f 'observe_record_optimize_loop' 2>/dev/null || true
    pkill -f 'observe-loop.sh' 2>/dev/null || true
    sleep 2
    start_thermal
    start_observe "$left"
  fi
  sleep "$next_sleep"
  next_sleep="$EVAL_EVERY"
  read -r tag eh sc pmed nnd n start_sc start_pol champ pmid <<<"$(eval_journal)"
  wall=$(python3 -c "print(f'{($t_cap - $(date +%s))/3600.0:.2f}')")
  log "eval tag=${tag} t=${eh}h wall_left=${wall}h n=${n} score=${sc} champ=${champ} polar_med=${pmed} polar_mid=${pmid} nnd=${nnd} start_pol=${start_pol}"
  if [[ "$tag" != "go" ]]; then
    continue
  fi
  decision=$(python3 -c "
eh=float('${eh}')
p=float('${pmed}')
pmid=float('${pmid}')
sc=float('${sc}')
champ=float('${champ}')
start_sc=float('${start_sc}')
start_p=float('${start_pol}')
prev='${prev_pmed}'
prev_p=float(prev) if prev not in ('', 'nan') else None
if p >= 0.76 and pmid >= 0.72:
    print('stop_ok')
elif eh >= 0.55 and p < 0.68 and p <= start_p + 0.02:
    print('stop_stuck')
elif eh >= 0.55 and champ < start_sc - 0.25 and p < start_p + 0.03:
    print('stop_score')
elif eh >= 1.60 and 0.70 <= p < 0.76 and (prev_p is None or p >= prev_p - 0.005):
    print('extend')
else:
    print('continue')
")
  log "decision=${decision}"
  prev_pmed="$pmed"
  case "$decision" in
    stop_ok)
      log "polar reached target band; stop"
      break
      ;;
    stop_stuck|stop_score)
      log "no further gain expected; stop"
      break
      ;;
    extend)
      if [[ $extended -eq 0 ]]; then
        extended=1
        left=$(python3 -c "print(max(0.4, min(1.2, ($t_cap - $(date +%s))/3600.0)))")
        log "polar climbing in 0.70-0.76; continue remaining ~${left}h"
      fi
      ;;
  esac
done

stop_all
log "retrain done"
echo "retrain-eval exit=0"
