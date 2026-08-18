#![allow(clippy::disallowed_types)]

use axum::Router;
use axum::body::Body;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Extension, Json, Path, Query, State};
use axum::http::StatusCode;
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::response::Response;
use axum::routing::{get, patch, post};

use roseui_api_types::{
    ActiveCountResponse, ApiResponse, ApprovalCheckQuery, ApprovalCheckResponse, CancelConversationRequest,
    CancelConversationResponse, CloneConversationRequest, CompactConversationResponse, ConfirmRequest,
    ConfirmationListResponse, ConversationArtifactListResponse, ConversationArtifactResponse, ConversationListResponse,
    ConversationResponse, CreateConversationRequest, EnsureConversationRuntimeResponse, ListConversationsQuery,
    ListMessagesQuery, MessageListResponse, MessageResponse, MessageSearchResponse, SearchMessagesQuery,
    SendMessageRequest, SendMessageResponse, UpdateConversationArtifactRequest, UpdateConversationRequest,
};
use roseui_auth::CurrentUser;
use roseui_common::{ApiError, now_ms};
use roseui_db::models::SessionEventRow;

use crate::ConversationError;
use crate::state::ConversationRouterState;

impl From<ConversationError> for ApiError {
    fn from(error: ConversationError) -> Self {
        match error {
            ConversationError::NotFound { id } => ApiError::NotFound(format!("Conversation {id} not found")),
            ConversationError::MessageNotFound { id } => ApiError::NotFound(format!("Message {id} not found")),
            ConversationError::ArtifactNotFound { id } => ApiError::NotFound(format!("Artifact {id} not found")),
            ConversationError::ActiveAgentNotFound { .. } => {
                ApiError::NotFound("No active agent for this conversation".into())
            }
            ConversationError::Archived { reason, .. } => ApiError::ConversationArchived(reason),
            ConversationError::BadRequest { reason } => ApiError::BadRequest(reason),
            ConversationError::Busy { reason } if reason.starts_with("CROSS_ACCOUNT_REFERENCE:") => {
                ApiError::coded(StatusCode::CONFLICT, "CROSS_ACCOUNT_REFERENCE", reason, None)
            }
            ConversationError::Busy { reason } => ApiError::Conflict(reason),
            ConversationError::Forbidden { reason } => ApiError::Forbidden(reason),
            ConversationError::NotFoundReason { reason } => ApiError::NotFound(reason),
            ConversationError::Unauthorized { reason } => ApiError::Unauthorized(reason),
            ConversationError::RateLimited => ApiError::RateLimited,
            ConversationError::BadGateway { reason } => ApiError::BadGateway(reason),
            ConversationError::Timeout { reason } => ApiError::Timeout(reason),
            ConversationError::ConfigConfirmationTimeout {
                conversation_id,
                option_id,
                requested,
                last_observed,
            } => ApiError::coded(
                StatusCode::GATEWAY_TIMEOUT,
                "confirmation_timeout",
                "ACP runtime did not confirm the requested config option before timeout",
                Some(serde_json::json!({
                    "conversation_id": conversation_id,
                    "option_id": option_id,
                    "requested": requested,
                    "last_observed": last_observed,
                })),
            ),
            ConversationError::ConfigUpdateInProgress {
                conversation_id,
                option_id,
                requested,
            } => ApiError::coded(
                StatusCode::CONFLICT,
                "config_update_in_progress",
                "ACP config update is already in progress",
                Some(serde_json::json!({
                    "conversation_id": conversation_id,
                    "option_id": option_id,
                    "requested": requested,
                })),
            ),
            ConversationError::TeamRuntimeRequired {
                conversation_id,
                team_id,
            } => ApiError::coded(
                StatusCode::CONFLICT,
                "TEAM_RUNTIME_REQUIRED",
                "This conversation belongs to a team; use the team runtime session",
                Some(serde_json::json!({
                    "conversation_id": conversation_id,
                    "team_id": team_id,
                })),
            ),
            ConversationError::Unprocessable { reason } => ApiError::UnprocessableEntity(reason),
            ConversationError::Internal { reason } => ApiError::Internal(reason),
            ConversationError::WorkspacePathUnavailable { path } => ApiError::WorkspacePathUnavailable(path),
            ConversationError::WorkspacePathRuntimeUnavailable { path } => {
                ApiError::WorkspacePathRuntimeUnavailable(path)
            }
            ConversationError::OpenClawGatewayUnreachable { detail } => ApiError::coded(
                StatusCode::BAD_GATEWAY,
                "USER_AGENT_OPENCLAW_GATEWAY_UNREACHABLE",
                "OpenClaw Gateway is not reachable",
                Some(serde_json::json!({
                    "detail": detail,
                    "error_kind": "openclaw_gateway_unreachable",
                    "backend": "openclaw",
                    "port": 18789
                })),
            ),
            ConversationError::Acp(_) => ApiError::BadGateway("Agent protocol error".into()),
        }
    }
}

