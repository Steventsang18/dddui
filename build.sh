#!/usr/bin/env bash
# DoDidDoneUi (DDDUI) 一键构建入口
#
# 构建出内嵌前端的单 Rust 二进制 dodiddoneui：
#   1. 前端 vite build (frontend/)
#   2. dist 经 rust-embed 内嵌进后端 (backend/crates/roseui-webhost/assets)
#   3. cargo build 编译 dodiddoneui
#
# 用法:
#   ./build.sh            # release 构建
#   ./build.sh --debug   # debug 构建（编译快，运行慢）
#
# 前置依赖:
#   - Rust toolchain 1.95.0 (rustup)
#   - Node.js >=22 <25 + npm
set -euo pipefail
exec "$(dirname "${BASH_SOURCE[0]}")/scripts/build-binary.sh" "$@"
