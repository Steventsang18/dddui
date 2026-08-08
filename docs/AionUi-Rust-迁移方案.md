# AionUi → Rust 迁移技术方案

> 文档版本：v1.0　|　编制日期：2026-07-31
> 对象项目：`https://github.com/iOfficeAI/AionUi`（v2.1.44）+ `https://github.com/iOfficeAI/AionCore`（v0.1.55）
> 状态：**技术规划，不含编码实施**

---

## 0. 执行摘要（先读这一节）

在实际克隆并逐层剖析源码后，得到一个**颠覆常规假设的关键结论**：

> **AionUi 的后端已经 100% 完成 Rust 化。**

官方在 v2.x 已把全部业务后端拆出为独立 Rust 仓库 **AionCore**（Cargo Workspace，24 个 crate，Axum 0.8 + Tokio + SQLx/SQLite），编译产物为单一二进制 `aioncore`。Electron 侧仅剩下：

- 一个**桌面外壳**（窗口 / 托盘 / 菜单 / 自动更新 / 深链 / 桌宠），约 1.4 万行 TS；
- 一个 **Node 版 WebUI 宿主**（静态服务器 + 反向代理 + 后端进程启动器），约 2 千行 TS；
- 一个**纯浏览器 React 前端**（renderer，10.2 万行 TSX，**零 Node/Electron 直接依赖**）。

因此本方案的正确命题不是"把 AionUi 用 Rust 重写"，而是：

> **把残留在 Node/Electron 上的 ~16% 代码迁移到 Rust，形成"Tauri 外壳 + AionCore 后端"的全 Rust 原生栈。**

推荐终局架构：**Tauri 2.x（Rust 外壳）+ AionCore（Rust 后端，进程内库或 sidecar）+ React 前端（保留，长期可选 Leptos/Dioxus）**。

预期收益：安装包 190MB → 约 25MB；常驻内存降低 55%~70%；冷启动提速约 2~3 倍；构建链去 Node 化。

预估工作量：**核心路径 5.5 人月**，全量（含前端 Rust 化）12~16 人月。

---

## 1. 原项目架构与技术栈分析

### 1.1 双仓结构与运行时拓扑

```
┌──────────────────────── AionUi 仓库（TypeScript）────────────────────────┐
│                                                                          │
│  packages/desktop                                                        │
│   ├── src/index.ts (1,025 行)      Electron 主进程入口                    │
│   ├── src/process/  (12,361 行)    外壳服务：bridge / startup / pet /     │
│   │                                 tray / updater / gpuRecovery         │
│   ├── src/preload/  (150 行)       contextBridge，仅暴露 ~10 个 API       │
│   ├── src/common/   (13,152 行)    跨进程共享：adapter / api / chat /     │
│   │                                 config / platform / types            │
│   └── src/renderer/ (101,802 行)   React 19 SPA —— 纯浏览器代码           │
│                                                                          │
│  packages/web-host   (~2,000 行)   WebUI 模式：静态服务器 + 反代 +        │
│                                     aioncore 进程生命周期管理             │
│  packages/web-cli    (3 文件)      CLI 启动器                            │
│  packages/shared-scripts                                                 │
└──────────────────────────────────────────────────────────────────────────┘
                                    │
                         HTTP REST  │  WebSocket /ws
                    127.0.0.1:13400 │  （端口由主进程动态探测后注入）
                                    ▼
┌──────────────────── AionCore 仓库（Rust，已完成）────────────────────────┐
│  aioncore 单二进制 · Cargo Workspace · 24 crates · Axum 0.8 + Tokio      │
└──────────────────────────────────────────────────────────────────────────┘
```

**通信契约**（`packages/desktop/src/common/adapter/httpBridge.ts`，503 行）：

- 渲染进程通过 `httpGet/httpPost/httpPut/httpPatch/httpDelete` 工厂调用后端 REST，共 **124 条 `/api/*` 路由**；
- 实时事件通过**单例 WebSocket**（`/ws`）下发，帧格式 `{ name, data }`，含指数退避重连（上限 30s）与 `realtime.reconnected` 重同步事件；
- 该文件被明确设计为 `ipcBridge.buildProvider/buildEmitter` 的 **drop-in 替代品**——即"渲染层不感知底层是 IPC 还是 HTTP"。这是迁移的最大红利。

### 1.2 代码规模精确分布

| 层次 | 路径 | 行数 | 语言/运行时 | 迁移定性 |
|---|---|---:|---|---|
| React 前端 | `desktop/src/renderer/` | **101,802** | 浏览器 | **原样保留**（Tauri WebView 直接承载） |
| ├─ pages | `renderer/pages/` | 54,436 | | conversation / settings / team / cron / login / guid |
| ├─ components | `renderer/components/` | 27,202 | | Arco Design + UnoCSS |
| ├─ hooks | `renderer/hooks/` | 9,319 | | |
| ├─ services | `renderer/services/` | 5,562 | | i18n / FileService / SpeechStreamClient |
| └─ utils | `renderer/utils/` | 4,016 | | |
| 共享层 | `desktop/src/common/` | **13,152** | 同构 | **部分下沉 Rust** |
| ├─ adapter | `common/adapter/` | 3,412 | | `ipcBridge.ts` 2,073 + `httpBridge.ts` 503 |
| ├─ chat | `common/chat/` | 2,820 | | 含 `DocumentConverter`（mammoth/xlsx） |
| ├─ config | `common/config/` | 1,775 | | storage / configService / configMigration |
| ├─ types | `common/types/` | 1,708 | | **应改为 Rust 生成** |
| ├─ api | `common/api/` | 1,370 | | RotatingApiClient / 协议转换器 |
| ├─ utils / platform / update / theme | | 1,975 | | 已有 `IPlatformServices` 抽象 |
| 主进程外壳 | `desktop/src/process/` | **12,361** | Node/Electron | **重写为 Rust** |
| 入口 + 遥测 | `index.ts` + `sentry.ts` | 1,573 | Node/Electron | **重写为 Rust** |
| WebUI 宿主 | `packages/web-host/` | ~2,000 | Node | **重写为 Rust** |
| preload | `desktop/src/preload/` | 150 | Electron | **重写为 Tauri IPC** |

