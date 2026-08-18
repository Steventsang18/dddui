//! Rupoo agent engine adapter.
//!
//! `RupooAgentManager` implements DoDidDoneUi's [`crate::agent_task::IAgentTask`]
//! contract by wrapping the local `rupoo` engine crate (path dependency). The
//! engine crate name (`rupoo`) is a build-time dependency label only — every
//! user-visible surface exposed through this manager is roseui-branded.
//!
//! The manager owns a `rupoo::agent::Agent` instance and drives it through the
//! engine's headless `agent_chat` entry point. Engine events (`AgentEvent`)
//! are translated into DoDidDoneUi's `AgentStreamEvent` wire protocol via the
//! [`translate`] module and fanned out on a broadcast channel.

pub mod translate;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use roseui_common::{AgentKillReason, AgentType, Confirmation, ConversationStatus, TimestampMs, now_ms};
use rupoo::agent::ToolExecutor;
use roseui_api_types::{AgentModeResponse, ConfigOptionConfirmation, GetConfigOptionsResponse, SetConfigOptionResponse, SlashCommandItem};
use tokio::sync::broadcast;
use tokio::sync::mpsc;

use crate::agent_task::IAgentTask;
use crate::error::AgentError;
use crate::protocol::events::{AcpPermissionEventData, AgentStreamEvent};
use crate::protocol::send_error::AgentSendError;
use crate::types::{AionrsResolvedConfig, SendMessageData};

use translate::EventTranslator;

/// In-process manager that drives the Rupoo engine for one conversation.
pub struct RupooAgentManager {
    conversation_id: String,
    workspace: String,
    event_tx: broadcast::Sender<AgentStreamEvent>,
    last_activity: Mutex<TimestampMs>,
    /// Shared engine instance (its `agent_chat` takes `&self`).
    agent: Arc<rupoo::agent::Agent>,
    /// Per-turn translation state (call-id sequencing for tool events).
    translator: Arc<Mutex<EventTranslator>>,
    /// Rolling conversation history fed back into the engine each turn.
    history: Arc<Mutex<rupoo::llm::ConversationHistory>>,
    /// Set by `cancel()` to abort an in-flight `agent_chat` loop.
    cancelled: Arc<AtomicBool>,
    /// Whether the turn has started (status reporting).
    started: Mutex<Option<ConversationStatus>>,
    /// Interactive approval gate shared with the engine's tool executor.
    approval_gate: Arc<rupoo::approval::ApprovalGate>,
    /// Industry-template system prompt (highest precedence) captured at build
    /// time, replayed on every turn via `agent_chat(system_prompt_override)`.
    industry_system_prompt: Option<String>,
    /// Pending confirmation cards surfaced to the frontend.
    confirmations: Arc<RwLock<Vec<Confirmation>>>,
    /// Host-side receiver for approval notices from the engine. Wrapped in
    /// `Option` so the drain task (spawned once on the first `send_message`)
    /// can take ownership of it exactly once.
    approval_rx: Mutex<Option<mpsc::UnboundedReceiver<rupoo::approval::ApprovalRequest>>>,
    /// Handle of the long-lived approval drain task, started once per manager.
    approval_drain: Mutex<Option<tokio::task::JoinHandle<()>>>,
    /// Max agentic tool-call turns per user request. Mirrors rupoo's own
    /// `DEFAULT_MAX_TURNS = 50` so complex multi-step tasks don't hit the
    /// `MaxTurnError` prematurely. Resolved from `config.max_turns` (which the
    /// upstream `AionrsBuildExtra.max_turns` override can raise), falling back to
    /// 50 when unset. This is the *loop* cap fed into `agent_chat` /
    /// `multi_turn`, intentionally distinct from the `ConversationHistory`
    /// retention window (also sized from `config.max_turns`).
    max_turns: usize,
    /// Model context window (tokens) used as the denominator of the UI
    /// context-usage ring. Resolved once from the model label — NOT the
    /// engine's internal `TokenBudget` (a 4096-token prompt-assembly budget
    /// that made every conversation look nearly full).
    context_window: u64,
}