/// Build the conversation router (CRUD + message flow + confirmation + extended operations).
///
/// All routes require authentication (applied by the caller).
pub fn conversation_routes(state: ConversationRouterState) -> Router {
    Router::new()
        .route("/api/conversations", post(create).get(list))
        .route("/api/conversations/{id}", get(get_one).patch(update).delete(delete_one))
        .route("/api/conversations/{id}/reset", post(reset))
        .route("/api/conversations/{id}/associated", get(associated))
        .route("/api/conversations/{id}/messages", get(list_msg).post(send_msg))
        .route("/api/conversations/{id}/messages/{messageId}", get(get_msg))
        .route("/api/conversations/{id}/trace-event", post(record_trace_event))
        .route("/api/conversations/{id}/trace", get(trace))
        .route("/api/conversations/{id}/trace/export", get(trace_export))
        .route("/api/conversations/{id}/artifacts", get(list_artifacts))
        .route("/api/conversations/{id}/artifacts/{artifactId}", patch(update_artifact))
        .route("/api/conversations/{id}/cancel", post(cancel))
        .route("/api/conversations/{id}/compact", post(compact))
        .route("/api/conversations/{id}/runtime/ensure", post(ensure_runtime))
        .route("/api/conversations/{id}/active-lease", post(active_lease))
        // Confirmation system
        .route("/api/conversations/{id}/confirmations", get(list_confirmations))
        .route("/api/conversations/{id}/confirmations/{callId}/confirm", post(confirm))
        .route("/api/conversations/{id}/approvals/check", get(check_approval))
        .route("/api/conversations/active-count", get(active_count))
        .route("/api/conversations/clone", post(clone))
        .route("/api/messages/search", get(search_messages))
        .with_state(state)
}

// ── Handlers ───────────────────────────────────────────────────────

async fn create(
    State(state): State<ConversationRouterState>,
    Extension(user): Extension<CurrentUser>,
    body: Result<Json<CreateConversationRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<ApiResponse<ConversationResponse>>), ApiError> {
    let Json(req) = body.map_err(ApiError::from)?;
    let conversation = state.service.create(&user.id, req).await.map_err(ApiError::from)?;
    Ok((StatusCode::CREATED, Json(ApiResponse::ok(conversation))))
}

async fn list(
    State(state): State<ConversationRouterState>,
    Extension(user): Extension<CurrentUser>,
    Query(query): Query<ListConversationsQuery>,
) -> Result<Json<ApiResponse<ConversationListResponse>>, ApiError> {
    let result = state.service.list(&user.id, query).await.map_err(ApiError::from)?;
    Ok(Json(ApiResponse::ok(result)))
}

async fn clone(
    State(state): State<ConversationRouterState>,
    Extension(user): Extension<CurrentUser>,
    body: Result<Json<CloneConversationRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<ApiResponse<ConversationResponse>>), ApiError> {
    let Json(req) = body.map_err(ApiError::from)?;
    let conversation = state
        .service
        .clone_create(&user.id, req)
        .await
        .map_err(ApiError::from)?;
    Ok((StatusCode::CREATED, Json(ApiResponse::ok(conversation))))
}

