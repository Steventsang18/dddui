//! Supervisor audit logger — SQLite-backed audit event storage.
//!
//! Uses a dedicated `audit_events` table (not the `settings` key-value store)
//! for efficient SQL-level filtering and TTL-based cleanup.

use crate::error::{AgentError, AgentResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 审计事件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditEventType {
    ComplianceCheck,
    ConfidenceCheck,
    CircuitBreakerCheck,
    ActionApproved,
    ActionBlocked,
    ActionPaused,
    ToolCall,
    ToolResult,
    GoalParsed,
    PlanSelected,
    ReplanTriggered,
    TaskCompleted,
}

impl AuditEventType {
    /// Canonical string representation for DB storage.
    fn as_str(&self) -> &'static str {
        match self {
            Self::ComplianceCheck => "ComplianceCheck",
            Self::ConfidenceCheck => "ConfidenceCheck",
            Self::CircuitBreakerCheck => "CircuitBreakerCheck",
            Self::ActionApproved => "ActionApproved",
            Self::ActionBlocked => "ActionBlocked",
            Self::ActionPaused => "ActionPaused",
            Self::ToolCall => "ToolCall",
            Self::ToolResult => "ToolResult",
            Self::GoalParsed => "GoalParsed",
            Self::PlanSelected => "PlanSelected",
            Self::ReplanTriggered => "ReplanTriggered",
            Self::TaskCompleted => "TaskCompleted",
        }
    }
}

/// 审计结果
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditResult {
    Passed,
    Blocked,
    Paused,
}

impl AuditResult {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Passed => "Passed",
            Self::Blocked => "Blocked",
            Self::Paused => "Paused",
        }
    }
}

/// 全链路审计事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub layer: String,
    pub action_id: String,
    pub actor: String,
    pub detail: serde_json::Value,
    pub result: AuditResult,
}

impl AuditEvent {
    pub fn new(event_type: AuditEventType, layer: &str, detail: &serde_json::Value) -> Self {
        Self {
            timestamp: Utc::now(),
            event_type,
            layer: layer.to_string(),
            action_id: uuid::Uuid::new_v4().to_string(),
            actor: "agent".to_string(),
            detail: detail.clone(),
            result: AuditResult::Passed,
        }
    }

    pub fn new_blocked(event_type: AuditEventType, layer: &str, reason: &str) -> Self {
        let mut event = Self::new(event_type, layer, &serde_json::json!({"reason": reason}));
        event.result = AuditResult::Blocked;
        event
    }
}

/// 审计日志存储 Trait
#[async_trait]
pub trait AuditLogger: Send + Sync {
    async fn record(&self, event: AuditEvent) -> AgentResult<()>;
    async fn query_by_type(
        &self,
        event_type: AuditEventType,
        limit: usize,
    ) -> AgentResult<Vec<AuditEvent>>;
    async fn query_blocked(&self, limit: usize) -> AgentResult<Vec<AuditEvent>>;
    async fn count_events(&self) -> AgentResult<usize>;
    /// Delete audit events older than `max_age_days`.
    async fn cleanup(&self, max_age_days: u32) -> AgentResult<u64>;
}

// ---------------------------------------------------------------------------
// 审计脱敏 (redaction)
// ---------------------------------------------------------------------------
//
// 审计日志比业务数据存活更久、被更多角色访问，且常同步到 SIEM/日志聚合系统，
// 因此绝不能把明文凭据写入 `audit_events`。脱敏只遮蔽「内容」（密钥、口令、
// Token、路径、正文），保留「元数据」（动作类型、结果、时间、actor），
// 从而在不削弱可审计性的前提下最小化泄露面。

/// 被脱敏字段的占位值。
const REDACTED: &str = "***";

/// 字段名敏感子串表：小写后命中任意子串即判定为敏感字段。
///
/// 采用「包含匹配」而非「精确匹配」，以覆盖 `client_secret`、`access_token`
/// 等派生命名；同时刻意**不**把裸 `key` 放进子串表，避免误伤 `keyboard`、
/// `monkey` 等普通字段名。
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