/// Best-effort lookup of a model's real context-window size (tokens) from its
/// label, mirroring rupoo's own CLI table. The engine's `TokenBudget` (4096)
/// is only a prompt-assembly budget, so the UI ring must use the model window
/// as its denominator. Unknown labels fall back to the aionrs compact default
/// (200k) — a conservative, never-red under-estimate of the real window.
fn model_context_window(label: &str) -> Option<u64> {
    let l = label.to_ascii_lowercase();
    const TABLE: &[(&str, u64)] = &[
        ("claude", 200_000),
        ("gpt-4o", 128_000),
        ("gpt-4", 128_000),
        ("o1", 200_000),
        ("gpt-3.5", 16_385),
        ("gemini", 1_000_000),
        ("deepseek", 64_000),
        ("qwen", 32_768),
        ("llama", 8_192),
        ("mistral", 32_000),
        ("yi-", 200_000),
        ("glm", 128_000),
    ];
    TABLE
        .iter()
        .find(|(key, _)| l.contains(key))
        .map(|(_, window)| *window)
}

impl RupooAgentManager {
    /// Build a manager bound to `conversation_id` / `workspace`, constructing
    /// the underlying engine from the resolved RoseUi provider configuration.
    pub async fn new(
        conversation_id: String,
        workspace: String,
        config: AionrsResolvedConfig,
    ) -> Result<Self, AgentError> {
        let (event_tx, _rx) = broadcast::channel(512);

        // ── Provider configuration → Rupoo LlmConfig ──
        let provider = map_provider(&config.provider);
        let mut llm_config = rupoo::llm::LlmConfig::new(provider, Some(config.api_key.clone()));
        llm_config.model = config.model.clone();
        llm_config.base_url = config.base_url.clone();
        if let Some(max_tokens) = config.max_tokens {
            llm_config.max_tokens = max_tokens;
        }

        // ── Persistence (engine's own rusqlite TaskRepo) ──
        let db_path = std::path::Path::new(&workspace)
            .join(".roseui_agent.db")
            .to_string_lossy()
            .into_owned();
        let repo = Arc::new(
            rupoo::db::TaskRepo::new(&db_path)
                .map_err(|e| AgentError::internal(format!("Rupoo TaskRepo init failed: {e}")))?,
        );

        // ── Tool executor: reuse Rupoo's built-in jail-aware tool set ──
        // Interactive approval gate: high-risk tools pause for user confirmation.
        let (approval_gate, approval_rx) = rupoo::approval::ApprovalGate::new();
        let approval_gate = Arc::new(approval_gate);

        // When an industry template is active, derive the tool executor (and its
        // embedded SafetyContext) from the *resolved* SafetySection + AgentProfile.
        // This makes template policies actually bite: command jail, approval
        // policy, and tool allow/exclude scoping all flow into the engine.
        let (executor, safety): (rupoo::mcp::McpToolExecutor, rupoo::safety::SafetyContext) =
            if let (Some(section), Some(profile)) =
                (config.industry_safety.as_ref(), config.industry_profile.as_ref())
            {
                let exec = rupoo::mcp::McpToolExecutor::with_safety_section(
                    section,
                    Some(profile),
                    Some(approval_gate.clone()),
                );
                let safety = exec.safety().clone();
                (exec, safety)
            } else {
                let safety = rupoo::safety::SafetyContext::default();
                let exec = rupoo::mcp::McpToolExecutor::with_safety_and_approval(
                    safety.clone(),
                    approval_gate.clone(),
                );
                (exec, safety)
            };

        // ── roseui: consume resolved extra_mcp_servers (wiki/team stdio MCP) ──
        // Connect each stdio server via rupoo's built-in MCP client and register
        // its tools under the standard `mcp__<server>__<tool>` wire naming so the
        // agent can actually call them. Non-stdio transports are skipped.
        let extra_tools = register_external_mcp_servers(&executor, &config.extra_mcp_servers).await;

        let approval_tool_names = safety.approval_required_tools();
        let tool_executor: Arc<dyn ToolExecutor> = Arc::new(executor);

        // ── Engine assembly ──
        let agent = rupoo::agent::Agent::with_safety(repo, tool_executor, safety);
        let gateway = rupoo::llm::LlmGateway::with_jail(
            llm_config,
            std::path::PathBuf::from(&workspace),
        )
        .with_extra_tools(extra_tools);
        let agent = Arc::new(agent.with_llm(gateway));

        let history = rupoo::llm::ConversationHistory::new(config.max_turns.unwrap_or(50))
            .with_token_budget(60_000);

        let translator = EventTranslator::new().with_approval_tools(approval_tool_names.clone());

        Ok(Self {
            conversation_id,
            workspace,
            event_tx,
            last_activity: Mutex::new(now_ms()),
            agent,
            translator: Arc::new(Mutex::new(translator)),
            history: Arc::new(Mutex::new(history)),
            cancelled: Arc::new(AtomicBool::new(false)),
            started: Mutex::new(None),
            approval_gate,
            industry_system_prompt: config.industry_system_prompt.clone(),
            confirmations: Arc::new(RwLock::new(Vec::new())),
            approval_rx: Mutex::new(Some(approval_rx)),
            approval_drain: Mutex::new(None),
            max_turns: config.max_turns.unwrap_or(50),
            context_window: model_context_window(&config.model).unwrap_or(200_000),
        })
    }

