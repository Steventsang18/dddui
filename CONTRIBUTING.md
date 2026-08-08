# 贡献指南 (Contributing to DoDidDoneUi)

感谢你关注 DoDidDoneUi。本仓库由两部分构成，均为独立 git 仓库：

- `upstream-AionUi/` — 前端（React 19 + TypeScript + Vite + UnoCSS + Arco Design）
- `upstream-AionCore/` — 后端（Rust / Axum 0.8 / SQLx / 26 个 crate），产物为单二进制 `dodiddoneui`
- `upstream-AionCore/extern/rupoo/` — 内嵌的 Rupoo agent 引擎副本（MIT，Steventsang18，上游 `https://github.com/Steventsang18/rupoo`）

## 开发环境

- Rust toolchain **1.95.0**（edition 2024）
- Node.js **24**（使用 `npm`，**非** bun）
- 操作系统：macOS / Linux（Windows 未验证）

## 本地构建

```bash
# 一键构建单二进制（前端内嵌）
./scripts/build-binary.sh --release
# 产物：upstream-AionCore/target/release/dodiddoneui

# 或开发热更新模式（两个终端）
cd upstream-AionCore && cargo run -- --port 3080 --host 127.0.0.1 --identity-mode owner
cd upstream-AionUi   && npm install && npm run dev   # http://127.0.0.1:5173
```

## 提交规范

- 修改上游衍生文件时，保留原 `Copyright 2025 AionUi (aionui.com)` 头注，并新增
  `Modified by DoDidDoneUi 团队 on <YYYY-MM-DD>` 头注（Apache-2.0 第 4 条）。
- Rust：提交前确保 `cargo fmt --check` 与 `cargo clippy --workspace --exclude rupoo -- -D warnings` 零告警。
  > `rupoo` 为 vendored 外部引擎，其 lint 不在本项目的质量门内。
- 前端：`npm run lint`（oxlint，0 error）与 `npm run build` 应通过。
- 不得放宽安全底线：默认仅绑定 `127.0.0.1`、禁止 `Allow-Origin: *`、保留 JWT+CSRF、校验 `Host` 防 DNS Rebinding。

## 红线（请勿改动）

- `iOfficeAI/RoseUi` 外部 URL、`aionrs`/`Aionrs` 内核、`aion:*` 协议、`rupoo` 引擎路径
- `ROSEUI_` 错误码、`x-roseui-*` 头、`roseui-*` crate 名
- 不引入 Tauri / 桌面客户端；不替换 SQLx；不新增大型运行时依赖

## 同步上游 AionUi / AionCore

本项目的代码基于 AionUi (Apache-2.0) 修改，名称与商标独立于 aionui.com。
同步上游补丁时，请保持其原始版权与许可声明不变。
