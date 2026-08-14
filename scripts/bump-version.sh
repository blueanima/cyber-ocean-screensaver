#!/usr/bin/env bash
# 解析当前版本；若该版本的 git 标签已存在，则自动 +0.0.1。
# 用法：
#   scripts/bump-version.sh           打印并写回 VERSION / Cargo.toml
#   scripts/bump-version.sh always    强制再升一档补丁号
#   scripts/bump-version.sh dry       只打印，不写文件
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FILE="$ROOT/VERSION"
CARGO="$ROOT/native/Cargo.toml"
MODE="${1:-auto}"

cur=""
if [[ -f "$FILE" ]]; then
  cur="$(tr -d ' \t\n' < "$FILE")"
fi
cur="${cur#v}"
if [[ ! "$cur" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  if [[ -f "$CARGO" ]]; then
    cur="$(sed -n 's/^version = "\([0-9.]*\)"/\1/p' "$CARGO" | head -n1)"
  fi
fi
if [[ ! "$cur" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  tag="$(git -C "$ROOT" describe --tags --abbrev=0 2>/dev/null || true)"
  cur="${tag#v}"
fi
if [[ ! "$cur" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  cur="1.0.1"
fi

bump() {
  local a b p
  IFS=. read -r a b p <<< "$1"
  p=$((10#$p + 1))
  echo "${a}.${b}.${p}"
}

is_released() {
  git -C "$ROOT" show-ref --tags --verify --quiet "refs/tags/v$1" && return 0
  git -C "$ROOT" ls-remote --exit-code --tags origin "refs/tags/v$1" >/dev/null 2>&1 && return 0
  return 1
}

if is_released "$cur"; then
  cur="$(bump "$cur")"
fi
if [[ "$MODE" == "always" ]]; then
  cur="$(bump "$cur")"
fi

if [[ "$MODE" != "dry" ]]; then
  printf '%s\n' "$cur" > "$FILE"
  if [[ -f "$CARGO" ]]; then
    sed -i "0,/^version = /s/^version = \".*\"/version = \"${cur}\"/" "$CARGO"
  fi
fi
printf '%s\n' "$cur"
