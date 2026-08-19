#!/usr/bin/env bash
# 无头观察循环：分种 CMA-ES。默认磷虾专场，锁 space，只动 slip/zone。
# 用法：
#   ./scripts/observe-loop.sh 8
#   OBSERVE_CI=14 OBSERVE_LOCK_SPACE=1 OBSERVE_ALIGN_ONLY=1 ./scripts/observe-loop.sh 8
#   OBSERVE_CI=loose OBSERVE_ALIGN_ONLY=1 ./scripts/observe-loop.sh 0.5
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export OBSERVE_HOURS="${1:-${OBSERVE_HOURS:-24}}"
export OBSERVE_GENS="${OBSERVE_GENS:-12}"
export OBSERVE_CI="${OBSERVE_CI:-14}"
export OBSERVE_LOCK_SPACE="${OBSERVE_LOCK_SPACE:-1}"
export OBSERVE_ALIGN_ONLY="${OBSERVE_ALIGN_ONLY:-1}"
export OBSERVE_EVAL="${OBSERVE_EVAL:-20}"
export OBSERVE_LOAD_BEST="${OBSERVE_LOAD_BEST:-0}"
export OBSERVE_SIGMA="${OBSERVE_SIGMA:-0.38}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/native/target}"
export CARGO_NET_OFFLINE="${CARGO_NET_OFFLINE:-1}"
export PATH="${HOME}/.cargo/bin:${PATH}"
mkdir -p "$ROOT/.cache/life-obs"
echo "observe-loop hours=${OBSERVE_HOURS} gens=${OBSERVE_GENS} ci=${OBSERVE_CI} lock_space=${OBSERVE_LOCK_SPACE} align_only=${OBSERVE_ALIGN_ONLY} eval=${OBSERVE_EVAL}s sigma=${OBSERVE_SIGMA} load_best=${OBSERVE_LOAD_BEST}"
echo "logs: $ROOT/.cache/life-obs/journal.tsv"
cd "$ROOT/native"
cmd=(cargo test --offline --bin cyber-ocean-native -- --ignored --nocapture observe_record_optimize_loop)
chmod +x "$ROOT/scripts/stay-awake.sh"
"$ROOT/scripts/stay-awake.sh" apply
restore_idle() { "$ROOT/scripts/stay-awake.sh" restore || true; }
trap restore_idle EXIT
if command -v systemd-inhibit >/dev/null 2>&1; then
  echo "inhibit: logind idle/sleep/lid + cosmic-idle suspend=None"
  systemd-inhibit \
    --what=idle:sleep:handle-lid-switch \
    --who=cyber-ocean-observe \
    --why="observe-loop ${OBSERVE_HOURS}h ci=${OBSERVE_CI}" \
    --mode=block \
    -- "${cmd[@]}"
else
  echo "warn: systemd-inhibit missing; relying on cosmic-idle None" >&2
  "${cmd[@]}"
fi
