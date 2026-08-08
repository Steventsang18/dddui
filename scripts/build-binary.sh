#!/usr/bin/env bash
# 一键构建 DoDidDoneUi 单二进制（内嵌前端）：
#   1. 前端 vite build
#   2. 拷贝 dist 到后端 webhost 内嵌目录
#   3. cargo build --release 编译 dodiddoneui
#
# 用法: ./scripts/build-binary.sh [--release|--debug]
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
UI_DIR="$ROOT/frontend"
CORE_DIR="$ROOT/backend"
WEBHOST_ASSETS="$CORE_DIR/crates/roseui-webhost/assets"

MODE="${1:---release}"
case "$MODE" in
  --release) CARGO_PROFILE="--release" ;;
  --debug)   CARGO_PROFILE="" ;;
  *) echo "unknown mode: $MODE" >&2; exit 1 ;;
esac

echo "==> [1/3] building frontend (vite build)"
cd "$UI_DIR"
if [ ! -d node_modules ]; then
  echo "    installing npm deps..." >&2
  npm install
fi
npm run build

echo "==> [2/3] copying dist -> $WEBHOST_ASSETS"
rm -rf "$WEBHOST_ASSETS"/*
mkdir -p "$WEBHOST_ASSETS"
cp -r "$UI_DIR/packages/desktop/src/renderer/dist/." "$WEBHOST_ASSETS"/

echo "==> [3/3] cargo build $CARGO_PROFILE"
cd "$CORE_DIR"
# shellcheck disable=SC2086
cargo build $CARGO_PROFILE

echo "==> done. run: ./backend/target/${CARGO_PROFILE:+release/}dodiddoneui --port 3080 --host 127.0.0.1"
