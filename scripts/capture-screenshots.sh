#!/usr/bin/env bash
# 生成 README 用截图（需要 firefox）。
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/docs/screenshots"
PORT="${PORT:-8766}"
mkdir -p "$OUT"

python3 "$ROOT/main.py" --no-browser --port "$PORT" &
PID=$!
PROFILE="$(mktemp -d /tmp/cyber-ocean-ff.XXXXXX)"
cleanup() {
  kill "$PID" 2>/dev/null || true
  rm -rf "$PROFILE"
}
trap cleanup EXIT

for i in 1 2 3 4 5 6 7 8 9 10; do
  curl -fsS "http://127.0.0.1:$PORT/api/health" >/dev/null && break
  sleep 0.3
done

FF="${FIREFOX:-firefox}"
timeout 30 "$FF" --headless --profile "$PROFILE" --window-size 1600,900 \
  --screenshot "$OUT/ocean.png" \
  "http://127.0.0.1:$PORT/screensaver?wallpaper=1&seed=42&shot=1" || true

timeout 30 "$FF" --headless --profile "$PROFILE" --window-size 1600,900 \
  --screenshot "$OUT/gallery.png" \
  "http://127.0.0.1:$PORT/?seed=42&shot=1" || true

ls -la "$OUT"