async fn get_one(
    State(state): State<ConversationRouterState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<ConversationResponse>>, ApiError> {
    let conversation = state.service.get(&user.id, &id).await.map_err(ApiError::from)?;
    Ok(Json(ApiResponse::ok(conversation)))
}

/// 接收前端上报的对话轨迹事件（如 ACP `request_trace` 的 model_call），写入
/// session_events 供回放查询。Owner 模式下鉴权中间件注入 system_default_user，
/// 无需请求自带 token。
#[derive(serde::Deserialize)]
struct RecordTraceEventRequest {
    /// 事件类型，默认 `model_call`；允许前端扩展其他类型。
    #[serde(default = "default_trace_kind")]
    event_kind: String,
    /// 角色，默认 assistant。
    #[serde(default)]
    role: Option<String>,
    /// 模型标识（如 request_trace 的 model_id）。
    #[serde(default)]
    model: Option<String>,
    /// 输入 JSON（对象）。
    #[serde(default)]
    input: serde_json::Value,
    /// 输出 JSON（对象）。
    #[serde(default)]
    output: serde_json::Value,
    /// token 用量 / 耗时等 JSON。
    #[serde(default)]
    token_usage: serde_json::Value,
    /// 状态，默认 finish。
    #[serde(default = "default_trace_status")]
    status: String,
    /// 可选客户端时间戳（epoch ms）；缺省用服务端 now_ms。
    #[serde(default)]
    created_at: Option<i64>,
}

fn default_trace_kind() -> String {
    "model_call".to_string()
}
fn default_trace_status() -> String {
    "finish".to_string()
}

async fn record_trace_event(
    State(state): State<ConversationRouterState>,
    Extension(user): Extension<CurrentUser>,
    Path(conversation_id): Path<String>,
    body: Result<Json<RecordTraceEventRequest>, JsonRejection>,
) -> Result<StatusCode, ApiError> {
    let Json(req) = body.map_err(ApiError::from)?;

    // 脱敏输入/输出/用量，避免明文凭据落库。
    let input_json = roseui_db::models::redact_json(&req.input).to_string();
    let output_json = roseui_db::models::redact_json(&req.output).to_string();
    let token_usage_json = roseui_db::models::redact_json(&req.token_usage).to_string();

    let event = SessionEventRow {
        id: format!("trace_{}_{}", conversation_id, now_ms()),
        conversation_id: conversation_id.clone(),
        turn_seq: 0, // 由 repo 派生 MAX+1
        event_kind: req.event_kind,
        role: req.role.or_else(|| Some("assistant".to_string())),
        model: req.model,
        input_json,
        output_json,
        token_usage_json,
        status: Some(req.status),
        created_at: req.created_at.unwrap_or_else(now_ms),
    };

    state
        .service
        .conversation_repo()
        .clone()
        .insert_session_event(&user.id, &event)
        .await
        .map_err(|e| ApiError::Internal(format!("failed to record trace event: {e}")))?;

    Ok(StatusCode::CREATED)
}

async fn update(
    State(state): State<ConversationRouterState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<String>,
    body: Result<Json<UpdateConversationRequest>, JsonRejection>,
) -> Result<Json<ApiResponse<ConversationResponse>>, ApiError> {
    let Json(req) = body.map_err(ApiError::from)?;
    let conversation = state
        .service
        .update(&user.id, &id, req, &state.task_manager)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(ApiResponse::ok(conversation)))
}

async fn delete_one(
    State(state): State<ConversationRouterState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    state.service.delete(&user.id, &id).await.map_err(ApiError::from)?;
    Ok(Json(ApiResponse::success()))
}

async fn reset(
    State(state): State<ConversationRouterState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    state.service.reset(&user.id, &id).await.map_err(ApiError::from)?;
    Ok(Json(ApiResponse::success()))
}

async fn associated(
    State(state): State<ConversationRouterState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Vec<ConversationResponse>>>, ApiError> {
    let items = state
        .service
        .list_associated(&user.id, &id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(ApiResponse::ok(items)))
}

async fn list_msg(
    State(state): State<ConversationRouterState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<String>,
    Query(query): Query<ListMessagesQuery>,
) -> Result<Json<ApiResponse<MessageListResponse>>, ApiError> {
    let result = state
        .service
        .list_messages(&user.id, &id, query)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(ApiResponse::ok(result)))
}