/// 判定字段名是否敏感（大小写不敏感）。
fn is_sensitive_key(key: &str) -> bool {
    let k = key.to_ascii_lowercase();
    // 裸 `key` 精确命中（审计 JSON 里 `key` 通常是密钥本体）。
    if k == "key" {
        return true;
    }
    SENSITIVE_KEY_SUBSTRINGS.iter().any(|s| k.contains(s))
}

/// 常见凭据前缀（用于对「值整体即 Token」的场景做前缀遮蔽）。
const KEY_PREFIXES: &[&str] = &[
    "sk-", "sk_", "ghp_", "github_pat_", "xoxb-", "xoxp-", "AKIA",
];

/// 对字符串值做前缀遮蔽：保留可识别前缀，隐藏机密主体。
///
/// 仅处理「整串以已知前缀开头」的明确场景，不做内嵌正则（易误伤）。
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

/// 递归脱敏 `serde_json::Value`。
///
/// - 字段名敏感 → 值整体替换为 `REDACTED`（无论原值类型）。
/// - 字段名普通 → 递归处理嵌套对象/数组；字符串值过 `redact_string`。
fn redact_value(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, v) in map {
                if is_sensitive_key(k) {
                    out.insert(k.clone(), serde_json::Value::String(REDACTED.to_string()));
                } else {
                    out.insert(k.clone(), redact_value(v));
                }
            }
            serde_json::Value::Object(out)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(redact_value).collect())
        }
        serde_json::Value::String(s) => serde_json::Value::String(redact_string(s)),
        other => other.clone(),
    }
}

/// SQLite 实现的审计日志 — uses dedicated `audit_events` table.
pub struct SqliteAuditLogger {
    repo: std::sync::Arc<crate::db::TaskRepo>,
}

impl SqliteAuditLogger {
    /// Create a logger that opens the default database at `RUPOO_HOME/agent.db`.
    pub fn new() -> AgentResult<Self> {
        let path = crate::config::rupoo_home().join("agent.db");
        let repo = std::sync::Arc::new(
            crate::db::TaskRepo::new(path.to_str().unwrap_or(":memory:"))
                .map_err(|_| AgentError::Config("无法打开审计日志数据库".to_string()))?,
        );
        Ok(Self { repo })
    }

    /// Create a logger backed by an existing TaskRepo (for testing).
    pub fn with_repo(repo: std::sync::Arc<crate::db::TaskRepo>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl AuditLogger for SqliteAuditLogger {
    /// Record an audit event into the `audit_events` table.
    ///
    /// # Preconditions
    /// - The `audit_events` table must exist (created by `TaskRepo::new`).
    ///
    /// # Postconditions
    /// - One row is inserted into `audit_events`.
    async fn record(&self, mut event: AuditEvent) -> AgentResult<()> {
        let event_type = event.event_type.as_str().to_string();
        let result = event.result.as_str().to_string();
        let timestamp = event.timestamp.to_rfc3339();
        // 脱敏：落库前对 detail 里的敏感字段（密钥/token/口令/凭据前缀）做不可逆遮蔽。
        event.detail = redact_value(&event.detail);
        let payload = serde_json::to_string(&event)?;

        self.repo
            .with_conn(move |conn| {
                conn.execute(
                    "INSERT INTO audit_events (event_type, result, timestamp, payload_json)
                     VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![event_type, result, timestamp, payload],
                )?;
                Ok(())
            })
            .await
    }

    /// Query audit events by type, ordered by timestamp descending.
    ///
    /// Uses SQL WHERE clause for efficient filtering (no full-table scan).
    async fn query_by_type(
        &self,
        event_type: AuditEventType,
        limit: usize,
    ) -> AgentResult<Vec<AuditEvent>> {
        let type_str = event_type.as_str().to_string();
        self.repo
            .with_read_conn(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT payload_json FROM audit_events
                     WHERE event_type = ?1
                     ORDER BY timestamp DESC
                     LIMIT ?2",
                )?;
                let rows = stmt.query_map(rusqlite::params![type_str, limit as i64], |row| {
                    row.get::<_, String>(0)
                })?;
                let mut events = Vec::new();
                for row in rows.flatten() {
                    if let Ok(event) = serde_json::from_str::<AuditEvent>(&row) {
                        events.push(event);
                    }
                }
                Ok(events)
            })
            .await
    }

