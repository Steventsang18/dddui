# DoDidDoneUi — 本地垂直行业 Agent 平台

> 本地化、垂直行业向的 Agent 编排平台，桌面 App（Tauri）/ 单 Rust 二进制 + 浏览器双形态，数据不出本机。基于 Apache-2.0 许可的上游开源代码修改而来（署名见许可证章节）。

📘 英文版文档：[`README.en.md`](./README.en.md)

## 项目定位

本项目**不是通用型 AI 聊天平台**，而是面向**特定垂直行业**（法律、教育、医疗、金融等）的本地化 Agent 解决方案。核心特征：

- **数据不出本机**：后端完全运行在本机，无云端依赖（模型 API 除外，由用户自备 Key）
- **垂直行业适配**：针对细分市场的领域知识库、工作流、合规要求做定向优化
- **本地优先**：桌面 App 与 Web 双形态，单人即可部署

## 为什么选 DoDidDoneUi？

- **数据不出本机**：应用完全跑在你的机器上，没有云后端（除你用自己的 Key 连接的模型 API）。
- **垂直行业就绪**：领域知识库、工作流与合规护栏是为特定市场调优的，而非"通用聊天"一刀切。
- **本地优先、双形态**：桌面 App（Tauri 壳 + 本机后端 sidecar，产物 `.app`/`.dmg`）开箱即用；也可以只跑单个 Rust 二进制用浏览器访问。一个人用笔记本就能部署。

## 功能详介

### 🤖 多 Agent 编排
把多个专职 Agent 串联、协同，解决复杂的多步骤任务。每个 Agent 可被赋予角色、工具与知识范围，再组合成工作流。

### 📚 垂直行业知识库（Wiki）
内置面向领域的 Markdown 知识库：
- 基于 SQLite **FTS5** 的全文检索（快、确定性、离线可用）。
- 笔记间的**双向链接**与 typed edges，可构建知识图谱。
- 通过 MCP `wiki_*` 工具由 Agent 读/写/搜，也可在界面里直接管理。
- 拖入 `pdf` / `docx` / `md` / `txt`——由纯 Rust 本地解析为汇总页 + 可检索切片。原始文件永不被修改。

### 🔌 MCP（模型上下文协议）
通过 MCP 服务器扩展工具与外部数据源。既可用内置 wiki MCP 工具，也可接入你自己的。

### 💬 对话
在简洁的对话界面中与模型、Agent 交流。若尚未配置任何模型，应用会给出友好引导而非报错。

### 🗂 文件与 Office
通过 Agent 直接处理本机文件与 Office 文档——Agent 可读取、编辑并基于你的本地文件推理。

### ⏰ 定时任务
设置周期性或定时任务，让 Agent / 工作流自动运行。

### 👥 团队（规划中）
团队协作层在规划中。当前单主控模式下打开即用，无登录墙。

### 📦 单二进制，前端内嵌
React 前端在编译期经 `rust-embed` 嵌入 Rust 二进制，运行期同源托管。无需独立 Web 服务器、无需额外目录。

## 快速开始

### 方式 A — 桌面 App（推荐）

macOS 原生窗口体验：Tauri 壳负责窗口/托盘，后端作为 sidecar 随 App 启动与退出，数据目录独立（`~/Library/Application Support/com.dodiddoneui.desktop`）。

```bash
# 全量构建，产出 .app / .dmg
./desktop/build.sh
#    产物：desktop/src-tauri/target/release/bundle/{macos,dmg}/

# 增量构建可跳过已完成的部分
./desktop/build.sh --skip-backend   # 复用已有后端二进制
./desktop/build.sh --skip-frontend  # 复用已有前端 dist
./desktop/build.sh --dev            # 开发模式（tauri dev + vite HMR）
```

把 `DoDidDoneUi.app` 拖进 Applications 即可使用。详见 [`desktop/README.md`](./desktop/README.md)。

### 方式 B — 单二进制 + 浏览器

想要一个前端已内嵌的生产形态单二进制：

```bash
# 1. 构建单二进制（前端会自动内嵌）
./build.sh            # 等价于 ./scripts/build-binary.sh --release
#    产物：backend/target/release/dodiddoneui

# 2. 启动（release 形态，前端已内嵌；默认绑定 127.0.0.1:3080）
./backend/start-roseui.sh --release

# 3. 浏览器打开
#    http://127.0.0.1:3080
```

`start-roseui.sh` 支持：`--release`（发布形态，内嵌前端）/ 默认（开发态，读磁盘 dist）/ `--stop` / `--restart`。就这么简单——不需要 Node，不需要额外服务器，二进制包揽一切。

> **首次配置**：在能聊天或使用 Agent 之前，请到 **设置 → 模型** 配置一个国产主流大模型（如 DeepSeek）并粘贴你的 API Key。配置后对话 / Agent 功能即会启用。

### 方式 C — 开发模式（热更新）

如果你是开发者，想在改前端时实时刷新：

```bash
# 终端 1 — 后端
cd backend && cargo run -- --port 3080 --host 127.0.0.1 --identity-mode owner

# 终端 2 — 前端（Vite dev server，HMR）
cd frontend && npm install && npm run dev
#    浏览器打开 http://127.0.0.1:5173
```

> 首次模型配置同方式 B：先在 **设置 → 模型** 配置模型，再使用对话 / Agent 功能。

### 方式 D — 伺服已构建的前端目录（高级）

如果你已有前端 `dist`，想跳过内嵌副本：

```bash
./backend/target/release/dodiddoneui --port 3080 --host 127.0.0.1 --static-dir <前端dist目录路径>
```

## 架构

```
dodiddoneui (单 Rust 二进制)
 ├─ roseui-webhost   rust-embed 内嵌前端 dist + SPA 静态托管（含 history fallback）
 ├─ 26 个业务 crate（Agent/MCP/对话/文件/Office/Team/定时任务/Wiki...）
 └─ roseui-wiki      垂直行业知识库（FTS5 全文检索 + 双链 + typed edges）

桌面形态：DoDidDoneUi.app (Tauri 壳) ─ sidecar 拉起 ─→ dodiddoneui --local
Web  形态：浏览器 ←→ REST /api/* + WebSocket /ws ←→ dodiddoneui（同源）
两种形态共用同一个后端与同一套 REST/WS 协议。
```

## 许可与来源

- 原始代码：Apache License 2.0 — Copyright 2025 AionUi (aionui.com)
- 本项目修改与新代码：Apache License 2.0 — Copyright 2026 DoDidDoneUi 团队
- 内置 Rupoo agent 引擎：MIT License — Copyright 2026 Steventsang18（见 [`NOTICE`](./NOTICE)）

详见 [`NOTICE`](./NOTICE) 与 [`LICENSE`](./LICENSE)（上游副本见 [`frontend/LICENSE`](./frontend/LICENSE) / [`backend/LICENSE`](./backend/LICENSE)）。

本项目基于 Apache-2.0 许可的上游代码修改而来，以**独立名称 DoDidDoneUi** 分发；根据 Apache 2.0 第 6 条，上游名称与商标不随许可证授予，本项目不暗示与上游的任何关联。

## 免责声明

本项目按 "AS IS" 提供，不含任何明示或暗示担保。使用前请遵守所在行业的合规要求（数据隐私、执业资质等）。

<!-- keep-test -->