/// Query parameters for `GET /api/conversations/{id}/trace`.
/// All filters are optional; `event_kind`/`model`/`from_ts`/`to_ts` narrow the
/// replay window. `limit` caps the returned event count (default 200).
#[derive(serde::Deserialize)]
struct TraceQuery {
    #[serde(default)]
    event_kind: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    from_ts: Option<i64>,
    #[serde(default)]
    to_ts: Option<i64>,
    #[serde(default)]
    limit: Option<u32>,
    #[serde(default)]
    fmt: Option<String>,
}

/// Export format for `GET /api/conversations/{id}/trace/export`.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum TraceExportFormat {
    #[default]
    Markdown,
    Json,
}

impl TraceExportFormat {
    fn parse(s: &Option<String>) -> Self {
        match s.as_deref() {
            Some("json") => TraceExportFormat::Json,
            _ => TraceExportFormat::Markdown,
        }
    }
}

/// Returns the session trace (监控录像) as a downloadable file.
///
/// Reuses the same ownership-checked `list_session_events` + `build_timeline`
/// pipeline as `trace`, but renders to a self-contained document instead of the
/// matrix envelope. Content is already redacted at write-time (see
/// `StreamPersistenceAdapter`), so no extra masking is needed here.
async fn trace_export(
    State(state): State<ConversationRouterState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<String>,
    Query(query): Query<TraceQuery>,
) -> Result<Response, ApiError> {
    let filters = roseui_db::SessionEventFilters {
        event_kind: query.event_kind.clone(),
        model: query.model.clone(),
        from_ts: query.from_ts,
        to_ts: query.to_ts,
        limit: query.limit.unwrap_or(200),
    };
    let events = state
        .service
        .list_session_events(&user.id, &id, filters)
        .await
        .map_err(ApiError::from)?;
    let questions = state
        .service
        .list_user_questions(&user.id, &id)
        .await
        .map_err(ApiError::from)?;
    let timeline = crate::trace_timeline::build_timeline(&events, &questions);

    let fmt = TraceExportFormat::parse(&query.fmt);
    match fmt {
        TraceExportFormat::Json => {
            let body = serde_json::to_vec_pretty(&serde_json::json!({
                "conversation_id": id,
                "events": events.iter().map(|e| serde_json::json!({
                    "id": e.id,
                    "turn_seq": e.turn_seq,
                    "event_kind": e.event_kind,
                    "role": e.role,
                    "model": e.model,
                    "input": serde_json::from_str::<serde_json::Value>(&e.input_json).unwrap_or(serde_json::Value::Null),
                    "output": serde_json::from_str::<serde_json::Value>(&e.output_json).unwrap_or(serde_json::Value::Null),
                    "token_usage": serde_json::from_str::<serde_json::Value>(&e.token_usage_json).unwrap_or(serde_json::Value::Null),
                    "status": e.status,
                    "created_at": e.created_at,
                })).collect::<Vec<_>>(),
                "timeline": timeline,
            }))
            .map_err(|e| ApiError::Internal(format!("serialize trace export: {e}")))?;
            Ok(Response::builder()
                .header(CONTENT_TYPE, "application/json; charset=utf-8")
                .header(CONTENT_DISPOSITION, format!("attachment; filename=\"trace-{id}.json\""))
                .body(Body::from(body))
                .map_err(|e| ApiError::Internal(format!("build response: {e}")))?)
        }
        TraceExportFormat::Markdown => {
            let md = render_trace_markdown(&id, &events, &timeline);
            Ok(Response::builder()
                .header(CONTENT_TYPE, "text/markdown; charset=utf-8")
                .header(CONTENT_DISPOSITION, format!("attachment; filename=\"trace-{id}.md\""))
                .body(Body::from(md))
                .map_err(|e| ApiError::Internal(format!("build response: {e}")))?)
        }
    }
}

