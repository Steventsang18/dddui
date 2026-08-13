# DoDidDoneUi (DDDUI) 发布打包流程与 Release 检查清单

本文档面向**发布形态（:3080 单二进制）**：编译产物 `dodiddoneui` 通过 rust-embed 内嵌前端 `dist/`，启动即完整应用，用户无需安装 Node、无需起 Vite。

> 形态对比：
> - `:3080` = **生产/用户态**（单 Rust 二进制，内嵌前端，打开即用）→ 发布用它。
> - `:5173` = **开发态**（Vite dev server，热更新，仅开发者用）→ 发布不用。

---

## 一、标准发布打包流程

### 前置依赖（一次性）
- Rust toolchain `1.95.0`（`rustup`）
- Node.js `>=22 <25` + npm
- macOS / Linux 构建机（Windows 另需 `windows-sys` 链路，见 `roseui-app/Cargo.toml`）

### 步骤（推荐直接跑一键脚本）
在**仓库根目录**执行：

```bash
# release 构建（前端 build → dist 内嵌 → cargo build --release）
./build.sh

# 或调试构建（编译快、运行慢，仅自测用）
./build.sh --debug
```

`build.sh` 实际做的事（`scripts/build-binary.sh`）：
1. `cd frontend && npm run build` —— 生成 `frontend/packages/desktop/src/renderer/dist/`
2. 拷贝 `dist/` → `backend/crates/roseui-webhost/assets/`（rust-embed 内嵌点，旧文件先清空）
3. `cd backend && cargo build --release` —— 产出 `backend/target/release/dodiddoneui`

### 产物位置
- 二进制：`backend/target/release/dodiddoneui`
- 内嵌前端：已编译进二进制（无需单独分发 `dist/`）

---

## 二、启动验证 :3080（发布形态）

发布形态必须用**编译产物内嵌**的方式启动，**不要**用 `start-roseui.sh`（它当前指向 `target/debug` 并 `--static-dir` 读磁盘 dist，是开发态）。

```bash
cd backend

# 确保数据目录固定，避免配置漂移（同 start-roseui.sh 的固化逻辑）
export JWT_SECRET="<与历史库一致的密钥，或留空让首启自动生成并持久化>"

./target/release/dodiddoneui \
  --port 3080 \
  --host 127.0.0.1 \
  --identity-mode owner \
  --data-dir "$(pwd)/data"
```

验证：

```bash
# 健康检查（注意是 /health，不是 /api/health）
curl -fsS http://127.0.0.1:3080/health && echo " OK"

# 前端可访问
curl -fsS http://127.0.0.1:3080/ | head -n 5
```

浏览器打开 `http://127.0.0.1:3080/#/guid` 正常进入即成功。

> 退出：生产形态点界面「系统设置 → Exit」按钮（先回 200 再 `process::exit`）；或 `lsof -ti :3080 | xargs kill`。
> 重启铁律：旧进程占 `*.flock`，新实例会 `PEER_ALREADY_RUNNING` 退让 → 先 kill 旧 PID 再起。

---

## 三、Release 检查清单

### 构建前
- [ ] 工作树干净 / 已切到待发布 commit（`git status`）
- [ ] 已从 `frontend/` 正确目录 build（dist 非陈旧）
- [ ] `frontend/node_modules` 存在（否则 `build.sh` 会自动 `npm install`）

### 构建中
- [ ] `npm run build` 无报错（约 15s，非挂起）
- [ ] `backend/crates/roseui-webhost/assets/` 已更新（dist 拷贝成功）
- [ ] `cargo build --release` 无 error / 无新增 clippy 告警
- [ ] 产物存在：`backend/target/release/dodiddoneui`

### 安全（最高约束 `docs/编译遵守原则.md`）
- [ ] 默认绑 `127.0.0.1`（**禁止 0.0.0.0**，Owner 模式无登录墙，绑公网=任何人可用）
- [ ] 无 `Allow-Origin: *`、保留 JWT + CSRF、WS 握手校验 Origin
- [ ] `JWT_SECRET` 不入仓库（仅 `backend/.env`，已被 gitignore；模板见 `.env.example`）
- [ ] 仓库历史无明文密钥（已 filter-repo 清理；发布前再 `git log -p | grep` 抽查）
- [ ] 遥测默认关闭（feature gate）

### 运行验证
- [ ] `:3080/health` 返回 200
- [ ] 前端 `:3080/#/guid` 可正常进入、对话/模型配置可用
- [ ] 已配置模型的 API Key 重启后仍可解密（依赖固定 `JWT_SECRET` + 固化 `--data-dir`）
- [ ] 退出按钮在生产态可一键关闭后端

### 发布分发
- [ ] 仅分发 `dodiddoneui` 二进制 + `data/` 初始化逻辑（前端已内嵌）
- [ ] 配套 `start-roseui.sh` 或等价启动脚本（固化 `--data-dir` 与 `JWT_SECRET`）
- [ ] README 写明启动命令与默认地址 `127.0.0.1:3080`
- [ ] LICENSE + NOTICE + 修改头注（`Modified by ... on <date>`）就位

---

## 四、常见问题

| 现象 | 原因 | 解决 |
|------|------|------|
| 前端白屏/旧页面 | dist 未重新内嵌 | 重跑 `./build.sh`，确认 `webhost/assets/` 已更新 |
| 启动即退出 `PEER_ALREADY_RUNNING` | 旧进程占 `*.flock` | `lsof -ti :3080 | xargs kill` 后重试 |
| API Key 重启后失效 | `JWT_SECRET` 变化或 `--data-dir` 漂移 | 固化 `--data-dir` + 固定 `JWT_SECRET`（用 `start-roseui.sh` 逻辑） |
| `:5173` 能进、`:3080` 进不去 | 3080 是内嵌形态，需先 build 再起二进制 | 按本文第二节用 release 二进制启动 |
