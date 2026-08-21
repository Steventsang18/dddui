#!/usr/bin/env bash
# DoDidDoneUi 桌面端一键构建：后端单二进制 → sidecar 同步 → 前端 dist → Tauri 打包
#
# 用法：
#   ./desktop/build.sh                 # 全量构建，产出 .app / .dmg
#   ./desktop/build.sh --dev           # 开发模式（tauri dev + vite HMR）
#   ./desktop/build.sh --skip-backend  # 复用已有后端二进制
#   ./desktop/build.sh --skip-frontend # 复用已有前端 dist
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DESKTOP_DIR="$ROOT/desktop"
TARGET_TRIPLE="$(rustc -vV | awk '/^host:/ {print $2}')"

MODE="build"
SKIP_BACKEND=false
SKIP_FRONTEND=false
for arg in "$@"; do
  case "$arg" in
    --dev) MODE="dev" ;;
    --skip-backend) SKIP_BACKEND=true ;;
    --skip-frontend) SKIP_FRONTEND=true ;;
    *) echo "未知参数: $arg（可用：--dev --skip-backend --skip-frontend）" >&2; exit 1 ;;
  esac
done

# 1. 后端单二进制（只构建 roseui-app 的 dodiddoneui bin，不碰其他 bin）
if [ "$SKIP_BACKEND" = false ]; then
  echo "==> 构建后端 (cargo build --release -p roseui-app)"
  (cd "$ROOT/backend" && cargo build --release -p roseui-app)
fi

# 2. sidecar 同步到 Tauri 工程（externalBin 按 target triple 命名）
SIDECAR_SRC="$ROOT/backend/target/release/dodiddoneui"
if [ ! -x "$SIDECAR_SRC" ]; then
  echo "错误：未找到后端二进制 $SIDECAR_SRC，请去掉 --skip-backend 重试" >&2
  exit 1
fi
mkdir -p "$DESKTOP_DIR/src-tauri/binaries"
cp "$SIDECAR_SRC" "$DESKTOP_DIR/src-tauri/binaries/dodiddoneui-$TARGET_TRIPLE"
chmod +x "$DESKTOP_DIR/src-tauri/binaries/dodiddoneui-$TARGET_TRIPLE"

# 3. 前端
if [ "$SKIP_FRONTEND" = false ]; then
  echo "==> 构建前端 (npm run build)"
  (cd "$ROOT/frontend" && npm run build)
fi
# vite root = packages/desktop/src/renderer，产物落在其下的 dist/
FRONTEND_DIST="$ROOT/frontend/packages/desktop/src/renderer/dist"
if [ "$MODE" = "build" ] && [ ! -f "$FRONTEND_DIST/index.html" ]; then
  echo "错误：未找到前端产物 $FRONTEND_DIST，请去掉 --skip-frontend 重试" >&2
  exit 1
fi

# 4. Tauri
if [ "$MODE" = "dev" ]; then
  echo "==> cargo tauri dev"
  (cd "$DESKTOP_DIR" && cargo tauri dev)
else
  echo "==> cargo tauri build"
  (cd "$DESKTOP_DIR" && cargo tauri build)
  echo "==> 产物位于 $DESKTOP_DIR/src-tauri/target/release/bundle/"
fi