/// Render the trace into a human-readable Markdown document.
fn render_trace_markdown(conversation_id: &str, events: &[SessionEventRow], timeline: &serde_json::Value) -> String {
    let mut out = String::new();
    out.push_str("# 对话轨迹导出\n\n");
    out.push_str(&format!("会话 ID: `{conversation_id}`\n"));
    out.push_str(&format!("事件数: {}\n\n", events.len()));
    out.push_str("---\n\n");

    if let Some(turns) = timeline.get("turns").and_then(|t| t.as_array()) {
        for turn in turns {
            let idx = turn.get("turn").and_then(|v| v.as_u64()).unwrap_or(0);
            let time = turn.get("time").and_then(|v| v.as_str()).unwrap_or("");
            let question = turn.get("question").and_then(|v| v.as_str()).unwrap_or("");
            out.push_str(&format!("## 第 {} 轮 · {}\n\n", idx, time));
            if !question.is_empty() {
                out.push_str(&format!("**提问**: {}\n\n", question));
            }
            if let Some(steps) = turn.get("steps").and_then(|s| s.as_array()) {
                for step in steps {
                    let kind = step.get("kind").and_then(|v| v.as_str()).unwrap_or("");
                    let summary = step.get("summary").and_then(|v| v.as_str()).unwrap_or("");
                    let status = step.get("status").and_then(|v| v.as_str()).unwrap_or("");
                    out.push_str(&format!("- **[{kind}]** {summary} _{status}_\n"));
                }
            }
            out.push('\n');
        }
    }

    out.push_str("---\n\n## 原始事件明细\n\n");
    for e in events {
        let input = e.input_json.chars().take(200).collect::<String>();
        let output = e.output_json.chars().take(200).collect::<String>();
        out.push_str(&format!(
            "- `[{}]` **{}** ({}) model=`{}` status=`{}`\n  - in: {}\n  - out: {}\n",
            e.turn_seq,
            e.event_kind,
            e.role.clone().unwrap_or_default(),
            e.model.clone().unwrap_or_default(),
            e.status.clone().unwrap_or_default(),
            input,
            output
        ));
    }
    out
}

/// Returns the session trace (监控录像) for a single conversation. Per-conversation
/// scoped only — never crosses sessions. Inputs/outputs were redacted at write
/// time, so no further scrubbing is needed here.
///
/// Response keeps `data`（原始事件，供迷你矩阵）原样不动，并在顶层新增
/// `timeline`：以「用户提问」为锚聚合出的轮次步骤，只供前端人性化列表使用。
async fn trace(
    State(state): State<ConversationRouterState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<String>,
    Query(query): Query<TraceQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let filters = roseui_db::SessionEventFilters {
        event_kind: query.event_kind,
        model: query.model,
        from_ts: query.from_ts,
        to_ts: query.to_ts,
        limit: query.limit.unwrap_or(200),
    };
    let events = state
        .service
        .list_session_events(&user.id, &id, filters)
        .await
        .map_err(ApiError::from)?;

    // 用户提问：时间线轮次锚点（与过滤后的 data 无关，取自 messages 表）。
    let questions = state
        .service
        .list_user_questions(&user.id, &id)
        .await
        .map_err(ApiError::from)?;

    // 人性化时间线（与过滤后的 events 同源，保证矩阵与列表始终一致）。
    let timeline = crate::trace_timeline::build_timeline(&events, &questions);

    // Project rows into a stable JSON envelope the frontend matrix can render.
    let items: Vec<serde_json::Value> = events
        .into_iter()
        .map(|e| {
            serde_json::json!({
                "id": e.id,
                "turn_seq": e.turn_seq,
                "event_kind": e.event_kind,
                "role": e.role,
                "model": e.model,
                "input": serde_json::from_str::<serde_json::Value>(&e.input_json).unwrap_or(serde_json::Value::Object(Default::default())),
                "output": serde_json::from_str::<serde_json::Value>(&e.output_json).unwrap_or(serde_json::Value::Object(Default::default())),
                "token_usage": serde_json::from_str::<serde_json::Value>(&e.token_usage_json).unwrap_or(serde_json::Value::Object(Default::default())),
                "status": e.status,
                "created_at": e.created_at,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "success": true,
        "data": items,
        "timeline": timeline,
    })))
}

#[derive(serde::Deserialize)]
struct MessagePathParams {
    id: String,
    #[serde(rename = "messageId")]
    message_id: String,
}

