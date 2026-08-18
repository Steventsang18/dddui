use std::sync::Arc;

use roseui_ai_agent::protocol::events::{
    ErrorEventData, TipType, TipsEventData,
    tool_call::{AcpToolCallStatus, ToolCallStatus},
};
use roseui_api_types::{ConversationRuntimeSummary, WebSocketMessage};
use roseui_common::{ErrorChain, normalize_keys_to_snake_case, now_ms};
use roseui_db::models::{MessageRow, SessionEventRow, redact_json, text_event};
use roseui_db::{ConversationRowUpdate, DbError, IConversationRepository, MessageRowUpdate};
use roseui_realtime::EventBroadcaster;
use serde_json::json;
use tracing::{debug, error, warn};

use crate::runtime_completion::RuntimeCompletionPublisher;
use crate::runtime_persistence::{RuntimePersistenceCoordinator, RuntimeWriteKind};
use crate::service::ConversationService;

fn is_not_found(err: &DbError) -> bool {
    matches!(err, DbError::NotFound(_))
}

fn is_foreign_key_constraint(err: &DbError) -> bool {
    err.to_string().contains("FOREIGN KEY constraint failed")
}

fn is_deleted_during_stream_persistence(err: &DbError) -> bool {
    is_not_found(err) || is_foreign_key_constraint(err)
}

