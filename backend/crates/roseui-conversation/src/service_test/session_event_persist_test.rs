//! 阶段 1a 验证：消息落库时是否自动写入 session_events（含脱敏）。
//! 直接驱动 `StreamPersistenceAdapter` 走真实内存 SQLite，断言轨迹自动生成。

use std::sync::Arc;

use roseui_ai_agent::protocol::events::tool_call::{ToolCallEventData, ToolCallStatus};
use roseui_common::now_ms;
use roseui_db::models::ConversationRow;
use roseui_db::{IConversationRepository, SqliteConversationRepository, init_database_memory};
use serde_json::json;
use sqlx::Row;

use crate::service_test::seed_test_user;
use crate::stream_persistence::{StreamPersistenceAdapter, TextSegmentState};

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
    let adapter = StreamPersistenceAdapter::new(user_id.to_owned(), conv_id.to_owned(), "msg-1".to_owned(), repo, None);

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
    let msg_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM messages WHERE conversation_id = ? AND type = 'tool_call'")
            .bind(conv_id)
            .fetch_one(db.pool())
            .await
            .unwrap();
    assert_eq!(msg_count, 1, "既有 messages 落库行为应保持");
}

#[tokio::test]
async fn insert_session_event_model_call_persists() {
    let db = init_database_memory().await.unwrap();
    let user_id = "trace-user-3";
    let conv_id = "trace-conv-3";
    seed_test_user(db.pool(), user_id).await;
    seed_conversation(db.pool(), user_id, conv_id).await;

    let repo = Arc::new(SqliteConversationRepository::new(db.pool().clone()));
    let event = roseui_db::models::SessionEventRow {
        id: "trace_model_1".to_owned(),
        conversation_id: conv_id.to_owned(),
        turn_seq: 0,
        event_kind: "model_call".to_owned(),
        role: Some("assistant".to_owned()),
        model: Some("claude-sonnet-4".to_owned()),
        input_json: "{}".to_owned(),
        output_json: "{}".to_owned(),
        token_usage_json: json!({ "duration_ms": 1234 }).to_string(),
        status: Some("finish".to_owned()),
        created_at: 1,
    };
    repo.insert_session_event(user_id, &event).await.unwrap();

    let row =
        sqlx::query("SELECT event_kind, model, token_usage_json, status FROM session_events WHERE conversation_id = ?")
            .bind(conv_id)
            .fetch_one(db.pool())
            .await
            .unwrap();
    assert_eq!(row.get::<String, _>("event_kind"), "model_call");
    assert_eq!(row.get::<String, _>("model"), "claude-sonnet-4");
    assert_eq!(row.get::<String, _>("token_usage_json"), "{\"duration_ms\":1234}");
    assert_eq!(row.get::<String, _>("status"), "finish");
}

#[tokio::test]
async fn flush_text_segment_writes_text_session_event() {
    let db = init_database_memory().await.unwrap();
    let user_id = "trace-user-2";
    let conv_id = "trace-conv-2";
    seed_test_user(db.pool(), user_id).await;
    seed_conversation(db.pool(), user_id, conv_id).await;

    let repo = Arc::new(SqliteConversationRepository::new(db.pool().clone()));
    let adapter = StreamPersistenceAdapter::new(user_id.to_owned(), conv_id.to_owned(), "msg-2".to_owned(), repo, None);

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

#[tokio::test]
async fn list_session_events_filters_by_kind_model_and_range() {
    let db = init_database_memory().await.unwrap();
    let user_id = "trace-user-4";
    let conv_id = "trace-conv-4";
    seed_test_user(db.pool(), user_id).await;
    seed_conversation(db.pool(), user_id, conv_id).await;

    let repo = SqliteConversationRepository::new(db.pool().clone());
    let make = |id: &str, kind: &str, model: &str, ts: i64, seq: i64| roseui_db::models::SessionEventRow {
        id: id.to_owned(),
        conversation_id: conv_id.to_owned(),
        turn_seq: seq,
        event_kind: kind.to_owned(),
        role: Some("assistant".to_owned()),
        model: Some(model.to_owned()),
        input_json: "{}".to_owned(),
        output_json: "{}".to_owned(),
        token_usage_json: "{}".to_owned(),
        status: Some("finish".to_owned()),
        created_at: ts,
    };
    // 3 条：两个 model_call（不同模型/时间），一个 tool_call
    repo.insert_session_event(user_id, &make("e1", "model_call", "claude-sonnet-4", 1000, 1))
        .await
        .unwrap();
    repo.insert_session_event(user_id, &make("e2", "tool_call", "claude-sonnet-4", 2000, 2))
        .await
        .unwrap();
    repo.insert_session_event(user_id, &make("e3", "model_call", "gpt-4o", 3000, 3))
        .await
        .unwrap();

    // 全量：按 turn_seq 升序返回 3 条
    let all = repo
        .list_session_events(user_id, conv_id, &Default::default())
        .await
        .unwrap();
    assert_eq!(all.len(), 3);
    assert_eq!(all[0].id, "e1");
    assert_eq!(all[2].id, "e3");

    // 按 event_kind 过滤
    let only_model = repo
        .list_session_events(
            user_id,
            conv_id,
            &roseui_db::SessionEventFilters {
                event_kind: Some("model_call".to_owned()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(only_model.len(), 2);
    assert!(only_model.iter().all(|e| e.event_kind == "model_call"));

    // 按 model 子串过滤
    let only_sonnet = repo
        .list_session_events(
            user_id,
            conv_id,
            &roseui_db::SessionEventFilters {
                model: Some("sonnet".to_owned()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(only_sonnet.len(), 2);
    assert!(
        only_sonnet
            .iter()
            .all(|e| e.model.as_deref() == Some("claude-sonnet-4"))
    );

    // 按时间范围过滤（仅 1000~2000）
    let ranged = repo
        .list_session_events(
            user_id,
            conv_id,
            &roseui_db::SessionEventFilters {
                from_ts: Some(1000),
                to_ts: Some(2000),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(ranged.len(), 2);
    assert_eq!(ranged[0].id, "e1");
    assert_eq!(ranged[1].id, "e2");

    // 交叉过滤：model_call + sonnet
    let combined = repo
        .list_session_events(
            user_id,
            conv_id,
            &roseui_db::SessionEventFilters {
                event_kind: Some("model_call".to_owned()),
                model: Some("sonnet".to_owned()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(combined.len(), 1);
    assert_eq!(combined[0].id, "e1");
}