    /// Fan a single wire event out to all subscribers.
    pub(crate) fn emit(&self, event: AgentStreamEvent) -> usize {
        *self.last_activity.lock().unwrap() = now_ms();
        self.event_tx.send(event).unwrap_or(0)
    }
}

/// Connect external stdio MCP servers declared on the resolved config and
/// register their tools into the engine executor under the standard
/// `mcp__<server>__<tool>` wire naming. Non-stdio transports (SSE/HTTP) are
/// skipped with a warning — rupoo's MCP client only drives stdio subprocesses.
async fn register_external_mcp_servers(
    executor: &rupoo::mcp::McpToolExecutor,
    servers: &std::collections::HashMap<String, aion_config::config::McpServerConfig>,
) -> Vec<rupoo::mcp::McpToolDyn> {
    if servers.is_empty() {
        return Vec::new();
    }

    let mut stdio: std::collections::HashMap<String, rupoo::config::McpServerConfig> =
        std::collections::HashMap::new();
    for (name, cfg) in servers {
        match (&cfg.transport, &cfg.command) {
            (aion_config::config::TransportType::Stdio, Some(command)) => {
                stdio.insert(
                    name.clone(),
                    rupoo::config::McpServerConfig {
                        command: command.clone(),
                        args: cfg.args.clone().unwrap_or_default(),
                        env: cfg.env.clone().unwrap_or_default(),
                    },
                );
            }
            _ => {
                tracing::warn!(
                    server = %name,
                    transport = ?cfg.transport,
                    "skipping MCP server: only stdio transport is supported by the rupoo client"
                );
            }
        }
    }
    if stdio.is_empty() {
        return Vec::new();
    }

    let client = Arc::new(rupoo::mcp_client::McpClientManager::new());
    let results = client.connect_all(&stdio).await;
    let mut extra_tools: Vec<rupoo::mcp::McpToolDyn> = Vec::new();
    for (name, result) in results {
        match result {
            Ok(tools) => {
                tracing::info!(
                    server = %name,
                    tools = tools.len(),
                    "external stdio MCP server connected"
                );
                executor
                    .register_mcp_client_tools(client.clone(), &name, &tools)
                    .await;
                extra_tools.extend(
                    tools.iter().map(|info| {
                        rupoo::mcp::McpToolDyn::new(client.clone(), &name, info)
                    }),
                );
            }
            Err(error) => {
                tracing::warn!(
                    server = %name,
                    error = %error,
                    "failed to connect external stdio MCP server"
                );
            }
        }
    }
    extra_tools
}

