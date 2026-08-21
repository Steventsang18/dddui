# DoDidDoneUi 桌面端（Tauri 2）

轻量桌面壳：系统 WebView（macOS 为 WKWebView）+ sidecar 托管的 Rust 后端单二进制。
壳层本身约 10MB，不捆绑浏览器内核，前端复用 `frontend/` 的 React 构建产物。

## 架构

```
DoDidDoneUi.app
 ├─ Tauri 壳（本工程 src-tauri）
 │    ├─ 启动 dodiddoneui sidecar（--local --parent-pid，随机 loopback 端口）
 │    ├─ 轮询 /api/auth/status 判定就绪
 │    └─ 创建窗口并注入 window.__backendPort（前端 httpBridge 走桌面直连模式）
 ├─ 前端 dist（frontend/packages/desktop/src/renderer/dist，tauri://localhost 加载）
 └─ dodiddoneui sidecar（backend 单二进制，externalBin 打包）
```

数据目录：`~/Library/Application Support/com.dodiddoneui.desktop`（与 Web 形态的 `backend/data` 相互独立）。

## 开发

前置：Rust、Node ≥ 22、`cargo install tauri-cli`、先构建过一次后端。

```bash
# 一键开发模式（后端 release 构建 + sidecar 同步 + tauri dev + vite HMR）
./desktop/build.sh --dev

# 后端没改动时跳过重建
./desktop/build.sh --dev --skip-backend
```

## 打包

```bash
./desktop/build.sh          # 产出 .app / .dmg：desktop/src-tauri/target/release/bundle/
```

一期不做签名与公证，本机可直接运行；分发需 Developer ID 签名 + notarize（二期接 CI）。

## 约束

- 桌面专属逻辑全部收在本目录，backend 各 crate 禁止引入 tauri 依赖（为未来进程内嵌演进保留空间）。
- sidecar 崩溃自动重启上限 3 次；正常退出由 `--parent-pid` 兜底防孤儿进程。
- `--local` 模式免认证，后端仅监听 127.0.0.1 随机端口；本机其他进程理论可访问，彻底隔离需后端实例级 token（二期）。