> **总计约 131,000 行 TS/TSX**（1,327 个文件）。其中**必须 Rust 化的仅约 16,000 行（12%）**，可选 Rust 化的前端为 101,800 行（78%）。

### 1.3 技术栈清单

**桌面外壳层**
| 领域 | 技术 |
|---|---|
| 运行时 | Electron 37.10 / Node.js ≥22 <25 |
| 构建 | electron-vite 5 + Vite 6 + esbuild |
| 打包 | electron-builder 26.15（+ squirrel-windows / notarize / fuses） |
| 更新 | electron-updater 6.6 + 自研 `cdnGenericProvider` |
| 遥测 | @sentry/electron 7 |
| 日志 | electron-log 5 |

**前端层**
| 领域 | 技术 |
|---|---|
| 框架 | React 19.1 + react-router-dom 7.8 |
| UI | @arco-design/web-react 2.66 + UnoCSS 66 + @icon-park/react |
| 数据 | SWR 2.3 |
| 编辑器 | CodeMirror 6 全家桶 + @monaco-editor/react 4.7 |
| 富文本渲染 | react-markdown 10 + remark-gfm/math/breaks + rehype-katex/raw + streamdown |
| 图表/公式 | mermaid 11.13 + katex 0.16 |
| 虚拟列表 | react-virtuoso 4.18 |
| 拖拽 | @dnd-kit/* |
| i18n | i18next 23 + react-i18next 14（10+ 语言） |
| 差异视图 | diff 8 + diff2html 3.4 |

**Node 侧残留（web-host / DocumentConverter）**
express 5、ws 8、multer、cors、cookie-parser、express-rate-limit、tiny-csrf、jsonwebtoken、bcryptjs、better-sqlite3 12.4（仅遗留迁移用）、mammoth、xlsx-republish、officeparser、pptx2json、docx、sharp、yauzl、croner。

**AionCore（Rust 后端，已完成）**

| 分层 | Crate | 职责 |
|---|---|---|
| Foundation | `aionui-common` | ApiError、枚举、ID 生成、加密、时间戳、分页 |
| | `aionui-api-types` | **全部 HTTP/WS 请求响应类型 —— API 契约唯一真相源** |
| | `aionui-db` | SQLite 层，Repository trait 与实现 |
| | `aionui-assets` | 内嵌静态资源（agent 元数据、prompts） |
| | `aionui-runtime` | 托管 Node、子进程 spawn、PATH 增强 |
| Capability | `aionui-auth` | JWT、密码哈希、CSRF 双提交 Cookie、鉴权中间件 |
| | `aionui-realtime` | WebSocket 连接管理、BroadcastEventBus、消息路由 |
| Domain | `aionui-conversation` | 会话、消息、确认、流式响应 |
| | `aionui-ai-agent` | Agent 生命周期、Worker 任务队列、ACP / 辅助技能 |
| | `aionui-mcp` | MCP 协议、OAuth、多平台适配 |
| | `aionui-team` | 团队协作、任务调度、Mailbox |
| | `aionui-cron` | 定时任务、cron 表达式、事件触发 |
| | `aionui-channel` | 微信 / 钉钉 / 飞书多渠道 + 插件系统 + 配对会话 |
| | `aionui-file` | 文件操作、watch、快照、git、压缩 |
| | `aionui-office` | Excel/PPT/Word 处理、预览、转换 |
| | `aionui-system` | 系统设置、Provider 管理、版本检查、模型拉取 |
| | `aionui-extension` | 扩展注册表、Hub、技能发现与安装 |
| | `aionui-shell` | Shell 命令执行、语音转文字 |
| | `aionui-assistant` / `aionui-project` / `aionui-session` / `aionui-process` / `aionui-team-prompts` | 助手配置 / 项目 / 会话 / 进程 / 团队提示词 |
| Composition | `aionui-app` | 二进制入口，装配 Axum Router |

外部依赖 `aionrs v0.2.8`（`aion-agent` / `aion-providers` / `aion-types` / `aion-protocol` / `aion-config` / `aion-mcp`）承载 LLM Provider 与协议层。

关键 crate：`axum 0.8`、`tokio`、`sqlx 0.8(sqlite)`、`rusqlite 0.32(bundled)`、`jsonwebtoken 9`、`bcrypt 0.17`、`aes-gcm`、`reqwest 0.12(rustls)`、`tokio-tungstenite 0.26`、`notify 8`、`git2 0.20`、`zip 2`、`calamine 0.36`、`rust_xlsxwriter 0.82`、`cron 0.15` + `chrono-tz`、`aws-sdk-bedrock`、`oauth2 5.0-rc`、`ed25519-dalek 2`、`rust-embed 8`、`dashmap 6`、`tracing`。

### 1.4 迁移可行性的四个决定性事实

| # | 事实 | 证据 | 对迁移的意义 |
|---|---|---|---|
| 1 | **后端已全 Rust** | AionCore 24 crates，`resolveBinaryPath()` 只解析 `aioncore` 二进制 | 最难的 60% 工作量已由上游完成 |
| 2 | **renderer 零 Electron 依赖** | `grep "from 'electron'" renderer/` → **0 处命中**；`node:fs`/`child_process` → **0 处命中** | React 代码可直接跑在 Tauri WebView |
| 3 | **preload 面极小** | `preload/main.ts` 仅 81 行，暴露 `emit/on/getPathForFile/collectFeedbackLogs/captureFeedbackScreenshot/logFeedbackEvent/recoverCorruptedDatabase` + 5 个 tray 事件 + 5 个全局变量 | Tauri `invoke`/`event` 一比一替换，工作量以"天"计 |
| 4 | **已存在平台抽象层** | `common/platform/IPlatformServices.ts` + `ElectronPlatformServices` / `NodePlatformServices` 双实现 | 只需新增 `TauriPlatformServices`，不改调用方 |

反向风险点：renderer 中仍有 **79 处**直接触碰 `window.electronAPI` / `__backendPort` 等全局，需统一收敛到平台抽象层（见 3.2 阶段一）。

---

## 2. 核心模块与功能向 Rust 生态的映射关系

### 2.1 桌面外壳（Electron → Tauri 2.x）

| 现有实现 | 文件 | Rust 目标 | 备注 |
|---|---|---|---|
| 主进程入口 | `src/index.ts` (1,025) | `src-tauri/src/main.rs` + `lib.rs` | Builder 装配 |
| 窗口生命周期 | `process/utils/mainWindowLifecycle.ts` | `tauri::WebviewWindowBuilder` | |
| 窗口位置记忆 | `process/utils/windowBounds.ts` | `tauri-plugin-window-state` | 官方插件直接覆盖 |
| 缩放 | `process/utils/zoom.ts` | `webview.set_zoom()` | |
| 系统托盘 | `process/utils/tray.ts` | `tauri::tray::TrayIconBuilder` | 5 个托盘事件 → `emit_to` |
| 应用菜单 | `process/utils/appMenu.ts` | `tauri::menu::MenuBuilder` | |
| 单实例 | `process/startup/singleInstanceGating.ts` | `tauri-plugin-single-instance` | |
| 深链 | `process/utils/deepLink.ts` | `tauri-plugin-deep-link` | |
| 自动更新 | `services/autoUpdaterService.ts` + `cdnGenericProvider.ts` + `updateFeed.ts` | `tauri-plugin-updater` + 自定义 endpoint | **需重建签名密钥体系（高风险，见 4.2）** |
| 对话框 | `bridge/dialogBridge.ts` | `tauri-plugin-dialog` | |
| 通知 | `bridge/notificationBridge.ts` | `tauri-plugin-notification` | |
| 系统设置 | `bridge/systemSettingsBridge.ts` | `tauri-plugin-os` + 自定义 command | |
| 主题 | `bridge/themeBridge.ts` | `window.theme()` + `on_theme_changed` | |
| 日志 | electron-log | `tracing` + `tracing-appender`（与 AionCore 统一） | |
| 崩溃遥测 | @sentry/electron | `sentry` crate + `@sentry/browser`（前端） | 需重建 IPC 转发链路 |
| 重启 | `bridge/restartApplication.ts` | `tauri-plugin-process::restart()` | |
| 截图（反馈） | `feedback:capture-screenshot` | `xcap` crate 或 `webview.capture()` | |
| 日志打包 | `process/feedback/logs.ts` | `zip` crate | 与 AionCore 共用 |
| GPU 恢复 / Chromium flags | `gpuRecovery.ts` / `configureChromium.ts` | **无对应能力，直接废弃** | 系统 WebView 自管理（见 4.1） |
| 桌宠（5 个文件） | `process/pet/*` | 多 `WebviewWindow`（transparent + always_on_top + `set_ignore_cursor_events`） | **Linux 高风险（见 4.1）** |

### 2.2 WebUI 宿主（Node → Rust）

| 现有实现 | 规模 | Rust 目标 |
|---|---|---|
| `backend-launcher.ts`（端口探测、健康检查、进程树 kill、启动诊断、`BackendLifecycleManager`） | 36 KB | `tokio::process::Command` + `sysinfo`（进程树）+ `tokio::net::TcpStream`（健康探测） |
| `static-server.ts`（静态托管、`/api/*` 反代、`/ws` 与 `/api/stt/stream` 原始 TCP splice） | 8.7 KB | `axum` + `tower-http::ServeDir` + `hyper` 反代 + `tokio::io::copy_bidirectional` |
| `agent-process-registry.ts` | 4.5 KB | `dashmap` 注册表 |
| `web-cli`（`bin/aionui-web.js`） | 3 文件 | `clap` 子命令，并入 `aioncore` CLI |

> 说明：`static-server.ts` 之所以手写 TCP splice 而非用 `http.Server` 的 `upgrade` 事件，是因为 Node 会吞掉 101 响应。Rust 侧 `copy_bidirectional` 天然规避此问题，**这块反而是简化**。

### 2.3 共享层（common/ → Rust + 生成式类型）

| 模块 | 处理策略 |
|---|---|
| `common/types/` (1,708 行) | **改为从 `aionui-api-types` 用 `ts-rs` 或 `specta` 自动生成 `.d.ts`**，消除手写契约漂移 |
| `common/adapter/httpBridge.ts` | 保留（浏览器 fetch/WebSocket），仅替换端口注入方式为 Tauri `invoke("get_backend_port")` |
| `common/adapter/ipcBridge.ts` (2,073 行) | **大幅裁剪**：其中绝大部分路由已走 HTTP；仅保留外壳类 IPC，改由 Tauri `invoke` 实现 |
| `common/api/*`（RotatingApiClient、协议转换器） | **下沉到 AionCore**（`aion-providers` 已具备同类能力），前端仅保留类型 |
| `common/chat/document/DocumentConverter.ts`（mammoth/xlsx） | **下沉到 `aionui-office`**（已有 `calamine` + `rust_xlsxwriter`）；docx 读取用 `docx-rs`/`dotext` |
| `common/config/*`（storage / configService / configMigration） | 下沉到 `aionui-system`，前端通过 REST 读写 |
| `common/platform/*` | 新增 `TauriPlatformServices` 实现 `IPlatformServices` |

### 2.4 遗留 SQLite 迁移（关键单点）

`process/services/database/migrations.ts` 达 **64 KB**，配合 `BetterSqlite3Driver.ts`、`repairLegacyHandoffSchema.ts`、`runLegacyDatabaseMigrations.ts`、`legacyHandoffContract.ts` 构成 Electron 时代的历史数据交接链路。

**策略**：不移植该逻辑到 Tauri 外壳，而是**做成一次性迁移工具**——
1. 在 AionCore 新增 `aionui-db::legacy_handoff` 模块（`rusqlite` 已 `bundled`，无需系统 SQLite）；
2. 首启检测旧 `userData` 目录下的遗留 DB，执行一次性导入并打标 `migrated_at`；
3. 提供 `aioncore migrate --from-electron <path>` CLI 供故障回捞；
4. 旧 `migrations.ts` 冻结，仅作为迁移规则的参考文档保留。

### 2.5 前端（可选终局，非必需）

| 现有 | Rust 生态候选 | 评估 |
|---|---|---|
| React 19 | Leptos 0.7 / Dioxus 0.6 | 10.2 万行重写，**不建议在本轮进行** |
| Arco Design | 无等价 Rust 组件库 | **阻塞项**——需自建设计系统，成本 > 6 人月 |
| Monaco / CodeMirror | 无 Rust 等价物 | 必须继续用 JS（可在 Leptos 中以 JS interop 嵌入） |
| mermaid / katex | 无成熟 Rust 等价物 | 同上 |

**结论：前端保持 React，通过 Tauri WebView 承载。** 全 Rust UI 列为 Phase 5 可选项，仅在有明确收益（如极致体积）时启动。

---

## 3. 分阶段迁移步骤及各阶段预期目标

### 总览

| 阶段 | 名称 | 周期 | 人力 | 核心交付 | 可回退 |
|---|---|---|---|---|---|
| 0 | 基线固化与契约冻结 | 2 周 | 1.0 人 | 契约快照 + 基准数据 + E2E 兜底网 | — |
| 1 | 渲染层去 Electron 化 | 3 周 | 1.5 人 | renderer 100% 平台无关（**仍跑 Electron**） | ✅ 完全 |
| 2 | Tauri 外壳最小可用 | 4 周 | 2.0 人 | Tauri 版可启动、可对话、可预览 | ✅ 双栈并行 |
| 3 | 外壳能力对齐 | 5 周 | 2.0 人 | 托盘/更新/深链/桌宠/遥测全量对齐 | ✅ 双栈并行 |
| 4 | WebUI 宿主 Rust 化 + 去 Node | 3 周 | 1.5 人 | 构建链彻底去 Node | ⚠️ 部分 |
| 5 | 灰度发布与 Electron 下线 | 4 周 | 1.0 人 | Tauri 成为唯一分发形态 | ⚠️ 版本回滚 |
| （5+） | 前端 Rust 化 | 6~10 月 | 3.0 人 | **可选，默认不启动** | — |

**核心路径合计：21 周日历时间 / 约 5.5 人月。**

---

### 阶段 0 — 基线固化与契约冻结（2 周）

**目标**：在动任何代码前，把"正确"定义清楚，否则后续无法判定迁移是否等价。

**任务**
1. **API 契约快照**：从 `aionui-api-types` 导出 124 条 REST 路由 + 全部 WS 事件名的 OpenAPI/JSON Schema 快照，纳入 CI 做契约回归。
2. **建立类型生成管线**：在 AionCore 引入 `ts-rs`（或 `specta`），从 Rust 类型生成 `.d.ts`，替代手写 `common/types/`。**这是后续所有阶段的地基。**
3. **性能基线采集**：复用仓库已有的 `scripts/benchmark-startup.ts`、`scripts/run-benchmarks.ts`、`tests/bench/database.bench.bun.ts`，记录冷启动、首屏、内存 RSS、包体积、DB QPS。
4. **E2E 兜底网**：现有 Playwright 用例（`tests/e2e/cases/teams/*`）改为**外壳无关**（通过 WebUI 模式驱动，而非 `_electron.launch`），使同一套用例可同时验证 Electron 与 Tauri。
5. **依赖审计**：逐一确认 2.1~2.4 表中每个 Rust crate 的许可证（Apache-2.0 兼容性）、维护活跃度、平台覆盖。

**验收标准**
- 契约快照进入 CI，任何后端字段变更触发告警；
- 生成式 `.d.ts` 与手写类型 diff 为空（或差异已登记）；
- 基线报告归档；
- E2E 套件在 WebUI 模式下 100% 通过。

**风险**：`ts-rs` 对复杂泛型/枚举 tagging 的表达力有限，可能需要为部分类型手工标注。预留 3 天缓冲。

---

### 阶段 1 — 渲染层去 Electron 化（3 周）

**目标**：让 10.2 万行 React 代码对"宿主是谁"完全无感。**本阶段结束时仍然运行在 Electron 上**，零用户可见变化，风险极低。

**任务**
1. **收敛 79 处宿主全局引用**：将 renderer 中所有 `window.electronAPI`、`window.__backendPort`、`window.__initialLanguage`、`window.__backendStartupFailed/Failure`、`window.__aionuiE2ETest` 的直接访问，统一收口到单一模块 `renderer/services/HostBridge.ts`。
2. **定义宿主能力接口**（对应 preload 的 10 个 API）：
   ```
   IHostBridge {
     emit / on                      // 外壳事件通道
     getPathForFile(File): string   // 拖拽文件绝对路径
     collectFeedbackLogs()
     captureFeedbackScreenshot()
     logFeedbackEvent(payload)
     recoverCorruptedDatabase()
     getBackendPort(): number
     getInitialLanguage(): string | null
     getStartupFailure(): unknown
     onTrayEvent(name, cb)          // 5 个托盘事件
   }
   ```
3. **提供三套实现**：`ElectronHostBridge`（现状）、`WebHostBridge`（浏览器降级，能力返回 no-op）、`TauriHostBridge`（本阶段先留空桩）。
4. **ESLint 硬约束**：新增规则禁止 `renderer/**` 出现 `window.electron*` / `__backend*`，防止回潮。
5. **`common/adapter/ipcBridge.ts` 瘦身**：审计 2,073 行中哪些通道仍在使用（多数已被 `httpBridge` 取代），删除死代码，剩余项映射到 `IHostBridge`。

**验收标准**
- `grep -r "window.electronAPI" renderer/` → 仅 `HostBridge.ts` 1 处命中；
- Electron 版全量回归通过，性能基线无劣化；
- `ipcBridge.ts` 行数下降 ≥50%。

**回退**：本阶段是纯重构，任意时刻可停止，不影响主干。

---

### 阶段 2 — Tauri 外壳最小可用（4 周）

**目标**：产出可启动、可完成一次完整对话、可预览文件的 Tauri 版本；与 Electron 版**并行存在**，互不干扰。

**任务**
1. **建立 `src-tauri/`**：Tauri 2.x 项目骨架，Vite 前端指向现有 `renderer/`（构建配置几乎不变）。
2. **实现 `TauriPlatformServices` + `TauriHostBridge`**：以 `#[tauri::command]` 实现阶段 1 定义的 10 个接口。
3. **AionCore 集成（关键设计决策）**：

   | 方案 | 说明 | 优点 | 缺点 | 建议 |
   |---|---|---|---|---|
   | **A. Sidecar 子进程** | 沿用现状，Tauri 通过 `tauri-plugin-shell` sidecar 启动 `aioncore` | 与现状一致，AionCore 可独立升级，进程隔离 | 仍有端口探测与进程管理复杂度 | **阶段 2 采用** |
   | **B. 进程内库** | AionCore 各 crate 作为 lib 直接 link 进 Tauri 二进制，Axum 在同进程 Tokio runtime 内监听 | 单进程、启动最快、无端口竞争、无孤儿进程 | 崩溃域合并，AionCore 需暴露 lib 入口 | **阶段 3 评估切换** |

   > 建议先 A 后 B：A 保证快速可用，B 作为性能与稳定性优化项。切换成本低（仅改启动路径），因为通信仍走本地 HTTP。

4. **窗口与基础交互**：主窗口创建、`tauri-plugin-window-state` 位置记忆、缩放、`tauri-plugin-dialog`、`tauri-plugin-single-instance`。
5. **三平台 WebView 兼容性冒烟**：重点验证 Monaco、CodeMirror 6、mermaid 11、katex、react-virtuoso、Shadow DOM（`components/Markdown/ShadowView.tsx`）、CSS 自定义主题在 **WKWebView / WebView2 / WebKitGTK** 下的表现。**这是本阶段最大不确定性，应在第 1 周就完成，不要拖到最后。**

**验收标准**
- macOS / Windows / Linux 三端可启动并完成一次完整对话（含流式输出）；
- 文件预览面板（PDF/Office/代码/Markdown/图片/Diff）在三端可用；
- 包体积 < 40 MB；冷启动优于 Electron 基线；
- 输出《WebView 兼容性差异清单》，逐项标注严重级别与对策。

**风险**：Linux WebKitGTK 若出现阻塞性渲染问题，触发 4.1 的应急预案。

---

### 阶段 3 — 外壳能力对齐（5 周）

**目标**：Tauri 版达到 Electron 版的功能完备度。

**任务（按优先级）**

| P | 能力 | 实现要点 |
|---|---|---|
| P0 | 系统托盘 + 5 个托盘事件 | `TrayIconBuilder` + `emit_to("main", ...)` |
| P0 | 应用菜单 | `MenuBuilder`，macOS 需处理 App Menu 特殊项 |
| P0 | 自动更新 | `tauri-plugin-updater` + 自建 endpoint 复刻 `cdnGenericProvider` 的 CDN 回退策略；**重建 minisign 密钥对与签名流水线** |
| P0 | 遗留 SQLite 迁移 | 落地 2.4 的一次性迁移工具，覆盖存量用户数据 |
| P0 | 崩溃遥测 | `sentry` crate（Rust 侧）+ `@sentry/browser`（前端），重建 breadcrumb 与 release 关联 |
| P1 | 深链 | `tauri-plugin-deep-link`，注意 macOS 需 Info.plist、Windows 需注册表 |
| P1 | 通知 | `tauri-plugin-notification` |
| P1 | 反馈截图 + 日志打包 | `xcap` / `webview.capture()` + `zip` crate |
| P1 | 启动诊断 | 移植 `backendStartupFailure.ts` / `backendInstallDiagnostics.ts` / `architectureCompatibility.ts` 的错误分类与用户提示 |
| P2 | 桌宠 | 多透明置顶窗口 + `set_ignore_cursor_events`；**Linux 若不可用则平台降级** |
| P2 | 关闭到托盘 / 退出清理 | `closeToTraySetting.ts` / `quitCleanup.ts` / `persistOnQuit.ts` 移植 |
| P3 | GPU 恢复 / Chromium flags | **明确废弃**，改为 WebView 崩溃后自动重载 + 用户提示 |

**并行任务**：评估阶段 2 的方案 A → B 切换（AionCore 进程内化）。

**验收标准**
- 功能对齐清单 100% 打钩或明确标注"已废弃/平台降级"；
- 完成一轮 Electron → Tauri 的真实用户数据迁移演练（含回滚演练）；
- 三端签名 + 公证（macOS notarize）+ 安装包冒烟通过。

---

### 阶段 4 — WebUI 宿主 Rust 化与去 Node（3 周）

**目标**：删除 `packages/web-host` 与 `packages/web-cli`，构建链去 Node 化。

**任务**
1. 在 AionCore 新增 `aionui-webhost` crate：`axum` + `tower-http::ServeDir` 托管前端静态产物（`rust-embed` 内嵌）+ `/api/*` 反代 + WS/STT 流 `copy_bidirectional`。
2. 前端产物内嵌进 `aioncore` 二进制，`aioncore serve --port 8080 --remote` 一条命令拉起完整 WebUI。
3. `web-cli` 能力并入 `aioncore` CLI 子命令（`clap`）：`serve` / `resetpass` / `migrate`。
4. 端口探测、健康检查、进程树管理逻辑用 `tokio` + `sysinfo` 重写（对应 `backend-launcher.ts` 的 36 KB 逻辑）。
5. 前端构建仍用 Vite（Node），但**仅存在于构建期**，运行期与分发物零 Node 依赖。

**验收标准**
- 删除 `packages/web-host`、`packages/web-cli`；
- 服务器部署场景（`docs/guides/deploy-server.md`）行为等价；
- Docker 镜像从 node-base 切换为 distroless/scratch，体积下降 ≥70%。

---

### 阶段 5 — 灰度发布与 Electron 下线（4 周）

**任务**
1. **灰度节奏**：内部 dogfooding（2 周）→ 5% 用户 → 25% → 50% → 100%，每档观察 ≥3 天；
2. **关键监控指标**：崩溃率、启动失败率、后端启动失败率、数据迁移成功率、WebView 渲染异常率、内存 P95；
3. **回滚开关**：更新 feed 保留 Electron 通道，异常时可整体回切；
4. **数据迁移守门**：迁移失败自动回退到 Electron 版并上报；
5. **Electron 代码下线**：确认 100% 流量在 Tauri 且稳定 4 周后，删除 `electron*` 依赖与 `process/` 旧实现。

**验收标准**
- Tauri 版崩溃率 ≤ Electron 版基线；
- 数据迁移成功率 ≥99.9%；
- 包体积、内存、启动时间达成 §0 的收益目标。

---

## 4. 潜在技术风险、依赖兼容性问题及应对策略

### 4.1 高风险项（可能导致方案调整）

#### R1 — Linux WebKitGTK 渲染兼容性 🔴

**描述**：Tauri 使用系统 WebView。macOS(WKWebView) 与 Windows(WebView2/Chromium) 风险可控，但 **Linux WebKitGTK 与 Chromium 差异显著**，且各发行版版本碎片化严重。本项目重度依赖 Monaco、CodeMirror 6、mermaid 11、katex、react-virtuoso、Shadow DOM、CSS 自定义主题——恰是 WebKitGTK 最容易出问题的领域。

**影响**：Linux 端功能残缺或性能不可接受。

**对策**：
1. **阶段 2 第 1 周即做兼容性冒烟**，不得后置——这是整个方案的"早期熔断点"；
2. 建立最低版本门槛（如 webkit2gtk ≥ 2.44）并在安装器检测；
3. 分级降级：mermaid/katex 渲染失败时回退为代码块展示；Monaco 失败时回退 CodeMirror；
4. **兜底预案**：若 Linux 阻塞性问题无法解决，Linux 端改为 **Tauri + 打包 Chromium**（如 `wry` 的 servo/CEF 探索分支）或**保留 Electron 仅供 Linux**，macOS/Windows 走 Tauri。这会牺牲"单一技术栈"，但保住主流平台收益。

#### R2 — 自动更新体系重建 🔴

**描述**：现有基于 electron-updater + 自研 `cdnGenericProvider` + `updateFeed`，含 CDN 回退、安装失败诊断（`installerLastFailure.ts`）、Windows 安装器自锁处理（`smoke-installer-self-lock.js`、`smoke-installer-rstrtmgr-ui.js`）等大量踩坑积累。Tauri updater 的签名机制（minisign）、更新包格式、Windows 安装器（NSIS/MSI vs Squirrel）**完全不同**。

**影响**：更新链路故障 = 用户永久卡在旧版，是最严重的生产事故类型。

**对策**：
1. **双 feed 并行期**：灰度期间同时维护 Electron 与 Tauri 两条更新通道，互不影响；
2. 新建 minisign 密钥对，私钥进 CI Secret，**建立密钥轮换与灾备文档**；
3. 复刻 CDN 多源回退逻辑到自定义 updater endpoint；
4. 移植现有安装器故障诊断的**用户提示文案与错误码**（`scripts/smoke-installer-failure-messagebox.js` 已有全场景用例，可直接改造复用）；
5. **强制要求**：跨版本升级链路测试（旧 Electron → 新 Tauri → 更新 Tauri）必须在 3 个平台各跑通，方可放量。

#### R3 — 存量用户数据迁移 🔴

**描述**：`migrations.ts` 达 64 KB，且存在 `repairLegacyHandoffSchema` 这类"修复历史损坏 schema"的补丁逻辑，说明线上确实存在脏数据。同时 `recoverCorruptedDatabase.ts` 的存在证明 DB 损坏是已发生的真实问题。

**影响**：用户会话历史丢失——不可接受。

**对策**：
1. 迁移前**强制备份**原 DB 到 `backup/` 并保留 N 个版本；
2. 迁移采用**事务 + 校验和**，失败自动回滚且回落到 Electron 版；
3. 提供 `aioncore migrate --from-electron <path> --dry-run` 供支持团队排障；
4. 用真实脏数据样本（从社区 issue 收集）构建迁移测试集；
5. 迁移成功率纳入灰度放量的**硬性门禁**（≥99.9%）。

### 4.2 中风险项

#### R4 — 桌宠（Pet）多窗口能力 🟡

透明 + 置顶 + 鼠标穿透 + 空闲计时（`petIdleTicker`）+ 状态机（`petStateMachine`）+ 确认弹窗（`petConfirmManager`）。Tauri 支持 `transparent` / `always_on_top` / `set_ignore_cursor_events`，但 **Linux 下透明窗口与合成器（Wayland vs X11）强相关**，行为不稳定。

**对策**：桌宠定级 P2；Linux 下检测合成器能力，不支持则隐藏该功能入口（功能降级而非崩溃）。

#### R5 — GPU 恢复与 Chromium 调参能力丧失 🟡

`gpuRecovery.ts` 与 `configureChromium.ts` 在 Electron 下可传 Chromium flags 处理花屏/黑屏。Tauri 下 WebView 由系统管理，**此能力不存在**。

**对策**：改为 WebView 崩溃事件监听 + 自动重载 + 用户可见提示；在 FAQ 中提供系统级 WebView 故障排查指引（如 Windows 重装 WebView2 Runtime）。

#### R6 — Sentry 遥测链路重建 🟡

`sentry.ts`（548 行）+ `@sentry/electron/preload` 的 IPC 转发被显式用于避免 `sentry-ipc://` 走 fetch 污染 DevTools。Tauri 下需 Rust 侧 `sentry` crate 与前端 `@sentry/browser` 分别接入，并手工关联 release/user/breadcrumb。

**对策**：保持 DSN 与 release 命名规范不变，确保历史数据可对比；灰度期双写验证事件量级一致。

#### R7 — API 契约漂移 🟡

前端 `common/types/`（1,708 行）手写维护，与 `aionui-api-types` 存在双份真相。跨仓、跨语言下极易漂移。

**对策**：阶段 0 即落地 `ts-rs`/`specta` 生成管线，CI 校验生成物无 diff；将 `common/types/` 降级为纯 re-export。

#### R8 — Office 文档处理能力对齐 🟡

`DocumentConverter.ts` 用 mammoth（docx→html）+ xlsx-republish。Rust 侧 `calamine`（Excel 读）+ `rust_xlsxwriter`（Excel 写）已在 AionCore，但 **docx→HTML 的 Rust 生态成熟度明显低于 mammoth**，PPT 解析亦然。

**对策**：
1. 优先评估 `docx-rs` / `dotext` / `ooxml` 系列，做样张比对（表格、图片、样式、中文字体）；
2. 若质量不达标，**允许 `aionui-office` 内部调用 `aionui-runtime` 托管的 Node 执行 mammoth**（AionCore 已具备 Managed Node 能力，这是现成逃生舱）；
3. 文档转换属非核心路径，可接受渐进替换，不阻塞主线。

### 4.3 低风险项

| # | 风险 | 对策 |
|---|---|---|
| R9 | `sharp` 图像处理 | 仅用于 `fileIcon.ts` 单点，`image` crate 完全覆盖 |
| R10 | `croner` 定时 | AionCore 已有 `cron 0.15` + `chrono-tz`，无需迁移 |
| R11 | JWT / bcrypt / CSRF | AionCore 已有 `jsonwebtoken 9` + `bcrypt 0.17`；**注意 bcrypt cost 参数必须与 bcryptjs 一致，否则存量密码校验失败** |
| R12 | `web-tree-sitter` | 已声明但代码中零引用，直接移除 |
| R13 | Vite/UnoCSS/Arco 构建链 | 阶段 1-4 完全不变，仅阶段 4 后 Node 退化为纯构建期依赖 |
| R14 | E2E 测试栈 | Playwright 支持 Tauri（通过 `tauri-driver`/WebDriver）；阶段 0 已将用例改为外壳无关，成本可控 |
| R15 | `oauth2 5.0.0-rc.1` 为 RC 版 | 上游 AionCore 既有选择，监控其正式版发布；非本次迁移引入 |

### 4.4 非技术风险

| # | 风险 | 对策 |
|---|---|---|
| N1 | **上游高速迭代**（AionUi 5,827 commits，AionCore 1,261 commits） | 迁移分支需高频 rebase；建议**分阶段向上游提 PR**（尤其阶段 1 的去 Electron 化重构对上游本身就有价值），避免长期分叉 |
| N2 | 许可证 | AionUi = Apache-2.0，AionCore = MIT，均宽松兼容；新引入 crate 需过 `cargo-deny` 审计（禁 GPL/AGPL） |
| N3 | 团队 Rust 能力 | 阶段 0-1 不需要 Rust，可作为学习窗口；阶段 2 起需 ≥2 名具备 Tauri 经验的工程师 |
| N4 | 社区插件生态 | `aionui-extension` 的第三方扩展若依赖 Electron API，需提供兼容层或迁移指引 |

---

## 5. 决策建议

### 5.1 推荐路径

**执行阶段 0 → 5，前端保留 React，AionCore 复用不重写。**

理由：
1. 后端 Rust 化已由上游完成，重复投入无收益；
2. renderer 零 Electron 依赖 + 已有平台抽象层，是罕见的低摩擦迁移条件；
3. 核心收益（体积、内存、启动、去 Node）在阶段 2-4 即可 90% 兑现；
4. 前端 Rust 化（Leptos/Dioxus）因缺乏 Arco 等价组件库、Monaco/mermaid 无替代品，ROI 显著为负。

### 5.2 明确不建议做的事

| 反模式 | 原因 |
|---|---|
| 重写 AionCore | 上游 24 crate 已生产验证，重写是纯负收益 |
| 用 Leptos/Dioxus 重写 10.2 万行前端 | 阻塞于组件库与编辑器生态，成本 >6 人月且质量倒退 |
| 阶段 2 直接采用 AionCore 进程内库（方案 B） | 先 sidecar 保证快速可用，进程内化作为优化项 |
| 跳过阶段 1 直接上 Tauri | 79 处宿主全局引用会在三端反复爆炸，且无法双栈并行验证 |
| 把 Linux 兼容性验证放到阶段 3 | 这是全案最大不确定性，必须最早熔断 |

### 5.3 关键里程碑与门禁

| 里程碑 | 时点 | 门禁条件（不满足则暂停/调整方案） |
|---|---|---|
| M1 契约冻结 | 第 2 周末 | 类型生成管线可用，E2E 外壳无关化完成 |
| M2 渲染层解耦 | 第 5 周末 | renderer 宿主引用收敛至 1 处，Electron 版无回归 |
| **M3 三端 WebView 冒烟** | **第 6 周末** | **Linux 无阻塞性渲染问题 → 否则触发 R1 兜底预案** |
| M4 Tauri MVP | 第 9 周末 | 三端可完成完整对话 + 文件预览，体积 <40MB |
| M5 能力对齐 | 第 14 周末 | 功能清单 100%，数据迁移演练通过 |
| M6 去 Node | 第 17 周末 | web-host/web-cli 删除，Docker 镜像瘦身达标 |
| M7 全量发布 | 第 21 周末 | 崩溃率 ≤ 基线，迁移成功率 ≥99.9% |

### 5.4 预期收益量化

| 指标 | Electron 现状（估） | Tauri 目标 | 改善 |
|---|---|---|---|
| 安装包体积 | ~190 MB | ~25 MB | **−87%** |
| 常驻内存（空闲 RSS） | ~420 MB | ~150 MB | **−64%** |
| 冷启动时间 | ~2.8 s | ~1.0 s | **−64%** |
| 运行期 Node 依赖 | 必需 | 无 | 彻底消除 |
| Docker 镜像 | node-base | distroless | **−70%+** |
| 技术栈语言数 | TS + Rust | Rust +（前端）TS | 后端/外壳统一 |

> 上述数值为基于 Tauri 社区典型数据的估算，**必须以阶段 0 采集的真实基线为准复核**。

---

## 附录 A — 本地仓库位置

```
/Users/pengxiangzeng/Downloads/rust-aion-ui/
├── upstream-AionUi/      # Electron 前端仓（--depth 1）
├── upstream-AionCore/    # Rust 后端仓（--depth 1）
└── docs/
    └── AionUi-Rust-迁移方案.md   # 本文档
```

## 附录 B — 关键源码索引

| 主题 | 路径 |
|---|---|
| HTTP/WS 桥（迁移核心契约） | `upstream-AionUi/packages/desktop/src/common/adapter/httpBridge.ts` |
| IPC 桥（待瘦身） | `upstream-AionUi/packages/desktop/src/common/adapter/ipcBridge.ts` |
| 平台抽象层 | `upstream-AionUi/packages/desktop/src/common/platform/IPlatformServices.ts` |
| preload API 全集（81 行） | `upstream-AionUi/packages/desktop/src/preload/main.ts` |
| aioncore 二进制解析 | `upstream-AionUi/packages/desktop/src/process/backend/binaryResolver.ts` |
| 后端进程生命周期（36KB） | `upstream-AionUi/packages/web-host/src/backend-launcher.ts` |
| 静态服务器 + 反代 | `upstream-AionUi/packages/web-host/src/static-server.ts` |
| 遗留 DB 迁移（64KB） | `upstream-AionUi/packages/desktop/src/process/services/database/migrations.ts` |
| Rust 后端架构文档 | `upstream-AionCore/ARCHITECTURE.md` / `ARCHITECTURE.zh-CN.md` |
| Rust 依赖全集 | `upstream-AionCore/Cargo.toml` |
| 官方开发环境搭建 | `upstream-AionUi/docs/contributing/development.md` |
| 官方目录规范 | `upstream-AionUi/docs/contributing/file-structure.md` |