async fn get_msg(
    State(state): State<ConversationRouterState>,
    Extension(user): Extension<CurrentUser>,
    Path(params): Path<MessagePathParams>,
) -> Result<Json<ApiResponse<MessageResponse>>, ApiError> {
    let result = state
        .service
        .get_message(&user.id, &params.id, &params.message_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(ApiResponse::ok(result)))
}

async fn send_msg(
    State(state): State<ConversationRouterState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<String>,
    body: Result<Json<SendMessageRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<ApiResponse<SendMessageResponse>>), ApiError> {
    let Json(req) = body.map_err(ApiError::from)?;
    let response = state
        .service
        .send_message(&user.id, &id, req, &state.task_manager)
        .await
        .map_err(ApiError::from)?;
    Ok((StatusCode::ACCEPTED, Json(ApiResponse::ok(response))))
}

async fn list_artifacts(
    State(state): State<ConversationRouterState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<ConversationArtifactListResponse>>, ApiError> {
    let result = state
        .service
        .list_artifacts(&user.id, &id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(ApiResponse::ok(result)))
}

#[derive(serde::Deserialize)]
struct ArtifactPathParams {
    id: String,
    #[serde(rename = "artifactId")]
    artifact_id: String,
}

async fn update_artifact(
    State(state): State<ConversationRouterState>,
    Extension(user): Extension<CurrentUser>,
    Path(params): Path<ArtifactPathParams>,
    body: Result<Json<UpdateConversationArtifactRequest>, JsonRejection>,
) -> Result<Json<ApiResponse<ConversationArtifactResponse>>, ApiError> {
    let Json(req) = body.map_err(ApiError::from)?;
    let artifact = state
        .service
        .update_artifact(&user.id, &params.id, &params.artifact_id, req)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(ApiResponse::ok(artifact)))
}

async fn compact(
    State(state): State<ConversationRouterState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<CompactConversationResponse>>, ApiError> {
    let response = state
        .service
        .compact_conversation(&user.id, &id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(ApiResponse::ok(response)))
}

async fn cancel(
    State(state): State<ConversationRouterState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<String>,
    body: Result<Json<CancelConversationRequest>, JsonRejection>,
) -> Result<Json<ApiResponse<CancelConversationResponse>>, ApiError> {
    let Json(req) = body.map_err(ApiError::from)?;
    let response = state
        .service
        .cancel(&user.id, &id, &req.turn_id, &state.task_manager)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(ApiResponse::ok(response)))
}

async fn ensure_runtime(
    State(state): State<ConversationRouterState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<EnsureConversationRuntimeResponse>>, ApiError> {
    let response = state
        .service
        .ensure_runtime(&user.id, &id, &state.task_manager)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(ApiResponse::ok(response)))
}

async fn active_lease(
    State(state): State<ConversationRouterState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    state
        .service
        .renew_active_lease(&user.id, &id, &state.active_leases)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(ApiResponse::success()))
}

async fn search_messages(
    State(state): State<ConversationRouterState>,
    Extension(user): Extension<CurrentUser>,
    Query(query): Query<SearchMessagesQuery>,
) -> Result<Json<ApiResponse<MessageSearchResponse>>, ApiError> {
    let result = state
        .service
        .search_messages(&user.id, query)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(ApiResponse::ok(result)))
}

// ── Confirmation handlers ─────────────────────────────────────────

async fn list_confirmations(
    State(state): State<ConversationRouterState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<ConfirmationListResponse>>, ApiError> {
    let items = state
        .service
        .list_confirmations(&user.id, &id, &state.task_manager)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(ApiResponse::ok(items)))
}

#[derive(serde::Deserialize)]
struct ConfirmPathParams {
    id: String,
    #[serde(rename = "callId")]
    call_id: String,
}

async fn confirm(
    State(state): State<ConversationRouterState>,
    Extension(user): Extension<CurrentUser>,
    Path(params): Path<ConfirmPathParams>,
    body: Result<Json<ConfirmRequest>, JsonRejection>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    let Json(req) = body.map_err(ApiError::from)?;
    state
        .service
        .confirm(&user.id, &params.id, &params.call_id, req, &state.task_manager)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(ApiResponse::success()))
}