fn log_persist_error(err: &DbError, message: &'static str) {
    if is_deleted_during_stream_persistence(err) {
        debug!(error = %ErrorChain(err), "{message}; conversation was likely deleted during stream finalization");
    } else {
        error!(error = %ErrorChain(err), "{message}");
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TextSegmentState {
    pub id: String,
    pub buffer: String,
    pub created_at: i64,
    pub record_created: bool,
    pub flush_counter: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct PersistedTextSegment {
    pub id: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ThinkingSegmentState {
    pub id: String,
    pub buffer: String,
    pub started_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FinalTextOverride {
    pub msg_id: String,
    pub text: String,
    pub hidden: bool,
}

#[derive(Clone)]
pub(crate) struct StreamPersistenceAdapter {
    user_id: String,
    conversation_id: String,
    msg_id: String,
    repo: Arc<dyn IConversationRepository>,
    persistence: Option<RuntimePersistenceCoordinator>,
}

impl StreamPersistenceAdapter {
    pub fn new(
        user_id: String,
        conversation_id: String,
        msg_id: String,
        repo: Arc<dyn IConversationRepository>,
        persistence: Option<RuntimePersistenceCoordinator>,
    ) -> Self {
        Self {
            user_id,
            conversation_id,
            msg_id,
            repo,
            persistence,
        }
    }

    pub fn with_persistence(mut self, persistence: RuntimePersistenceCoordinator) -> Self {
        self.persistence = Some(persistence);
        self
    }

    /// 追加一条轨迹事件到 session_events（脱敏后的 input/output）。
    /// 失败仅告警，不影响主消息落库（轨迹是旁路、尽力而为）。
    fn record_session_event(&self, event: SessionEventRow) {
        let repo = self.repo.clone();
        let user_id = self.user_id.clone();
        tokio::spawn(async move {
            if let Err(e) = repo.insert_session_event(&user_id, &event).await {
                warn!(error = %ErrorChain(&e), "Failed to record session trace event");
            }
        });
    }

    /// 便捷封装：从工具调用数据构造并落库一条 `tool_call` 轨迹事件。
    fn record_tool_event(
        &self,
        call_id: &str,
        status: &str,
        input: serde_json::Value,
        output: serde_json::Value,
        created_at: i64,
    ) {
        let event = SessionEventRow {
            id: format!("trace_{call_id}"),
            conversation_id: self.conversation_id.clone(),
            turn_seq: 0, // 由 repo 派生 MAX+1
            event_kind: "tool_call".to_string(),
            role: Some("assistant".to_string()),
            model: None,
            input_json: redact_json(&input).to_string(),
            output_json: redact_json(&output).to_string(),
            token_usage_json: "{}".to_string(),
            status: Some(status.to_string()),
            created_at,
        };
        self.record_session_event(event);
    }

    #[tracing::instrument(skip_all, fields(conversation_id = %self.conversation_id))]
    pub async fn complete_conversation(
        &self,
        broadcaster: &Arc<dyn EventBroadcaster>,
        turn_id: &str,
        runtime: Option<ConversationRuntimeSummary>,
    ) {
        if let Some(persistence) = &self.persistence {
            RuntimeCompletionPublisher::new(
                self.user_id.clone(),
                self.repo.clone(),
                broadcaster.clone(),
                persistence.clone(),
            )
            .publish(&self.conversation_id, turn_id, runtime)
            .await;
            return;
        }

        let update = ConversationRowUpdate {
            status: Some("finished".to_owned()),
            updated_at: Some(now_ms()),
            ..Default::default()
        };
        if let Err(e) = self.repo.update(&self.user_id, &self.conversation_id, &update).await {
            log_persist_error(&e, "Failed to update conversation status");
        }

        let payload = json!({
            "user_id": self.user_id,
            "conversation_id": self.conversation_id,
            "session_id": self.conversation_id,
            "turn_id": turn_id,
            "status": "finished",
            "canSendMessage": true,
            "runtime": runtime,
        });
        broadcaster.broadcast(WebSocketMessage::new("turn.completed", payload));

        debug!(conversation_id = %self.conversation_id, turn_id, status = "finished", "Turn completed");
    }

    fn allows_write(&self, kind: RuntimeWriteKind) -> bool {
        self.persistence
            .as_ref()
            .is_none_or(|persistence| persistence.allows(&self.conversation_id, kind))
    }

    #[tracing::instrument(skip_all)]
    pub async fn flush_text_segment(&self, segment: &mut TextSegmentState) {
        if !self.allows_write(RuntimeWriteKind::AssistantTextFlush) {
            return;
        }
        if segment.buffer.is_empty() {
            return;
        }

        let content = json!({ "content": segment.buffer }).to_string();

        if segment.record_created {
            let update = MessageRowUpdate {
                content: Some(content),
                status: Some(Some("work".into())),
                hidden: None,
            };
            if let Err(e) = self
                .repo
                .update_message(&self.user_id, &self.conversation_id, &segment.id, &update)
                .await
            {
                log_persist_error(&e, "Failed to update streaming text segment");
            }
        } else {
            let row = MessageRow {
                id: segment.id.clone(),
                conversation_id: self.conversation_id.clone(),
                msg_id: Some(segment.id.clone()),
                r#type: "text".into(),
                content,
                position: Some("left".into()),
                status: Some("work".into()),
                hidden: false,
                created_at: segment.created_at,
            };
            if let Err(e) = self.repo.insert_message(&self.user_id, &row).await {
                log_persist_error(&e, "Failed to create streaming text segment");
            } else {
                self.record_session_event(text_event(
                    segment.id.clone(),
                    self.conversation_id.clone(),
                    0,
                    Some("assistant".to_string()),
                    None,
                    &segment.buffer,
                    Some("work".to_string()),
                    segment.created_at,
                ));
            }
            segment.record_created = true;
        }
    }

    #[tracing::instrument(skip_all)]
    pub async fn finalize_text_segment(&self, segment: TextSegmentState, status: &str) -> Option<PersistedTextSegment> {
        if !self.allows_write(RuntimeWriteKind::AssistantTextFinalize) {
            return None;
        }
        if segment.buffer.is_empty() {
            return None;
        }

        let content = json!({ "content": segment.buffer }).to_string();
        if segment.record_created {
            let update = MessageRowUpdate {
                content: Some(content),
                status: Some(Some(status.to_owned())),
                hidden: Some(false),
            };
            if let Err(e) = self
                .repo
                .update_message(&self.user_id, &self.conversation_id, &segment.id, &update)
                .await
            {
                log_persist_error(&e, "Failed to finalize text segment");
                return None;
            }
        } else {
            let row = MessageRow {
                id: segment.id.clone(),
                conversation_id: self.conversation_id.clone(),
                msg_id: Some(segment.id.clone()),
                r#type: "text".into(),
                content,
                position: Some("left".into()),
                status: Some(status.to_owned()),
                hidden: false,
                created_at: segment.created_at,
            };
            if let Err(e) = self.repo.insert_message(&self.user_id, &row).await {
                log_persist_error(&e, "Failed to create finalized text segment");
                return None;
            }
        }

        Some(PersistedTextSegment { id: segment.id })
    }

    #[tracing::instrument(skip_all)]
    pub async fn persist_final_text(
        &self,
        text_segments: &[PersistedTextSegment],
        status: &str,
        final_text: &str,
        hidden: bool,
        rewrite_segments: bool,
    ) -> Vec<FinalTextOverride> {
        if !self.allows_write(RuntimeWriteKind::TerminalFinalize) {
            return Vec::new();
        }

        let mut overrides = Vec::new();
        if let Some(primary_segment) = text_segments.first() {
            if rewrite_segments {
                let content = json!({ "content": final_text }).to_string();
                let update = MessageRowUpdate {
                    content: Some(content),
                    status: Some(Some(status.to_owned())),
                    hidden: Some(hidden),
                };
                if let Err(e) = self
                    .repo
                    .update_message(&self.user_id, &self.conversation_id, &primary_segment.id, &update)
                    .await
                {
                    log_persist_error(&e, "Failed to rewrite finalized text segment");
                }
                overrides.push(FinalTextOverride {
                    msg_id: primary_segment.id.clone(),
                    text: final_text.to_owned(),
                    hidden,
                });

                for segment in text_segments.iter().skip(1) {
                    let hide_update = MessageRowUpdate {
                        content: None,
                        status: Some(Some(status.to_owned())),
                        hidden: Some(true),
                    };
                    if let Err(e) = self
                        .repo
                        .update_message(&self.user_id, &self.conversation_id, &segment.id, &hide_update)
                        .await
                    {
                        log_persist_error(&e, "Failed to hide superseded text segment");
                    }
                    overrides.push(FinalTextOverride {
                        msg_id: segment.id.clone(),
                        text: String::new(),
                        hidden: true,
                    });
                }
            } else {
                for segment in text_segments {
                    let status_update = MessageRowUpdate {
                        content: None,
                        status: Some(Some(status.to_owned())),
                        hidden: Some(false),
                    };
                    if let Err(e) = self
                        .repo
                        .update_message(&self.user_id, &self.conversation_id, &segment.id, &status_update)
                        .await
                    {
                        log_persist_error(&e, "Failed to finalize text segment status");
                    }
                }
            }
        } else if !hidden {
            let row = MessageRow {
                id: self.msg_id.clone(),
                conversation_id: self.conversation_id.clone(),
                msg_id: Some(self.msg_id.clone()),
                r#type: "text".into(),
                content: json!({ "content": final_text }).to_string(),
                position: Some("left".into()),
                status: Some(status.to_owned()),
                hidden: false,
                created_at: now_ms(),
            };
            if let Err(e) = self.repo.insert_message(&self.user_id, &row).await {
                log_persist_error(&e, "Failed to create final fallback message");
            } else {
                self.record_session_event(text_event(
                    self.msg_id.clone(),
                    self.conversation_id.clone(),
                    0,
                    Some("assistant".to_string()),
                    None,
                    final_text,
                    Some(status.to_owned()),
                    now_ms(),
                ));
            }
        }

        overrides
    }

    #[tracing::instrument(skip_all)]
    pub async fn persist_error_tip(&self, data: &ErrorEventData) {
        if !self.allows_write(RuntimeWriteKind::TerminalFinalize) {
            return;
        }

        let content = json!({ "content": &data.message, "type": "error", "error": &data }).to_string();
        let row = MessageRow {
            id: ConversationService::mint_msg_id(),
            conversation_id: self.conversation_id.clone(),
            msg_id: None,
            r#type: "tips".into(),
            content,
            position: Some("left".into()),
            status: Some("error".into()),
            hidden: false,
            created_at: now_ms(),
        };
        if let Err(e) = self.repo.insert_message(&self.user_id, &row).await {
            log_persist_error(&e, "Failed to store error message");
        }
    }

    #[tracing::instrument(skip_all)]
    pub async fn persist_tip(&self, data: &TipsEventData) {
        if !self.allows_write(RuntimeWriteKind::TerminalFinalize) {
            return;
        }

        let status = match data.tip_type {
            TipType::Error => "error",
            TipType::Success | TipType::Warning | TipType::Info => "finish",
        };
        let content = json!({
            "content": &data.content,
            "type": &data.tip_type,
            "code": &data.code,
            "params": &data.params,
        })
        .to_string();
        let row = MessageRow {
            id: ConversationService::mint_msg_id(),
            conversation_id: self.conversation_id.clone(),
            msg_id: None,
            r#type: "tips".into(),
            content,
            position: Some("left".into()),
            status: Some(status.into()),
            hidden: false,
            created_at: now_ms(),
        };
        if let Err(e) = self.repo.insert_message(&self.user_id, &row).await {
            log_persist_error(&e, "Failed to store tip message");
        }
    }

    #[tracing::instrument(skip_all)]
    pub async fn persist_thinking_segment(&self, segment: ThinkingSegmentState, duration_ms: u64) {
        if segment.buffer.is_empty() {
            return;
        }
        if !self.allows_write(RuntimeWriteKind::AssistantThinkingFinalize) {
            return;
        }
        let content = json!({
            "content": segment.buffer,
            "status": "done",
            "duration_ms": duration_ms,
        })
        .to_string();
        let seg_id = segment.id.clone();
        let row = MessageRow {
            id: seg_id.clone(),
            conversation_id: self.conversation_id.clone(),
            msg_id: Some(seg_id.clone()),
            r#type: "thinking".into(),
            content,
            position: Some("left".into()),
            status: Some("finish".into()),
            hidden: false,
            created_at: segment.started_at,
        };
        if let Err(e) = self.repo.insert_message(&self.user_id, &row).await {
            log_persist_error(&e, "Failed to persist thinking message");
        } else {
            self.record_session_event(text_event(
                seg_id,
                self.conversation_id.clone(),
                0,
                Some("assistant".to_string()),
                None,
                &segment.buffer,
                Some("finish".to_string()),
                segment.started_at,
            ));
        }
    }

    /// Persist a Gemini-style tool_call event.
    #[tracing::instrument(skip_all)]
    pub async fn persist_tool_call(&self, data: &roseui_ai_agent::protocol::events::tool_call::ToolCallEventData) {
        if !self.allows_write(RuntimeWriteKind::ToolCallPersist) {
            return;
        }
        if data.call_id.trim().is_empty() {
            warn!(
                tool = %data.name,
                status = ?data.status,
                "Skipping tool_call persistence because call_id is empty"
            );
            return;
        }

        let status = match data.status {
            ToolCallStatus::Running => "work",
            ToolCallStatus::Completed => "finish",
            ToolCallStatus::Error => "error",
            // A cancelled call is terminal: the row must leave "work" so the
            // frontend spinner (hasRunningToolMessages) stops after interrupt.
            ToolCallStatus::Canceled => "finish",
        };
        let content = serde_json::to_string(data).unwrap_or_default();

        // The engine's call_id (e.g. `rupoo-tool-1`) is only unique per session,
        // while the messages table keys on a globally unique `id`. Scope the
        // storage id by conversation so tool calls from different sessions never
        // collide on the primary key. The same (conversation, call_id) pair still
        // maps to the same row, so Running -> Completed updates in place.
        let storage_id = format!("{}-{}", self.conversation_id, data.call_id);
        let row = MessageRow {
            id: storage_id.clone(),
            conversation_id: self.conversation_id.clone(),
            msg_id: Some(storage_id),
            r#type: "tool_call".into(),
            content,
            position: Some("left".into()),
            status: Some(status.to_owned()),
            hidden: false,
            created_at: now_ms(),
        };
        if let Err(e) = self.repo.upsert_message(&self.user_id, &row).await {
            error!(
                call_id = %data.call_id,
                tool = %data.name,
                status,
                error = %ErrorChain(&e),
                "Failed to upsert tool_call message"
            );
        } else {
            debug!(
                call_id = %data.call_id,
                tool = %data.name,
                status,
                "Upserted tool_call message"
            );
            let input = serde_json::json!({
                "name": data.name,
                "args": data.args,
                "input": data.input,
            });
            let output = serde_json::json!({ "output": data.output });
            self.record_tool_event(
                &data.call_id,
                status,
                input,
                output,
                now_ms(),
            );
        }
    }

    /// Persist an ACP (Claude CLI) tool call event.
    #[tracing::instrument(skip_all)]
    pub async fn persist_acp_tool_call(
        &self,
        data: &roseui_ai_agent::protocol::events::tool_call::AcpToolCallEventData,
    ) {
        if !self.allows_write(RuntimeWriteKind::AcpToolCallPersist) {
            return;
        }
        let tool_call_id = &data.update.tool_call_id;
        let status = match data.update.status {
            Some(AcpToolCallStatus::Pending) | None => "work",
            Some(AcpToolCallStatus::InProgress) => "work",
            Some(AcpToolCallStatus::Completed) => "finish",
            Some(AcpToolCallStatus::Failed) => "error",
        };

        let mut value = serde_json::to_value(data).unwrap_or_default();
        normalize_keys_to_snake_case(&mut value);
        let content = value.to_string();

        let row = MessageRow {
            id: tool_call_id.clone(),
            conversation_id: self.conversation_id.clone(),
            msg_id: Some(tool_call_id.clone()),
            r#type: "acp_tool_call".into(),
            content,
            position: Some("left".into()),
            status: Some(status.to_owned()),
            hidden: false,
            created_at: now_ms(),
        };
        if let Err(e) = self.repo.upsert_message(&self.user_id, &row).await {
            error!(error = %ErrorChain(&e), "Failed to upsert acp_tool_call message");
        } else {
            let input = serde_json::json!({ "raw_input": data.update.raw_input });
            let output = serde_json::json!({ "raw_output": data.update.raw_output });
            self.record_tool_event(
                tool_call_id,
                status,
                input,
                output,
                now_ms(),
            );
        }
    }

    /// Persist a tool_group event (array of tool summaries).
    #[tracing::instrument(skip_all)]
    pub async fn persist_tool_group(&self, entries: &[roseui_ai_agent::protocol::events::tool_call::ToolGroupEntry]) {
        if !self.allows_write(RuntimeWriteKind::ToolGroupPersist) {
            return;
        }
        let all_done = entries.iter().all(|e| {
            matches!(
                e.status,
                ToolCallStatus::Completed | ToolCallStatus::Error | ToolCallStatus::Canceled
            )
        });
        let status = if all_done { "finish" } else { "work" };
        let content = serde_json::to_string(entries).unwrap_or_default();

        let group_id = entries
            .first()
            .map(|e| e.call_id.clone())
            .unwrap_or_else(ConversationService::mint_msg_id);

        let existing = self
            .repo
            .get_message_by_msg_id(&self.user_id, &self.conversation_id, &group_id, "tool_group")
            .await
            .unwrap_or(None);

        if existing.is_some() {
            let update = MessageRowUpdate {
                content: Some(content),
                status: Some(Some(status.to_owned())),
                hidden: None,
            };
            if let Err(e) = self
                .repo
                .update_message(&self.user_id, &self.conversation_id, &group_id, &update)
                .await
            {
                error!(error = %ErrorChain(&e), "Failed to update tool_group message");
            }
        } else {
            let row = MessageRow {
                id: group_id.clone(),
                conversation_id: self.conversation_id.clone(),
                msg_id: Some(group_id),
                r#type: "tool_group".into(),
                content,
                position: Some("left".into()),
                status: Some(status.to_owned()),
                hidden: false,
                created_at: now_ms(),
            };
            if let Err(e) = self.repo.insert_message(&self.user_id, &row).await {
                error!(error = %ErrorChain(&e), "Failed to persist tool_group message");
            }
        }
    }
}