#[async_trait::async_trait]
impl IAgentTask for RupooAgentManager {
    fn agent_type(&self) -> AgentType {
        AgentType::Aionrs
    }

    fn conversation_id(&self) -> &str {
        &self.conversation_id
    }

    fn workspace(&self) -> &str {
        &self.workspace
    }

    fn status(&self) -> Option<ConversationStatus> {
        *self.started.lock().unwrap()
    }

    fn last_activity_at(&self) -> TimestampMs {
        *self.last_activity.lock().unwrap()
    }

    fn subscribe(&self) -> broadcast::Receiver<AgentStreamEvent> {
        self.event_tx.subscribe()
    }

    async fn send_message(&self, data: SendMessageData) -> Result<(), AgentSendError> {
        // Mark the turn as running.
        *self.started.lock().unwrap() = Some(ConversationStatus::Running);

        // Anchor history with the latest user message.
        {
            let mut history = self.history.lock().unwrap();
            history.push_user(&data.content);
        }

        // Emit the turn-start marker.
        self.emit(translate::EventTranslator::start_event(Some(self.conversation_id.clone())));

        // Drive the engine. `agent_chat` streams via the `on_event` callback;
        // we translate and fan each event out on the broadcast channel.
        let agent = Arc::clone(&self.agent);
        let cancelled = Arc::clone(&self.cancelled);
        let event_tx = self.event_tx.clone();
        let translator = Arc::clone(&self.translator);
        let user_message = data.content.clone();
        let max_turns = self.max_turns;
        let safe_mode = true;

        // Drain approval notices from the engine concurrently with the turn.
        // Each notice becomes an `AcpPermission` confirmation card so the user
        // can approve/deny the high-risk tool before it executes.
        let notice_event_tx = self.event_tx.clone();
        let notice_confirmations = Arc::clone(&self.confirmations);
        let notice_translator = Arc::clone(&self.translator);

        // Start the approval drain exactly once for the lifetime of this manager.
        // The drain owns the engine's notice receiver; it must not be aborted
        // between turns or re-taken on the next `send_message` (that panicked
        // with "approval drain already started" and killed the tokio worker).
        {
            let mut drain_guard = self.approval_drain.lock().unwrap();
            if drain_guard.is_none() {
                let rx = self.approval_rx.lock().unwrap().take();
                if let Some(mut approval_rx) = rx {
                    let task = tokio::spawn(async move {
                        while let Some(req) = approval_rx.recv().await {
                            // Tell the translator to pair this approval's result with its id.
                            if let Ok(t) = notice_translator.lock() {
                                t.on_approval_request(req.call_id.clone());
                            }
                            let confirmation = build_confirmation(&req);
                            if let Ok(mut confs) = notice_confirmations.write() {
                                confs.push(confirmation.clone());
                            }
                            let _ = notice_event_tx.send(AgentStreamEvent::AcpPermission(
                                AcpPermissionEventData::Confirmation(confirmation),
                            ));
                        }
                    });
                    *drain_guard = Some(task);
                }
            }
        }

        let on_event = move |event: rupoo::llm::AgentEvent| {
            if cancelled.load(Ordering::SeqCst) {
                return;
            }
            let mut t = translator.lock().unwrap();
            for wire in t.translate(event) {
                let _ = event_tx.send(wire);
            }
        };

        // Snapshot history into an owned value so no MutexGuard crosses the
        // `agent_chat` await (std MutexGuard is not Send across awaits).
        let history_snapshot = self.history.lock().unwrap().clone();

        let result = agent
            .agent_chat(&user_message, &history_snapshot, max_turns, safe_mode, on_event, None, self.industry_system_prompt.clone())
            .await;

        // The approval drain stays alive across turns (it owns the engine's
        // notice receiver). It is only aborted when the manager is dropped.

        match result {
            Ok((response, usage)) => {
                // Record assistant turn in history for next round.
                self.history.lock().unwrap().push_assistant(&response);
                self.emit(translate::EventTranslator::finish_event(
                    Some(self.conversation_id.clone()),
                    Some(usage),
                ));
                *self.started.lock().unwrap() = Some(ConversationStatus::Finished);
                Ok(())
            }
            Err(e) => {
                tracing::error!(conversation_id = %self.conversation_id, error = %e, "rupoo agent_chat failed");
                self.emit(AgentStreamEvent::Error(
                    roseui_api_types::AgentStreamErrorData::legacy(
                        format!("engine error: {e}"),
                        None,
                    ),
                ));
                *self.started.lock().unwrap() = Some(ConversationStatus::Finished);
                Ok(())
            }
        }
    }

