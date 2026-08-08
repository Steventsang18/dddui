# DoDidDoneUi 前端 (DDDUI)

> 本地优先、垂直行业向的多 Agent 编排平台前端。基于 [AionUi](https://github.com/iOfficeAI/AionUi) (Apache-2.0) 深度改造，独立品牌 **DDDUI**，不暗示与上游的任何官方关联。

本仓库是 **DoDidDoneUi 的纯前端**（React 19 + TypeScript + UnoCSS + Arco Design）。它作为静态资源被后端 `dodiddoneui`（Rust 单二进制）在编译期通过 `rust-embed` 内嵌，运行期由同一二进制同源托管，**无需任何桌面客户端**。

## 技术栈

- React 19 + TypeScript
- Vite（多入口 MPA）
- UnoCSS + Arco Design
- oxlint / oxfmt 代码规范

## 开发

```bash
npm install
npm run dev          # Vite dev server，默认 http://127.0.0.1:5173
```

Vite 已配置代理：`/api/*` → 后端 `:3080`、`/ws` → `ws://:3080`（同源联调）。
需要先在「模型」设置中配置国产大模型（如 DeepSeek）并粘贴 API Key，方可使用对话 / Agent 能力。

## 构建

```bash
npm run build       # 产物输出到 packages/desktop/src/renderer/dist
npm run lint        # oxlint（0 error 为准；部分 warning 为历史既有，非阻断）
```

构建产物（`dist/`）由后端在编译 `dodiddoneui` 时经 `rust-embed` 内嵌；开发期也可让后端以
`dodiddoneui --static-dir <本仓库 dist 目录>` 跳过内嵌、直接伺服本地构建产物。

## 目录结构

```
packages/desktop/src/renderer/   # 主前端应用（React）
packages/desktop/src/common/     # 前后端共享类型 / 常量
packages/shared-scripts/         # 发布相关脚本（prepare-dodiddoneui 等）
public/                         # PWA / favicon 静态资源
resources/                      # 文档与品牌资源
```

## i18n

界面文案位于 `packages/desktop/src/renderer/services/i18n/locales/`，支持 13+ 语言。
品牌展示统一使用简称 **DDDUI**。

## 许可证

- 原始代码：Apache License 2.0 — Copyright 2025 AionUi (aionui.com)
- 本项目修改与新代码：Apache License 2.0 — Copyright 2026 DoDidDoneUi 团队
- 见仓库根 [`LICENSE`](./LICENSE) 与 [`NOTICE`](./NOTICE)

根据 Apache 2.0 第 6 条，"AionUi" 名称与商标不随许可证授予，本前端以独立名称 **DDDUI** 分发。
