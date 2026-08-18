//! 轨迹页「步骤时间线」聚合器（L2）。
//!
//! 把原始 session_events 流式碎片 + 用户提问，聚合成对小白友好的轮次步骤：
//! - 以「用户提问」为锚切分轮次；无提问的历史数据按时间间隙切轮。
//! - 连续 `text` 流式碎片合并为一条「回答」。
//! - `tool_call` 映射为中文动作（读取文件/搜索网页…）+ 关键参数 + 状态。
//! - `model_call` 展示模型名 + 耗时。
//!
//! 该结构**只供前端新时间线使用**；`data`（原始事件）与矩阵展示完全不变。

use roseui_db::models::SessionEventRow;
use roseui_db::UserQuestionRow;
use serde_json::{Value, json};

/// 工具中文名映射：(协议名, 中文动作, 图标)。
const TOOL_META: &[(&str, &str, &str)] = &[
    ("file_read", "读取文件", "📖"),
    ("read_file", "读取文件", "📖"),
    ("file_write", "写入文件", "✍️"),
    ("write_file", "写入文件", "✍️"),
    ("file_edit", "编辑文件", "✏️"),
    ("edit_file", "编辑文件", "✏️"),
    ("file_delete", "删除文件", "🗑️"),
    ("delete_file", "删除文件", "🗑️"),
    ("directory_list", "浏览目录", "📂"),
    ("list_directory", "浏览目录", "📂"),
    ("list_dir", "浏览目录", "📂"),
    ("list_files", "浏览目录", "📂"),
    ("directory_tree", "浏览目录树", "🌲"),
    ("project_tree", "浏览项目结构", "🌲"),
    ("directory_create", "创建目录", "📁"),
    ("grep", "搜索内容", "🔎"),
    ("web_search", "搜索网页", "🔍"),
    ("search_web", "搜索网页", "🔍"),
    ("web_fetch", "抓取网页", "🌐"),
    ("fetch_url", "抓取网页", "🌐"),
    ("http_request", "请求 API", "🌐"),
    ("shell", "执行命令", "⌘"),
    ("run_shell", "执行命令", "⌘"),
    ("bash", "执行命令", "⌘"),
    ("terminal", "终端命令", "⌘"),
    ("code_interpreter", "运行代码", "▶️"),
    ("python", "运行代码", "▶️"),
    ("apply_patch", "应用补丁", "🩹"),
    ("search_files", "搜索文件", "🔎"),
    ("glob", "搜索文件", "🔎"),
    ("github_search", "搜索 GitHub", "🐙"),
];

fn tool_meta(name: &str) -> (&'static str, &'static str) {
    for (n, cn, icon) in TOOL_META {
        if *n == name {
            return (cn, icon);
        }
    }
    ("调用工具", "🔧")
}

/// 从对象里取第一个非空字符串字段。
fn pick_str(v: &Value, keys: &[&str]) -> Option<String> {
    let obj = v.as_object()?;
    for k in keys {
        if let Some(s) = obj.get(*k).and_then(|x| x.as_str()) {
            let t = s.trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
    }
    None
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out = String::new();
        let mut n = 0;
        for c in s.chars() {
            if n >= max {
                break;
            }
            out.push(c);
            n += 1;
        }
        format!("{out}…")
    }
}

/// 工具结果的可读摘要（截断）。
fn summarize_output(v: &Value) -> String {
    if let Some(s) = pick_str(v, &["output", "content", "result", "message", "error", "text"]) {
        return truncate(&s, 160);
    }
    truncate(&serde_json::to_string(v).unwrap_or_default(), 160)
}