    async fn cancel(&self) -> Result<(), AgentError> {
        self.cancelled.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn kill(&self, _reason: Option<AgentKillReason>) -> Result<(), AgentError> {
        self.cancelled.store(true, Ordering::SeqCst);
        Ok(())
    }
}

impl Drop for RupooAgentManager {
    fn drop(&mut self) {
        // Stop the long-lived approval drain task so its receiver/Arc clones
        // don't leak. `JoinHandle::abort` is safe to call even if the task has
        // already finished.
        if let Ok(Some(handle)) = self.approval_drain.lock().map(|mut guard| guard.take()) {
            handle.abort();
        }
    }
}
///
/// Tool approval is delegated to the Rupoo engine's interactive `ApprovalGate`:
/// high-risk tools pause and surface an `AcpPermission` confirmation card
/// (via [`get_confirmations`] / the live event in `send_message`); [`confirm`]
/// resolves the gate. Low-risk tools run without prompting, and forbidden
/// commands are still blocked by the engine's jail.
impl RupooAgentManager {
    pub fn kill_and_wait(
        &self,
        reason: Option<AgentKillReason>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> {
        let _ = self.kill(reason);
        Box::pin(async {})
    }

    pub fn get_confirmations(&self) -> Vec<Confirmation> {
        self.confirmations
            .read()
            .map(|c| c.clone())
            .unwrap_or_default()
    }

    /// Live context-window occupancy for the #14 consumption ring. Reads the
    /// engine's unified `ConversationContext` directly (no wire round-trip):
    /// `estimated_context_tokens()` is the assembled prompt+history token count,
    /// while the capacity is the *model* context window (resolved from the
    /// model label at build time) — NOT the engine's internal `TokenBudget`
    /// (a 4096-token prompt-assembly budget). Returns `(used, size)`.
    pub fn get_usage(&self) -> (u64, u64) {
        let ctx = self.agent.context();
        let used = ctx.estimated_context_tokens() as u64;
        (used, self.context_window)
    }

    pub fn confirm(
        &self,
        _msg_id: &str,
        call_id: &str,
        data: serde_json::Value,
        always_allow: bool,
    ) -> Result<(), AgentError> {
        // Capture the tool name before dropping the card.
        let action = self
            .confirmations
            .read()
            .ok()
            .and_then(|c| c.iter().find(|c| c.call_id == call_id).cloned())
            .and_then(|c| c.action);

        // Remove the confirmation card.
        if let Ok(mut confs) = self.confirmations.write() {
            confs.retain(|c| c.call_id != call_id);
        }

        let value = data
            .get("value")
            .and_then(|v| v.as_str())
            .unwrap_or("cancel");
        let approved = value != "cancel";

        if approved && (always_allow || value == "proceed_always") {
            // Remember the tool so future calls skip the prompt this session.
            if let Some(action) = action {
                self.approval_gate.set_auto_approved(&action);
            }
        }

        self.approval_gate.resolve(call_id, approved);
        Ok(())
    }

    pub fn check_approval(&self, _action: &str, _command_type: Option<&str>) -> bool {
        false
    }

    pub async fn mode(&self) -> Result<AgentModeResponse, AgentError> {
        Ok(AgentModeResponse {
            mode: "default".to_owned(),
            initialized: true,
        })
    }

    pub async fn get_slash_commands(&self) -> Result<Vec<SlashCommandItem>, AgentError> {
        Ok(Vec::new())
    }

    pub async fn config_options(&self) -> Result<GetConfigOptionsResponse, AgentError> {
        Ok(GetConfigOptionsResponse {
            config_options: Vec::new(),
        })
    }

    pub async fn set_config_option(
        &self,
        _option_id: &str,
        _value: &str,
    ) -> Result<SetConfigOptionResponse, AgentError> {
        Ok(SetConfigOptionResponse {
            confirmation: ConfigOptionConfirmation::Observed,
            config_options: Some(Vec::new()),
        })
    }
}

/// Map an RoseUi provider label onto Rupoo's `LlmProvider` enum.
///
/// Rupoo supports a fixed provider set; anything unrecognized falls back to
/// OpenAI (the most common OpenAI-compatible proxy target).
fn map_provider(label: &str) -> rupoo::llm::LlmProvider {
    match label.to_ascii_lowercase().as_str() {
        "anthropic" => rupoo::llm::LlmProvider::Anthropic,
        "deepseek" => rupoo::llm::LlmProvider::DeepSeek,
        "gemini" => rupoo::llm::LlmProvider::Gemini,
        "openai" => rupoo::llm::LlmProvider::OpenAI,
        "ollama" => rupoo::llm::LlmProvider::Ollama,
        _ => rupoo::llm::LlmProvider::OpenAI,
    }
}

/// Build a frontend confirmation card for a pending Rupoo tool approval.
///
/// Mirrors the option set used by the Aionrs `BackendProtocolSink` so the
/// renderer shows the same allow-once / allow-always / deny choices.
fn build_confirmation(req: &rupoo::approval::ApprovalRequest) -> Confirmation {
    use roseui_common::ConfirmationOption;
    use serde_json::json;

    let description = match &req.args {
        serde_json::Value::String(s) if !s.trim().is_empty() => s.clone(),
        serde_json::Value::Object(_) => serde_json::to_string_pretty(&req.args).unwrap_or_default(),
        _ => String::new(),
    };

    Confirmation {
        id: roseui_common::generate_id(),
        call_id: req.call_id.clone(),
        title: Some(format!("roseui wants to use: {}", req.tool_name)),
        action: Some(req.tool_name.clone()),
        description,
        command_type: Some("execute".to_string()),
        options: vec![
            ConfirmationOption {
                label: "messages.confirmation.yesAllowOnce".to_string(),
                value: json!("proceed_once"),
                params: None,
            },
            ConfirmationOption {
                label: "messages.confirmation.yesAllowAlways".to_string(),
                value: json!("proceed_always"),
                params: None,
            },
            ConfirmationOption {
                label: "messages.confirmation.no".to_string(),
                value: json!("cancel"),
                params: None,
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::events::ToolCallStatus;
    use translate::EventTranslator;

    #[test]
    fn text_delta_maps_to_text_event() {
        let mut t = EventTranslator::new();
        let out = t.translate(rupoo::llm::AgentEvent::TextDelta("hi".into()));
        assert_eq!(out.len(), 1);
        match &out[0] {
            AgentStreamEvent::Text(d) => assert_eq!(d.content, "hi"),
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn tool_call_and_result_pair_within_one_turn() {
        let mut t = EventTranslator::new();
        let call = t.translate(rupoo::llm::AgentEvent::ToolCall {
            tool_name: "shell".into(),
            args: "{\"cmd\":\"ls\"}".into(),
        });
        let result = t.translate(rupoo::llm::AgentEvent::ToolResult {
            tool_name: "shell".into(),
            result: "ok".into(),
        });
        assert_eq!(call.len(), 1);
        assert_eq!(result.len(), 1);
        match (&call[0], &result[0]) {
            (AgentStreamEvent::ToolCall(c), AgentStreamEvent::ToolCall(r)) => {
                assert_eq!(c.call_id, r.call_id);
                assert_eq!(c.status, ToolCallStatus::Running);
                assert_eq!(r.status, ToolCallStatus::Completed);
                assert_eq!(r.output.as_deref(), Some("ok"));
            }
            _ => panic!("expected ToolCall events"),
        }
    }
}