    /// Query blocked audit events, ordered by timestamp descending.
    async fn query_blocked(&self, limit: usize) -> AgentResult<Vec<AuditEvent>> {
        self.repo
            .with_read_conn(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT payload_json FROM audit_events
                     WHERE result = 'Blocked'
                     ORDER BY timestamp DESC
                     LIMIT ?1",
                )?;
                let rows = stmt.query_map(rusqlite::params![limit as i64], |row| {
                    row.get::<_, String>(0)
                })?;
                let mut events = Vec::new();
                for row in rows.flatten() {
                    if let Ok(event) = serde_json::from_str::<AuditEvent>(&row) {
                        events.push(event);
                    }
                }
                Ok(events)
            })
            .await
    }

    /// Count total audit events.
    async fn count_events(&self) -> AgentResult<usize> {
        self.repo
            .with_read_conn(move |conn| {
                let count: i64 =
                    conn.query_row("SELECT COUNT(*) FROM audit_events", [], |row| row.get(0))?;
                Ok(count as usize)
            })
            .await
    }

    /// Delete audit events older than `max_age_days`.
    ///
    /// # Returns
    /// The number of deleted rows.
    async fn cleanup(&self, max_age_days: u32) -> AgentResult<u64> {
        let cutoff = (Utc::now() - chrono::Duration::days(max_age_days as i64)).to_rfc3339();
        self.repo
            .with_conn(move |conn| {
                let deleted = conn.execute(
                    "DELETE FROM audit_events WHERE timestamp < ?1",
                    rusqlite::params![cutoff],
                )?;
                Ok(deleted as u64)
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::TaskRepo;
    use std::sync::Arc;

    fn test_repo() -> Arc<TaskRepo> {
        Arc::new(TaskRepo::new(":memory:").expect("in-memory repo"))
    }

    #[tokio::test]
    async fn test_record_and_count() {
        let logger = SqliteAuditLogger::with_repo(test_repo());
        assert_eq!(logger.count_events().await.unwrap(), 0);

        let event = AuditEvent::new(
            AuditEventType::ToolCall,
            "test-layer",
            &serde_json::json!({"tool": "read_file"}),
        );
        logger.record(event).await.unwrap();
        assert_eq!(logger.count_events().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_query_by_type() {
        let logger = SqliteAuditLogger::with_repo(test_repo());

        // Record events of different types
        logger
            .record(AuditEvent::new(
                AuditEventType::ToolCall,
                "layer",
                &serde_json::json!({}),
            ))
            .await
            .unwrap();
        logger
            .record(AuditEvent::new(
                AuditEventType::ComplianceCheck,
                "layer",
                &serde_json::json!({}),
            ))
            .await
            .unwrap();
        logger
            .record(AuditEvent::new(
                AuditEventType::ToolCall,
                "layer",
                &serde_json::json!({}),
            ))
            .await
            .unwrap();

        let tool_calls = logger
            .query_by_type(AuditEventType::ToolCall, 10)
            .await
            .unwrap();
        assert_eq!(tool_calls.len(), 2, "should find 2 ToolCall events");

        let compliance = logger
            .query_by_type(AuditEventType::ComplianceCheck, 10)
            .await
            .unwrap();
        assert_eq!(compliance.len(), 1, "should find 1 ComplianceCheck event");
    }

    #[tokio::test]
    async fn test_query_blocked() {
        let logger = SqliteAuditLogger::with_repo(test_repo());

        logger
            .record(AuditEvent::new(
                AuditEventType::ActionBlocked,
                "layer",
                &serde_json::json!({}),
            ))
            .await
            .unwrap();
        logger
            .record(AuditEvent::new_blocked(
                AuditEventType::ActionBlocked,
                "layer",
                "forbidden command",
            ))
            .await
            .unwrap();

        let blocked = logger.query_blocked(10).await.unwrap();
        assert_eq!(blocked.len(), 1, "should find 1 blocked event");
        assert_eq!(blocked[0].result, AuditResult::Blocked);
    }

    #[tokio::test]
    async fn test_cleanup_removes_old_events() {
        let logger = SqliteAuditLogger::with_repo(test_repo());

        // Insert a recent event (timestamp = now)
        logger
            .record(AuditEvent::new(
                AuditEventType::ToolCall,
                "layer",
                &serde_json::json!({}),
            ))
            .await
            .unwrap();

        // Manually insert an old event (year 2020)
        let repo = logger.repo.clone();
        repo.with_conn(|conn| {
            conn.execute(
                "INSERT INTO audit_events (event_type, result, timestamp, payload_json)
                 VALUES ('ToolCall', 'Passed', '2020-01-01T00:00:00Z', ?1)",
                rusqlite::params![serde_json::to_string(&AuditEvent::new(
                    AuditEventType::ToolCall,
                    "layer",
                    &serde_json::json!({}),
                ))?],
            )?;
            Ok(())
        })
        .await
        .unwrap();

        assert_eq!(logger.count_events().await.unwrap(), 2);

        // cleanup(1) removes events older than 1 day — only the 2020 event
        let deleted = logger.cleanup(1).await.unwrap();
        assert_eq!(deleted, 1, "should delete 1 old event");
        assert_eq!(logger.count_events().await.unwrap(), 1);

        // cleanup(365) should NOT remove the recent event
        let deleted = logger.cleanup(365).await.unwrap();
        assert_eq!(deleted, 0, "recent event should survive");
        assert_eq!(logger.count_events().await.unwrap(), 1);
    }

    #[test]
    fn test_redact_sensitive_key_names() {
        let input = serde_json::json!({
            "action": "shell_exec",
            "api_key": "sk-9f3a2b1c",
            "password": "P@ssw0rd",
            "authorization": "Bearer abc",
            "nested": { "client_secret": "s3cret", "path": "/tmp/x" }
        });
        let out = redact_value(&input);
        assert_eq!(out["api_key"], REDACTED);
        assert_eq!(out["password"], REDACTED);
        assert_eq!(out["authorization"], REDACTED);
        assert_eq!(out["nested"]["client_secret"], REDACTED);
        // 非敏感字段与内容原样保留
        assert_eq!(out["action"], "shell_exec");
        assert_eq!(out["nested"]["path"], "/tmp/x");
    }

    #[test]
    fn test_redact_bare_key_field() {
        let out = redact_value(&serde_json::json!({ "key": "topsecret" }));
        assert_eq!(out["key"], REDACTED);
    }

    #[test]
    fn test_redact_token_prefix_in_string_value() {
        let input = serde_json::json!({
            "headers": [
                { "name": "X-Api-Token", "value": "sk-abcdef123456" },
                { "name": "X-Other", "value": "normal-value" }
            ]
        });
        let out = redact_value(&input);
        // `value` 字段名不敏感，但整串以 sk- 开头 → 前缀遮蔽
        assert_eq!(out["headers"][0]["value"], "sk-***");
        assert_eq!(out["headers"][1]["value"], "normal-value");
    }

    #[test]
    fn test_redact_string_prefixes() {
        assert_eq!(redact_string("Bearer secret-token"), "Bearer ***");
        assert_eq!(redact_string("sk-1234567890"), "sk-***");
        assert_eq!(redact_string("ghp_ABCDEFGHIJK"), "ghp_***");
        assert_eq!(redact_string("normal text"), "normal text");
        assert_eq!(redact_string(""), "");
    }

    #[tokio::test]
    async fn test_record_redacts_sensitive_detail() {
        let logger = SqliteAuditLogger::with_repo(test_repo());
        let event = AuditEvent::new(
            AuditEventType::ToolCall,
            "layer",
            &serde_json::json!({"tool": "shell_exec", "api_key": "sk-plaintext-secret"}),
        );
        logger.record(event).await.unwrap();

        let events = logger.query_by_type(AuditEventType::ToolCall, 10).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].detail["api_key"], REDACTED);
        // 明文绝不能出现在落库的 detail 里
        let raw = serde_json::to_string(&events[0].detail).unwrap();
        assert!(!raw.contains("sk-plaintext-secret"));
    }
}