async fn check_approval(
    State(state): State<ConversationRouterState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<String>,
    Query(query): Query<ApprovalCheckQuery>,
) -> Result<Json<ApiResponse<ApprovalCheckResponse>>, ApiError> {
    if query.action.trim().is_empty() {
        return Err(ApiError::BadRequest("action must not be empty".into()));
    }

    let result = state
        .service
        .check_approval(
            &user.id,
            &id,
            &query.action,
            query.command_type.as_deref(),
            &state.task_manager,
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(ApiResponse::ok(result)))
}

async fn active_count(
    State(state): State<ConversationRouterState>,
    Extension(user): Extension<CurrentUser>,
) -> Result<Json<ApiResponse<ActiveCountResponse>>, ApiError> {
    let count = state.service.active_count_for_user(&user.id).await?;
    Ok(Json(ApiResponse::ok(ActiveCountResponse { count })))
}

#[cfg(test)]
mod error_mapping_tests {
    use super::*;

    #[test]
    fn conversation_not_found_maps_to_app_not_found() {
        let app = ApiError::from(ConversationError::NotFound { id: "conv_1".into() });
        assert!(matches!(app, ApiError::NotFound(message) if message == "Conversation conv_1 not found"));
    }

    #[test]
    fn cross_account_reference_maps_to_stable_conflict_code() {
        let app = ApiError::from(ConversationError::Busy {
            reason: "CROSS_ACCOUNT_REFERENCE: acp_session conversation 'conv-1' belongs to another user".into(),
        });

        assert_eq!(app.status_code(), StatusCode::CONFLICT);
        assert_eq!(app.error_code(), "CROSS_ACCOUNT_REFERENCE");
    }

    #[test]
    fn conversation_archived_maps_to_app_conversation_archived() {
        let app = ApiError::from(ConversationError::Archived {
            id: "conv_1".into(),
            reason: "legacy runtime".into(),
        });
        assert!(matches!(app, ApiError::ConversationArchived(message) if message == "legacy runtime"));
    }

    #[test]
    fn message_not_found_maps_to_app_not_found() {
        let app = ApiError::from(ConversationError::MessageNotFound { id: "msg_1".into() });
        assert!(matches!(app, ApiError::NotFound(message) if message == "Message msg_1 not found"));
    }

    #[test]
    fn artifact_not_found_maps_to_app_not_found() {
        let app = ApiError::from(ConversationError::ArtifactNotFound {
            id: "artifact_1".into(),
        });
        assert!(matches!(app, ApiError::NotFound(message) if message == "Artifact artifact_1 not found"));
    }

    #[test]
    fn active_agent_not_found_maps_to_app_not_found() {
        let app = ApiError::from(ConversationError::ActiveAgentNotFound {
            conversation_id: "conv_1".into(),
        });
        assert!(matches!(app, ApiError::NotFound(message) if message == "No active agent for this conversation"));
    }

    #[test]
    fn conversation_api_error_compat_preserves_special_codes() {
        let app = ApiError::from(ConversationError::WorkspacePathRuntimeUnavailable {
            path: "/tmp/my project".into(),
        });
        assert!(matches!(
            app,
            ApiError::WorkspacePathRuntimeUnavailable(message) if message == "/tmp/my project"
        ));
    }

    #[test]
    fn openclaw_gateway_unreachable_maps_to_coded_bad_gateway() {
        let app = ApiError::from(ConversationError::OpenClawGatewayUnreachable {
            detail: "OpenClaw Gateway is not running or cannot be reached at 127.0.0.1:18789.".into(),
        });

        assert_eq!(app.status_code(), StatusCode::BAD_GATEWAY);
        assert_eq!(app.error_code(), "USER_AGENT_OPENCLAW_GATEWAY_UNREACHABLE");
        assert_eq!(app.public_message(), "OpenClaw Gateway is not reachable");
        let details = app.error_details().expect("details should be present");
        assert_eq!(details["backend"], "openclaw");
        assert_eq!(details["port"], 18789);
    }
}
