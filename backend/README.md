# DoDidDoneUi 后端 (DDDUI)

> 本地优先、垂直行业向的多 Agent 编排平台后端。基于 [AionCore](https://github.com/iOfficeAI/AionCore) (Apache-2.0) 深度改造，独立品牌 **DDDUI**，不暗示与上游的任何官方关联。

本仓库是 **DoDidDoneUi 的纯后端**：一个单 Rust 二进制 `dodiddoneui`（Axum 0.8 + Tokio + SQLx），
在编译期通过 `rust-embed` 内嵌前端静态产物（`dist/`），运行期同源托管 Web UI。
**禁止任何桌面客户端**（不 Tauri、不打包 exe/dmg/AppImage）。

## 技术栈

- Rust（edition 2024，toolchain `1.95.0`）
- Axum 0.8 + Tokio + SQLx（SQLite，禁止更换 ORM）
- tower-http（静态托管 / CORS / 限流）
- 内嵌 agent 引擎：**rupoo**（vendored 于 `extern/rupoo`，MIT License）

## 快速开始

### 开发模式

```bash
cargo run -p roseui-app -- --port 3080 --host 127.0.0.1 --identity-mode owner
# 浏览器访问 http://127.0.0.1:3080
```

> 默认 `IdentityMode::Owner`：单主控自用，打开即用、无登录墙。
> 首次使用需在「模型」设置中配置国产大模型（如 DeepSeek）并粘贴 API Key。

### 生产构建（单二进制，内嵌前端）

前端构建完成后，`dist/` 由 `rust-embed` 在编译期内嵌：

```bash
cargo build --release -p roseui-app
# 产物：target/release/dodiddoneui

./target/release/dodiddoneui --port 3080 --host 127.0.0.1
```

启动脚本（固化 data-dir 与 JWT 密钥，避免数据漂移）见 `start-roseui.sh`。

## 架构

```
dodiddoneui（单 Rust 二进制）
 ├─ roseui-webhost     rust-embed 内嵌前端 dist + SPA 静态托管（history fallback）
 ├─ 26 个业务 crate（Agent / MCP / 对话 / 文件 / Office / Team / 定时任务 / Wiki ...）
 └─ roseui-wiki        垂直行业知识库（SQLite FTS5 全文检索 + 双链 + typed edges）
```

浏览器 ←→ REST `/api/*` + WebSocket `/ws` ←→ `dodiddoneui`（同源）

## 安全底线

- 默认绑定 `127.0.0.1`，禁止 `Allow-Origin: *`
- 保留 JWT + CSRF；校验 Host 防 DNS rebinding；WS 握手校验 Origin
- 遥测默认关闭（feature gate）；rustfmt + clippy 零告警（rupoo 引擎豁免）

## 目录结构

```
crates/roseui-app/        # 二进制入口（name = "dodiddoneui"）
crates/roseui-db/         # SQLx migrations（不可修改已应用迁移，只可新增）
crates/roseui-webhost/    # 前端静态托管 + rust-embed
crates/roseui-ai-agent/   # agent 运行时（内置 rupoo 引擎调用）
extern/rupoo/             # vendored agent 引擎（MIT，单仓可编译关键）
migrations/               # 数据库迁移
```

## 许可证

- 原始代码：Apache License 2.0 — Copyright 2025 AionCore
- 本项目修改与新代码：Apache License 2.0 — Copyright 2026 DoDidDoneUi 团队
- 内置 Rupoo agent 引擎：MIT License — Copyright 2026 Steventsang18（见 [`NOTICE`](./NOTICE)）

见仓库根 [`LICENSE`](./LICENSE) 与 [`NOTICE`](./NOTICE)。
根据 Apache 2.0 第 6 条，"AionCore" 名称与商标不随许可证授予，本后端以独立名称 **DDDUI** 分发。
