//! 阶段 1a 验证：消息落库时是否自动写入 session_events（含脱敏）。
//! 直接驱动 `StreamPersistenceAdapter` 走真实内存 SQLite，断言轨迹自动生成。

use std::sync::Arc;

use roseui_ai_agent::protocol::events::tool_call::{ToolCallEventData, ToolCallStatus};
use roseui_common::now_ms;
use roseui_db::models::ConversationRow;
use roseui_db::{IConversationRepository, SqliteConversationRepository, init_database_memory};
use serde_json::json;
use sqlx::Row;

use crate::stream_persistence::{StreamPersistenceAdapter, TextSegmentState};
use crate::service_test::seed_test_user;

async fn seed_conversation(pool: &sqlx::SqlitePool, user_id: &str, conv_id: &str) {
    let repo = SqliteConversationRepository::new(pool.clone());
    let row = ConversationRow {
        id: conv_id.to_owned(),
        user_id: user_id.to_owned(),
        name: "trace-test".to_owned(),
        r#type: "acp".to_owned(),
        extra: "{}".to_owned(),
        model: None,
        status: Some("finished".to_owned()),
        source: Some("roseui".to_owned()),
        channel_chat_id: None,
        pinned: false,
        pinned_at: None,
        folder_id: None,
        project_id: None,
        created_at: 1,
        updated_at: 1,
    };
    repo.create(&row).await.unwrap();
}

#[tokio::test]
async fn persist_tool_call_writes_session_event_with_redaction() {
    let db = init_database_memory().await.unwrap();
    let user_id = "trace-user";
    let conv_id = "trace-conv-1";
    seed_test_user(db.pool(), user_id).await;
    seed_conversation(db.pool(), user_id, conv_id).await;

    let repo = Arc::new(SqliteConversationRepository::new(db.pool().clone()));
    let adapter = StreamPersistenceAdapter::new(
        user_id.to_owned(),
        conv_id.to_owned(),
        "msg-1".to_owned(),
        repo,
        None,
    );

    // 模拟一次工具调用，input 含敏感字段 api_key
    let tool_data = ToolCallEventData {
        call_id: "call-1".to_owned(),
        name: "file_read".to_owned(),
        args: json!({ "path": "/tmp/x" }),
        status: ToolCallStatus::Completed,
        input: Some(json!({ "api_key": "sk-SUPERSECRET123" })),
        output: Some("done".to_owned()),
        description: None,
    };
    adapter.persist_tool_call(&tool_data).await;

    // record_session_event 走 tokio::spawn，等一拍让落库完成
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    let rows = sqlx::query(
        "SELECT event_kind, input_json, output_json, status FROM session_events WHERE conversation_id = ? ORDER BY turn_seq",
    )
    .bind(conv_id)
    .fetch_all(db.pool())
    .await
    .unwrap();

    assert_eq!(rows.len(), 1, "应自动写入 1 条 tool_call 轨迹");
    let row = &rows[0];
    assert_eq!(row.get::<String, _>("event_kind"), "tool_call");
    assert_eq!(row.get::<String, _>("status"), "finish");

    let input: serde_json::Value = serde_json::from_str(&row.get::<String, _>("input_json")).unwrap();
    // 脱敏：api_key 的值应被遮蔽，不应出现明文 sk-SUPERSECRET123
    let api_key = input
        .get("input")
        .and_then(|v| v.get("api_key"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    // 字段名 `api_key` 命中敏感子串 → 值整体脱敏为 "***"，明文不得出现
    assert_ne!(api_key, "sk-SUPERSECRET123", "敏感字段必须脱敏");
    assert_eq!(api_key, "***", "敏感字段名命中时应整体遮蔽为 ***");

    // 同时 messages 表也应有 tool_call 行（既有行为不被破坏）
    let msg_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM messages WHERE conversation_id = ? AND type = 'tool_call'")
        .bind(conv_id)
        .fetch_one(db.pool())
        .await
        .unwrap();
    assert_eq!(msg_count, 1, "既有 messages 落库行为应保持");
}

#[tokio::test]
async fn flush_text_segment_writes_text_session_event() {
    let db = init_database_memory().await.unwrap();
    let user_id = "trace-user-2";
    let conv_id = "trace-conv-2";
    seed_test_user(db.pool(), user_id).await;
    seed_conversation(db.pool(), user_id, conv_id).await;

    let repo = Arc::new(SqliteConversationRepository::new(db.pool().clone()));
    let adapter = StreamPersistenceAdapter::new(
        user_id.to_owned(),
        conv_id.to_owned(),
        "msg-2".to_owned(),
        repo,
        None,
    );

    let mut seg = TextSegmentState {
        id: "seg-1".to_owned(),
        buffer: "你好，这是一段回复".to_owned(),
        created_at: now_ms(),
        record_created: false,
        flush_counter: 0,
    };
    adapter.flush_text_segment(&mut seg).await;
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    let rows = sqlx::query("SELECT event_kind, output_json FROM session_events WHERE conversation_id = ?")
        .bind(conv_id)
        .fetch_all(db.pool())
        .await
        .unwrap();
    assert_eq!(rows.len(), 1, "文本段应写入 1 条 text 轨迹");
    assert_eq!(rows[0].get::<String, _>("event_kind"), "text");
    let out: serde_json::Value = serde_json::from_str(&rows[0].get::<String, _>("output_json")).unwrap();
    assert_eq!(out.get("content").and_then(|v| v.as_str()), Some("你好，这是一段回复"));
}