/// 工具动作的一句话：中文动作 + 关键参数。
fn summary_for_tool(name: &str, args: &Value) -> String {
    let (cn, _) = tool_meta(name);
    let target = pick_str(args, &["path", "file_path", "filename", "file"])
        .map(|p| format!(" {p}"))
        .or_else(|| pick_str(args, &["query", "search", "keyword"]).map(|q| format!(" “{q}”")))
        .or_else(|| pick_str(args, &["command", "cmd"]).map(|c| format!(" {c}")))
        .or_else(|| pick_str(args, &["url"]).map(|u| format!(" {u}")))
        .or_else(|| pick_str(args, &["tool", "tool_name"]).map(|t| format!(" {t}")))
        .unwrap_or_default();
    format!("{cn}{target}")
}

/// 解析事件 JSON，失败回退空对象。
fn parse_json(s: &str) -> Value {
    serde_json::from_str::<Value>(s).unwrap_or_else(|_| Value::Object(Default::default()))
}

/// 单个时间线步骤。
struct Step {
    kind: &'static str,
    time: i64,
    ids: Vec<String>,
    payload: Value,
}

/// 一轮对话（提问为锚）。
struct Turn {
    turn: usize,
    time: i64,
    question: Option<(String, String, i64)>, // (id, content, created_at)
    steps: Vec<Step>,
}

