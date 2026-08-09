# DoDidDoneUi — 本地垂直行业 Agent 平台

> 本地化、垂直行业向的 Agent 编排平台，单 Rust 二进制 + 浏览器，数据不出本机。基于 Apache-2.0 许可的上游开源代码修改而来（署名见许可证章节）。

## 项目定位

本项目**不是通用型 AI 聊天平台**，而是面向**特定垂直行业**（法律、教育、医疗、金融等）的本地化 Agent 解决方案。核心特征：

- **数据不出本机**：单一 Rust 二进制，浏览器访问，无云端依赖（模型 API 除外，由用户自备 Key）
- **垂直行业适配**：针对细分市场的领域知识库、工作流、合规要求做定向优化
- **本地优先**：纯 Web 形态，无桌面客户端，单人即可部署


## 许可与来源

- 原始代码：Apache License 2.0 — Copyright 2025 AionUi (aionui.com)
- 本项目修改与新代码：Apache License 2.0 — Copyright 2026 DoDidDoneUi 团队
- 内置 Rupoo agent 引擎：MIT License — Copyright 2026 Steventsang18（见 [`NOTICE`](./NOTICE)）

详见 [`NOTICE`](./NOTICE) 与 [`LICENSE`](./LICENSE)（上游副本见 [`frontend/LICENSE`](./frontend/LICENSE) / [`backend/LICENSE`](./backend/LICENSE)）。

本项目基于 Apache-2.0 许可的上游代码修改而来，以**独立名称 DoDidDoneUi** 分发；根据 Apache 2.0 第 6 条，上游名称与商标不随许可证授予，本项目不暗示与上游的任何关联。

## 快速开始

### 开发模式（热更新）

```bash
# 终端 1 — 后端
cd backend && cargo run -- --port 3080 --host 127.0.0.1 --identity-mode owner

# 终端 2 — 前端（Vite dev server，HMR）
cd frontend && npm install && npm run dev
# 浏览器打开 http://127.0.0.1:5173
```

> 首次使用需在「模型」设置中配置国产大模型（如 DeepSeek）并粘贴 API Key，方可使用对话 / Agent 能力。

### 生产模式（单二进制，前端内嵌）

一键构建脚本会：构建前端 → 拷贝产物到后端内嵌目录 → 编译 `dodiddoneui`：

```bash
./scripts/build-binary.sh --release
# 产物：backend/target/release/dodiddoneui

# 启动（默认绑定 127.0.0.1，首启自动打开浏览器）
./backend/target/release/dodiddoneui --port 3080 --host 127.0.0.1
# 浏览器访问 http://127.0.0.1:3080
```

> 前端 `vite build` 产物在编译期经 `rust-embed` 内嵌进 `dodiddoneui` 二进制，
> 运行期同源托管，无需任何桌面客户端或额外目录。开发期也可用
> `dodiddoneui --static-dir <前端dist目录>` 跳过内嵌、直接伺服本地构建产物。

## 架构

```
dodiddoneui (单 Rust 二进制)
 ├─ roseui-webhost   rust-embed 内嵌前端 dist + SPA 静态托管（含 history fallback）
 ├─ 26 个业务 crate（Agent/MCP/对话/文件/Office/Team/定时任务/Wiki...）
 └─ roseui-wiki      垂直行业知识库（FTS5 全文检索 + 双链 + typed edges）

浏览器 ←→ REST /api/* + WebSocket /ws  ←→ dodiddoneui（同源）
```

## 免责声明

本项目按 "AS IS" 提供，不含任何明示或暗示担保。使用前请遵守所在行业的合规要求（数据隐私、执业资质等）。

<!-- keep-test -->
