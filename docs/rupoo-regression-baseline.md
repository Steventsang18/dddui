# rupoo 内核能力回归基准

> 状态：2026-08-04 建立。RoseUi 内置 agent 内核 = rupoo 引擎
> （`/Users/pengxiangzeng/rust-project/src-agent`）。本文档为可复现的能力回归基准，
> 用于每次 `prompt.default.toml` / 工具集改动后，确认四大工程化能力未被破坏。

## 四大工程化能力映射

| 能力 | rupoo 工具 | 验证方式 |
|------|-----------|---------|
| 理解工程结构（先映射再改） | `list_directory` / `code_search` | smoke 测试 step1 + `rig_tools` 单测 |
| 框架约定（不引新依赖） | `file_read` / `file_edit` | smoke 测试 step2 |
| 编译测试（改动后自验） | `run_tests` / `diff_check` | smoke 测试 step3 + `verify` 单测 |
| 多文件协同（定点编辑） | `file_edit`(replace_all) | `rig_tools` 单测 |

## 运行命令（零外部依赖）

```bash
cd /Users/pengxiangzeng/rust-project/src-agent
cargo test --test smoke_edit_build_test      # 核心闭环：定位→编辑→编译测试
cargo test                                   # 全量（含 file_read/list_directory/diff_check 单测）
```

真实 LLM 任务跑分（需 `RUPOO_BENCH=1` + 已构建二进制 + 任务目录）：
```bash
RUPOO_BENCH=1 cargo test -p rupoo --test terminal_bench -- --ignored
```

## 2026-08-04 基准结果

- `cargo test --test smoke_edit_build_test`：**1 passed**（编译 43.84s + 跑 0.52s）
- `cargo test` 全量：**386 + 86 + 4 + 1 + 3 + 12 = 492 passed**，3 ignored（LLM 真实任务）。
- 四大能力均覆盖，无回归。

## 红线

- `rupoo` crate 名 / 内部标识符**不改**（合规要求）。
- 仅改 `prompt.default.toml`（system prompt 内容）与工具集行为，改动后须重跑本基准
  并重新编译 `rosecore`（`cargo build -p roseui-app`）使其生效。
- system prompt 加载链：`$RUPOO_HOME/prompt.toml` > `prompt.default.toml` >
  `include_str!("../prompt.default.toml")`。改 `prompt.default.toml` 即生效路径。
