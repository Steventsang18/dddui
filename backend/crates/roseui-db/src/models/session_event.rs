//! 对话轨迹事件（session_events 表）的数据模型与密钥脱敏工具。
//!
//! session_events 是 append-only 可审计轨迹，与 `messages`（展示用）解耦。
//! 写入前必须对 input_json / output_json / token_usage_json 做密钥脱敏，绝不落明文凭据。

use serde_json::{Value, json};

/// 单条轨迹事件。字段与 migration `042_session_events.sql` 一一对应。
#[derive(Debug, Clone)]
pub struct SessionEventRow {
    pub id: String,
    pub conversation_id: String,
    pub turn_seq: i64,
    /// `text` | `thinking` | `tool_call` | `model_call`
    pub event_kind: String,
    pub role: Option<String>,
    pub model: Option<String>,
    pub input_json: String,
    pub output_json: String,
    pub token_usage_json: String,
    pub status: Option<String>,
    pub created_at: i64,
}

/// 被脱敏字段的占位值。
const REDACTED: &str = "***";

/// 敏感字段名子串表（小写包含匹配）。
const SENSITIVE_KEY_SUBSTRINGS: &[&str] = &[
    "password",
    "passwd",
    "pwd",
    "secret",
    "token",
    "authorization",
    "cookie",
    "credential",
    "api_key",
    "apikey",
    "access_key",
    "private_key",
];

/// 凭据前缀：整串值以这些前缀开头时，保留前缀、遮蔽主体。
const KEY_PREFIXES: &[&str] = &["sk-", "sk_", "ghp_", "github_pat_", "xoxb-", "xoxp-", "AKIA"];

fn is_sensitive_key(key: &str) -> bool {
    let k = key.to_ascii_lowercase();
    if k == "key" {
        return true;
    }
    SENSITIVE_KEY_SUBSTRINGS.iter().any(|s| k.contains(s))
}

fn redact_string(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return value.to_string();
    }
    if trimmed.starts_with("Bearer ") {
        return format!("Bearer {REDACTED}");
    }
    for prefix in KEY_PREFIXES {
        if trimmed.starts_with(prefix) {
            return format!("{prefix}{REDACTED}");
        }
    }
    value.to_string()
}

/// 递归脱敏 JSON：敏感字段名 → 值整体替换 REDACTED；普通字段递归；字符串值过前缀遮蔽。
pub fn redact_json(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, v) in map {
                if is_sensitive_key(k) {
                    out.insert(k.clone(), Value::String(REDACTED.to_string()));
                } else {
                    out.insert(k.clone(), redact_json(v));
                }
            }
            Value::Object(out)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(redact_json).collect()),
        Value::String(s) => Value::String(redact_string(s)),
        other => other.clone(),
    }
}

/// 把任意可序列化值脱敏后转字符串（失败回退空对象）。
pub fn redact_to_string<T: serde::Serialize>(value: &T) -> String {
    match serde_json::to_value(value) {
        Ok(v) => redact_json(&v).to_string(),
        Err(_) => "{}".to_string(),
    }
}

/// 构造一条文本事件（output 已脱敏）。
#[allow(clippy::too_many_arguments)]
pub fn text_event(
    id: String,
    conversation_id: String,
    turn_seq: i64,
    role: Option<String>,
    model: Option<String>,
    content: &str,
    status: Option<String>,
    created_at: i64,
) -> SessionEventRow {
    SessionEventRow {
        id,
        conversation_id,
        turn_seq,
        event_kind: "text".to_string(),
        role,
        model,
        input_json: "{}".to_string(),
        output_json: redact_json(&json!({ "content": content })).to_string(),
        token_usage_json: "{}".to_string(),
        status,
        created_at,
    }
}