/// 把事件列表 + 用户提问聚合成轮次时间线。
pub fn build_timeline(events: &[SessionEventRow], questions: &[UserQuestionRow]) -> Value {
    let mut evs: Vec<&SessionEventRow> = events.iter().collect();
    evs.sort_by_key(|e| e.created_at);
    let mut qs: Vec<&UserQuestionRow> = questions.iter().collect();
    qs.sort_by_key(|q| q.created_at);

    // ---- 切分轮次：以提问为锚；无提问则整段算一轮 ----
    let mut bounds: Vec<(i64, Option<&UserQuestionRow>)> = Vec::new();
    if qs.is_empty() {
        if let Some(first) = evs.first() {
            bounds.push((first.created_at, None));
        }
    } else {
        for (i, q) in qs.iter().enumerate() {
            let start = if i == 0 {
                evs.first().map(|e| e.created_at).unwrap_or(q.created_at)
            } else {
                q.created_at
            };
            bounds.push((start, Some(*q)));
        }
    }

    let mut turns: Vec<Turn> = Vec::new();
    for (bi, (start, q)) in bounds.iter().enumerate() {
        let end = bounds.get(bi + 1).map(|(s, _)| *s).unwrap_or(i64::MAX);
        let mut turn = Turn {
            turn: bi + 1,
            time: *start,
            question: q.map(|qq| (qq.id.clone(), qq.content.clone(), qq.created_at)),
            steps: Vec::new(),
        };
        for e in evs.iter().filter(|e| e.created_at >= *start && e.created_at < end) {
            push_step(&mut turn, e);
        }
        merge_text_steps(&mut turn);
        if !turn.steps.is_empty() || turn.question.is_some() {
            turns.push(turn);
        }
    }

    json!(turns
        .into_iter()
        .map(|t| {
            json!({
                "turn": t.turn,
                "time": t.time,
                "question": t.question.as_ref().map(|(_, c, _)| c.clone()),
                "question_id": t.question.as_ref().map(|(id, _, _)| id.clone()),
                "steps": t.steps.into_iter().map(|s| s.payload).collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>())
}

fn push_step(turn: &mut Turn, e: &SessionEventRow) {
    let time = e.created_at;
    let id = e.id.clone();
    let input = parse_json(&e.input_json);
    let output = parse_json(&e.output_json);
    let status = e.status.clone().unwrap_or_default();

    let (kind, payload): (&'static str, Value) = match e.event_kind.as_str() {
        "tool_call" => {
            let name = pick_str(&input, &["name"]).unwrap_or_default();
            let args = input.get("args").cloned().unwrap_or(Value::Object(Default::default()));
            let (cn, icon) = tool_meta(&name);
            let status_cls = match status.as_str() {
                "finish" | "done" | "completed" => "done",
                "error" | "failed" => "err",
                _ => "run",
            };
            let status_text = match status_cls {
                "done" => "完成",
                "err" => "出错",
                _ => "进行中",
            };
            let summary = summary_for_tool(&name, &args);
            (
                "tool_call",
                json!({
                    "kind": "tool_call",
                    "time": time,
                    "ids": [id],
                    "name": name,
                    "cn": cn,
                    "icon": icon,
                    "args": args,
                    "output": summarize_output(&output),
                    "status": status,
                    "status_text": status_text,
                    "status_cls": status_cls,
                    "summary": summary,
                    "raw_input": input,
                    "raw_output": output,
                }),
            )
        }
        "model_call" => {
            let model = e.model.clone().unwrap_or_default();
            let duration_ms = parse_json(&e.token_usage_json)
                .get("duration_ms")
                .and_then(|d| d.as_i64())
                .unwrap_or(0);
            let prompt = pick_str(&input, &["prompt", "query", "message", "content", "backend"])
                .unwrap_or_else(|| truncate(&serde_json::to_string(&input).unwrap_or_default(), 120));
            let summary = if !prompt.is_empty() {
                format!("调用模型 {model}：{prompt}")
            } else {
                format!("调用模型 {model}")
            };
            (
                "model_call",
                json!({
                    "kind": "model_call",
                    "time": time,
                    "ids": [id],
                    "model": model,
                    "duration_ms": duration_ms,
                    "prompt": prompt,
                    "summary": summary,
                    "raw_input": input,
                    "raw_output": output,
                }),
            )
        }
        "thinking" => {
            let content = pick_str(&output, &["content"]).unwrap_or_default();
            (
                "thinking",
                json!({
                    "kind": "thinking",
                    "time": time,
                    "ids": [id],
                    "content": content,
                    "summary": truncate(&content, 160),
                }),
            )
        }
        _ => {
            // text：暂存，稍后合并为回答。
            let content = pick_str(&output, &["content"]).unwrap_or_default();
            if content.is_empty() {
                return;
            }
            (
                "text",
                json!({
                    "kind": "text",
                    "time": time,
                    "ids": [id],
                    "content": content,
                }),
            )
        }
    };
    turn.steps.push(Step {
        kind,
        time,
        ids: vec![id],
        payload,
    });
}

/// 把同一轮里相邻的 text 碎片合并成「回答」步骤（间隔 > 60s 则切开）。
fn merge_text_steps(turn: &mut Turn) {
    const GAP_MS: i64 = 60_000;
    let mut out: Vec<Step> = Vec::with_capacity(turn.steps.len());
    for st in std::mem::take(&mut turn.steps) {
        if st.kind != "text" {
            out.push(st);
            continue;
        }
        let can_merge = out
            .last()
            .is_some_and(|last| last.kind == "answer" && st.time - last.time < GAP_MS);
        if can_merge {
            let last = out.last_mut().unwrap();
            let extra = st
                .payload
                .get("content")
                .and_then(|c| c.as_str())
                .unwrap_or_default()
                .to_string();
            last.ids.extend(st.ids.iter().cloned());
            if let Some(obj) = last.payload.as_object_mut() {
                let cur = obj
                    .get("content")
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_string();
                let merged = if cur.is_empty() {
                    extra
                } else {
                    format!("{cur}\n\n{extra}")
                };
                obj.insert("content".to_string(), json!(merged));
            }
        } else {
            let content = st
                .payload
                .get("content")
                .and_then(|c| c.as_str())
                .unwrap_or_default()
                .to_string();
            out.push(Step {
                kind: "answer",
                time: st.time,
                ids: st.ids.clone(),
                payload: json!({
                    "kind": "answer",
                    "time": st.time,
                    "ids": st.ids,
                    "content": content,
                }),
            });
        }
    }
    // 修正 parts 计数
    for st in out.iter_mut() {
        if st.kind == "answer" {
            if let Some(obj) = st.payload.as_object_mut() {
                obj.insert("parts".to_string(), json!(st.ids.len()));
            }
        }
    }
    turn.steps = out;
}
