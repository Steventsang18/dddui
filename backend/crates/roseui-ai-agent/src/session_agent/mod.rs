//! `SessionAgentTask` — adapts the clean-slate `roseui_session::SessionBackend`
//! (direct-CLI actor model for claude/codex) to origin's `IAgentTask` contract.

mod constants;
mod runtime;
mod build;
mod catalog;
mod pump;
mod persist;
mod translate;

pub(crate) use constants::*;
pub(crate) use runtime::*;
pub(crate) use build::*;
pub(crate) use catalog::*;
pub(crate) use pump::*;
pub(crate) use persist::*;
pub(crate) use translate::*;

pub(crate) use std::sync::Arc;
pub(crate) use std::sync::atomic::{AtomicI64, Ordering};

pub(crate) use roseui_common::{AgentKillReason, ConversationStatus, TimestampMs, now_ms};
pub(crate) use roseui_session::{
    BackendError, Command, CommandMeta, ContentBlock, SessionBackend, SessionEnvelope, SessionEvent, ToolResultContent,
};
pub(crate) use futures_util::stream::BoxStream;
pub(crate) use tokio::sync::broadcast;

pub(crate) use crate::agent_task::IAgentTask;
pub(crate) use crate::error::AgentError;
pub(crate) use crate::protocol::events::session_updates::AvailableCommandsEventData;
pub(crate) use crate::protocol::events::session_updates::ThinkingEventData;
pub(crate) use crate::protocol::events::tool_call::{ToolCallEventData, ToolCallStatus};
pub(crate) use crate::protocol::events::{
    AgentStreamEvent, FinishEventData, StartEventData, TextEventData, TipType, TipsEventData,
};
pub(crate) use crate::protocol::send_error::AgentSendError;
pub(crate) use crate::shared_kernel::PersistedSessionState;
pub(crate) use crate::types::SendMessageData;
pub(crate) use roseui_api_types::AcpBuildExtra;
pub(crate) use roseui_common::AgentType;
pub(crate) use roseui_db::{IAcpSessionRepository, IMcpServerRepository, SaveRuntimeStateParams};
pub(crate) use roseui_realtime::EventBroadcaster;


/// One claude/codex session, presented as an `IAgentTask`.
pub struct SessionAgentTask {
    agent_type: AgentType,
    conversation_id: String,
    /// Owner Core user — every acp_session persistence write and catalog
    /// writeback is scoped to this user (multi-account boundary).
    user_id: String,
    workspace: String,
    backend: Arc<dyn SessionBackend>,
    runtime: Arc<SessionRuntime>,
    /// The `acp_session` persistence sink, retained so `set_config_option` can persist
    /// the chosen EFFORT level into `config_selections` — claude does NOT emit a
    /// `ConfigChanged` for effort (only for mode/model), so the event-pump's
    /// `persist_side_effects` never sees it. Without this write, effort would be lost
    /// across a respawn/resume (unlike mode/model, which persist via ConfigChanged).
    /// `None` (tests) = no persistence. Shared with the pump (same Arc).
    session_repo: Option<Arc<dyn IAcpSessionRepository>>,
    /// Cold-start catalog preload parsed from the persisted `agent_metadata`
    /// handshake (what a PRIOR session discovered and `spawn_catalog_writeback`
    /// stored). The backend's live `capabilities()` is empty until the initialize
    /// round-trip lands (~seconds on resume); the mode/model getters serve this in
    /// the meantime so the `/api/agents` picker is populated immediately instead of
    /// blank, then the live catalog overwrites it the moment it arrives. Empty on
    /// paths with no persisted catalog (fresh agent, tests). Mirrors the ACP path's
    /// `preload_advertised_catalogs` "fill-when-empty, live-overwrites" semantics.
    catalog_preload: CatalogPreload,
    /// Command-id counter for `CommandMeta` (dispatch correlation).
    command_seq: AtomicI64,
    /// Resolved prompt-dump target (see [`SessionPromptDump`]). `None` when
    /// `--dump-prompts` is off. Read only by `send_message`.
    prompt_dump: Option<SessionPromptDump>,
}

impl SessionAgentTask {
    /// Build a task around an already-opened `SessionBackend` and start the
    /// event-translation pump. `agent_type` is `AgentType::Acp` for claude/codex
    /// (they present as the ACP family to the rest of the app).
    ///
    /// `session_repo`, when present, is the persistence sink the event pump writes
    /// on the SAME signals the legacy ACP path persisted via
    /// `AcpSessionSyncService` (which this direct-CLI path bypasses): `BackendBound`
    /// → `acp_session.session_id` (the resume anchor `build_session_instance` reads
    /// back), `ConfigChanged` → `current_mode_id`/`current_model_id` (the mode/model
    /// precedence source). `None` (tests) = no persistence.
    pub fn new(
        agent_type: AgentType,
        conversation_id: String,
        user_id: String,
        workspace: String,
        backend: Arc<dyn SessionBackend>,
        session_repo: Option<Arc<dyn IAcpSessionRepository>>,
    ) -> Arc<Self> {
        Self::build(
            agent_type,
            conversation_id,
            user_id,
            workspace,
            backend,
            session_repo,
            CatalogPreload::default(),
            None,
        )
    }

    /// Same as [`new`], plus a cold-start catalog preload parsed from the
    /// persisted `agent_metadata` handshake. Production resume path uses this so
    /// the model/mode picker is populated immediately from the last discovered
    /// catalog while the backend's live `capabilities()` is still empty (the
    /// initialize round-trip lands a beat later and overwrites it).
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_preload(
        agent_type: AgentType,
        conversation_id: String,
        user_id: String,
        workspace: String,
        backend: Arc<dyn SessionBackend>,
        session_repo: Option<Arc<dyn IAcpSessionRepository>>,
        handshake: &roseui_api_types::AgentHandshake,
        prompt_dump: Option<SessionPromptDump>,
    ) -> Arc<Self> {
        Self::build(
            agent_type,
            conversation_id,
            user_id,
            workspace,
            backend,
            session_repo,
            CatalogPreload::from_handshake(handshake),
            prompt_dump,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn build(
        agent_type: AgentType,
        conversation_id: String,
        user_id: String,
        workspace: String,
        backend: Arc<dyn SessionBackend>,
        session_repo: Option<Arc<dyn IAcpSessionRepository>>,
        catalog_preload: CatalogPreload,
        prompt_dump: Option<SessionPromptDump>,
    ) -> Arc<Self> {
        let (tx, _rx) = broadcast::channel(EVENT_CHANNEL_CAPACITY);
        let runtime = Arc::new(SessionRuntime {
            tx,
            last_activity_ms: AtomicI64::new(now_ms()),
            status: std::sync::Mutex::new(None),
            session_id: std::sync::Mutex::new(None),
            mode_override: std::sync::Mutex::new(None),
            model_override: std::sync::Mutex::new(None),
            effort_override: std::sync::Mutex::new(None),
        });
        // Subscribe to the backend's event stream HERE (sync), then hand ONLY the
        // stream to the pump — never a backend Arc (see `spawn_event_pump` for why
        // capturing a backend Arc there would leak the child process).
        let events = backend.events();
        spawn_event_pump(
            events,
            runtime.clone(),
            conversation_id.clone(),
            user_id.clone(),
            session_repo.clone(),
        );
        Arc::new(Self {
            agent_type,
            conversation_id,
            user_id,
            workspace,
            backend,
            runtime,
            session_repo,
            catalog_preload,
            command_seq: AtomicI64::new(0),
            prompt_dump,
        })
    }

    fn next_command_id(&self) -> u64 {
        self.command_seq.fetch_add(1, Ordering::Relaxed) as u64
    }

    /// DEV (`--dump-prompts`): dump this turn's final input blocks as a
    /// `session-cli-final-input` JSON, symmetric with the ACP path's
    /// `acp-final-input`. Best-effort: a failure only warns and never affects
    /// the send. No-op when `--dump-prompts` is off (`prompt_dump == None`).
    fn dump_session_cli_final_input(&self, content: &[ContentBlock], client_msg_id: Option<&str>) {
        let Some(dump) = self.prompt_dump.as_ref() else {
            return;
        };
        let input = serde_json::json!({ "content": session_content_blocks_to_json(content) });
        let resolved_context = serde_json::json!({ "workspace": &self.workspace });
        match crate::dev_prompt_dump::dump_agent_final_input(
            &dump.dir,
            crate::dev_prompt_dump::AgentFinalInputDump {
                kind: "session-cli-final-input",
                backend: dump.backend,
                conversation_id: &self.conversation_id,
                session_id: self.runtime.session_id().as_deref(),
                msg_id: client_msg_id,
                turn_id: None,
                input,
                resolved_context,
            },
        ) {
            Ok(path) => tracing::debug!(
                conversation_id = %self.conversation_id,
                path = %path.display(),
                "DEV session-cli final input dump written"
            ),
            Err(error) => tracing::warn!(
                conversation_id = %self.conversation_id,
                error = %error,
                "DEV session-cli final input dump failed"
            ),
        }
    }

    // ── enum-level helpers forwarded from AgentInstance::Session ──────────
    // Backed by the backend's cheap sync `capabilities()` snapshot (reflects
    // late model/mode/config discovery) and `dispatch` for mutations.

    /// Pending confirmations, projected from the backend's live
    /// `pending_permission_requests()`. The REST `/confirmations` recovery path
    /// (frontend `usePendingConfirmationsRecovery`) calls this on mount/reconnect to
    /// rebuild permission cards that were raised while the page was away — WITHOUT
    /// this returning them, a mid-turn permission (or AskUserQuestion) raised before
    /// the client subscribed is lost and the turn hangs forever waiting for an answer
    /// that can never be given. The card id == call_id == request_id, matching the
    /// live `AcpPermission` frame so a duplicate live+recovered pair de-dups. Options
    /// mirror the live translation: AskUserQuestion → its question options, else the
    /// generic allow/deny.
    pub fn get_confirmations(&self) -> Vec<roseui_common::Confirmation> {
        self.backend
            .pending_permission_requests()
            .into_iter()
            .map(|p| {
                let is_ask = p.tool_name == "AskUserQuestion";
                let options = if is_ask {
                    ask_user_question_options(p.questions.as_ref())
                } else {
                    Vec::new()
                };
                let options = if options.is_empty() {
                    default_permission_options()
                } else {
                    options
                };
                roseui_common::Confirmation {
                    id: p.request_id.clone(),
                    call_id: p.request_id,
                    title: (!p.tool_name.is_empty()).then(|| p.tool_name.clone()),
                    action: None,
                    description: String::new(),
                    command_type: None,
                    options: options
                        .into_iter()
                        .map(|o| roseui_common::ConfirmationOption {
                            label: o.name,
                            value: serde_json::Value::String(o.option_id),
                            params: None,
                        })
                        .collect(),
                }
            })
            .collect()
    }

    /// Answer a pending permission. `data` is the option the user picked (the card
    /// echoes the option's `option_id` — a string, or `{option_id|value}` object).
    /// The picked id maps to the answer:
    ///   - `reject`        → Denied
    ///   - `allow_always`  → AllowAlways
    ///   - `allow`         → Approved
    ///   - anything else   → an AskUserQuestion answer LABEL → Approved + `selected`
    ///     (claude keys the AskUserQuestion answer by the chosen label — see
    ///     claude_conn `build_control_response`; single-select single-question path).
    ///
    /// `always_allow` (legacy flag) forces AllowAlways regardless.
    pub fn confirm(
        &self,
        _msg_id: &str,
        call_id: &str,
        data: serde_json::Value,
        always_allow: bool,
    ) -> Result<(), AgentError> {
        use roseui_session::PermissionDecision;
        let picked = confirm_option_id(&data);
        let (decision, selected) = if always_allow {
            (PermissionDecision::AllowAlways, None)
        } else {
            match picked.as_deref() {
                Some(s) if s == PERM_REJECT => (PermissionDecision::Denied, None),
                Some(s) if s == PERM_ALLOW_ALWAYS => (PermissionDecision::AllowAlways, None),
                Some(s) if s == PERM_ALLOW => (PermissionDecision::Approved, None),
                None => (PermissionDecision::Approved, None),
                // A question answer label (AskUserQuestion): approve and forward the
                // label so claude records it as the chosen answer.
                Some(label) => (PermissionDecision::Approved, Some(label.to_owned())),
            }
        };
        let backend = self.backend.clone();
        let request_id = call_id.to_string();
        // dispatch is async; confirm() is sync in IAgentTask's sibling API, so
        // fire-and-forget on the runtime (the answer rides the stdin FIFO).
        tokio::spawn(async move {
            let _ = backend
                .dispatch(Command::AnswerPermission {
                    request_id,
                    decision,
                    selected,
                    answers: Vec::new(),
                })
                .await;
        });
        Ok(())
    }

    /// Resolve the catalog to serve: the backend's LIVE `capabilities()` when it has
    /// discovered models/modes, else the cold-start `catalog_preload` (last session's
    /// persisted handshake). The LIST and the CURRENT of each axis fall back
    /// INDEPENDENTLY — this matters at cold-start: the backend seeds `caps.current_model`
    /// /`caps.current_mode` from the RESOLVED snapshot at spawn (claude_conn spawn:
    /// `caps.current_model = config.model`, where config.model came from the persisted
    /// `current_model_id` column), so it is already the user's last interactive switch
    /// even before the initialize round-trip lands and fills `available_models`. The
    /// preload's current, by contrast, is frozen at the PRIOR session's write-back
    /// (`spawn_catalog_writeback` runs once at open, not on a mid-turn switch), so a
    /// list-emptiness-gated fallback would show a stale model in the pre-init window
    /// whenever the user switched mid-turn last session (backend runs the right model,
    /// picker briefly showed the old one). So: the LIST prefers live (non-empty) then
    /// preload; the CURRENT prefers `caps` (already snapshot-seeded) then preload.
    /// `None`-current + no preload = None (getters then lean on the runtime override).
    fn effective_catalog(
        &self,
    ) -> (
        Vec<roseui_session::ModelInfo>,
        Option<String>,
        Vec<roseui_session::ModeInfo>,
        Option<String>,
    ) {
        let caps = self.backend.capabilities();
        let models = if caps.available_models.is_empty() {
            self.catalog_preload.available_models.clone()
        } else {
            caps.available_models
        };
        let current_model = caps
            .current_model
            .or_else(|| self.catalog_preload.current_model.clone());
        let modes = if caps.available_modes.is_empty() {
            self.catalog_preload.available_modes.clone()
        } else {
            caps.available_modes
        };
        let current_mode = caps.current_mode.or_else(|| self.catalog_preload.current_mode.clone());
        (models, current_model, modes, current_mode)
    }

    /// Current mode: the optimistic override (last `set_config_option("mode")`) wins
    /// over the capabilities snapshot, which lags an in-band switch.
    pub async fn mode(&self) -> Result<roseui_api_types::AgentModeResponse, AgentError> {
        let caps = self.backend.capabilities();
        // Preload fallback: on cold-start resume the live catalog is empty until the
        // initialize round-trip lands, so serve the last-discovered current_mode
        // until then (override still wins; live current_mode overwrites once present).
        let current_mode = caps.current_mode.or_else(|| self.catalog_preload.current_mode.clone());
        Ok(roseui_api_types::AgentModeResponse {
            mode: self.runtime.mode_override().or(current_mode).unwrap_or_default(),
            initialized: true,
        })
    }

    /// Current model + catalog. The optimistic override (last set_config_option
    /// "model") wins over the capabilities snapshot (claude gives set_model no
    /// confirmation wire, so caps.current_model never reflects the switch).
    pub async fn get_model(&self) -> Result<roseui_api_types::GetModelInfoResponse, AgentError> {
        // Live catalog wins; cold-start resume falls back to the persisted-handshake
        // preload so the picker is populated before the initialize round-trip lands.
        let (models, current_model, _modes, _mode) = self.effective_catalog();
        let override_model = self.runtime.model_override();
        if models.is_empty() && current_model.is_none() && override_model.is_none() {
            return Ok(roseui_api_types::GetModelInfoResponse { model_info: None });
        }
        let available_models: Vec<roseui_api_types::ModelInfoEntry> = models
            .iter()
            .map(|m| roseui_api_types::ModelInfoEntry {
                id: m.id.clone(),
                label: m.name.clone(),
            })
            .collect();
        let current_id = override_model.or(current_model);
        let current_label = current_id
            .as_ref()
            .and_then(|id| available_models.iter().find(|e| &e.id == id).map(|e| e.label.clone()));
        Ok(roseui_api_types::GetModelInfoResponse {
            model_info: Some(roseui_api_types::ModelInfoPayload {
                current_model_id: current_id.clone(),
                current_model_label: current_label.or(current_id),
                available_models,
            }),
        })
    }

    /// Config-options (mode + model selects). For each select the optimistic override
    /// (last set_config_option) wins over the capabilities snapshot's current_value —
    /// this is what makes set_config_option's observed re-read succeed (the snapshot
    /// lags an in-band claude switch).
    pub async fn get_config_options(&self) -> Result<roseui_api_types::GetConfigOptionsResponse, AgentError> {
        // Live catalog wins; cold-start resume falls back to the persisted-handshake
        // preload (per-axis) so the picker renders before the initialize round-trip lands.
        let (models, current_model, modes, current_mode) = self.effective_catalog();
        // The effort catalog depends on the EFFECTIVE current model (override wins over the
        // snapshot's current_model), resolved before the model option consumes it below.
        let effective_model = self.runtime.model_override().or_else(|| current_model.clone());
        let mut config_options = Vec::new();
        if !modes.is_empty() {
            config_options.push(roseui_api_types::AcpConfigOptionDto {
                id: "mode".into(),
                name: Some("Mode".into()),
                label: None,
                description: None,
                category: Some("mode".into()),
                option_type: "select".into(),
                current_value: self.runtime.mode_override().or(current_mode),
                options: modes
                    .iter()
                    .map(|m| roseui_api_types::AcpConfigSelectOptionDto {
                        value: m.id.clone(),
                        name: Some(m.name.clone()),
                        label: None,
                        description: m.description.clone(),
                    })
                    .collect(),
            });
        }
        if !models.is_empty() {
            config_options.push(roseui_api_types::AcpConfigOptionDto {
                id: "model".into(),
                name: Some("Model".into()),
                label: None,
                description: None,
                category: Some("model".into()),
                option_type: "select".into(),
                current_value: self.runtime.model_override().or(current_model),
                options: models
                    .iter()
                    .map(|m| roseui_api_types::AcpConfigSelectOptionDto {
                        value: m.id.clone(),
                        name: Some(m.name.clone()),
                        label: None,
                        description: m.description.clone(),
                    })
                    .collect(),
            });
        }
        // Reasoning-effort ("thought level") axis — the direct-CLI analogue of the ACP
        // path's `thought_level` config option (category-keyed so RoseUi's
        // `deriveSelectOption(..., 'thought_level', ['reasoning_effort'])` lights the
        // picker's effort group). Only claude advertises per-model `supportedEffortLevels`;
        // the option is emitted only when the resolved current model actually offers
        // efforts. `current_value` prefers the optimistic override (claude emits no echo
        // for effort) then the backend's synchronously-seeded `current_effort`.
        let caps = self.backend.capabilities();
        let efforts = resolve_current_model_efforts(&models, effective_model.as_deref());
        if !efforts.is_empty() {
            config_options.push(roseui_api_types::AcpConfigOptionDto {
                id: "reasoning_effort".into(),
                name: Some("Thinking".into()),
                label: None,
                description: None,
                category: Some("thought_level".into()),
                option_type: "select".into(),
                current_value: self.runtime.effort_override().or(caps.current_effort),
                options: efforts
                    .iter()
                    .map(|e| roseui_api_types::AcpConfigSelectOptionDto {
                        value: e.clone(),
                        name: Some(e.clone()),
                        label: None,
                        description: None,
                    })
                    .collect(),
            });
        }
        Ok(roseui_api_types::GetConfigOptionsResponse { config_options })
    }

    /// Apply a config option (mode/model/other) via dispatch.
    pub async fn set_config_option(
        &self,
        option_id: &str,
        value: &str,
    ) -> Result<roseui_api_types::SetConfigOptionResponse, AgentError> {
        // Validate a runtime mode/model switch against the advertised catalog BEFORE
        // dispatch — the ACP `clear_invalid_desired_*` semantic, but as REJECT+report
        // (not silent-drop) since this is an explicit user action at the single runtime
        // chokepoint. An EMPTY / not-yet-discovered catalog is permissive (matches ACP
        // `is_mode_valid`/`is_model_valid`: an absent catalog cannot invalidate — the
        // capabilities snapshot may simply not have the list yet). Only a NON-empty
        // catalog that omits `value` rejects. Other option ids (effort/thought_level)
        // are validated by the backend itself (claude effort catalog check).
        let caps = self.backend.capabilities();
        // A NON-empty catalog that omits `value` is the only rejection case (empty
        // catalog = permissive, per the comment above). `known` = catalog carries value.
        let invalid = |catalog_has_value: bool, catalog_empty: bool| !catalog_empty && !catalog_has_value;
        match option_id {
            "mode"
                if invalid(
                    caps.available_modes.iter().any(|m| m.id == value),
                    caps.available_modes.is_empty(),
                ) =>
            {
                return Err(AgentError::bad_request(format!(
                    "mode '{value}' is not one of the available modes"
                )));
            }
            "model"
                if invalid(
                    caps.available_models.iter().any(|m| m.id == value),
                    caps.available_models.is_empty(),
                ) =>
            {
                return Err(AgentError::bad_request(format!(
                    "model '{value}' is not one of the available models"
                )));
            }
            _ => {}
        }
        let cmd = match option_id {
            "mode" => Command::SetMode {
                mode: value.to_string(),
            },
            "model" => Command::SetModel {
                model: value.to_string(),
            },
            other => Command::SetConfigOption {
                option_id: other.to_string(),
                value: value.to_string(),
            },
        };
        self.backend
            .dispatch(cmd)
            .await
            .map_err(|e| AgentError::bad_request(e.to_string()))?;
        // Cache the requested value as an optimistic override for mode/model, then
        // re-read the config-options snapshot so the response satisfies the frontend's
        // `hasObservedValue` contract (confirmation == Observed AND the option's
        // current_value == requested). This is required because claude's own
        // `capabilities()` does NOT reflect an in-band switch synchronously (set_model
        // has no confirmation wire; set_permission_mode confirms only via a later
        // async system/status), so without the override the option would never read
        // back as observed and the frontend would reject the switch as `command_ack`.
        // Mirrors the clean-slate runtime's optimistic override + observed re-read.
        // effort/thought_level is now a surfaced picker option too (id `reasoning_effort`,
        // category `thought_level`), so it also caches an override + falls through to the
        // observed re-read — the frontend's `hasObservedValue` requires Observed AND the
        // option's current_value == requested, same as mode/model.
        match option_id {
            "mode" => self.runtime.set_mode_override(value.to_string()),
            "model" => self.runtime.set_model_override(value.to_string()),
            "effort" | "reasoning_effort" | "thought_level" => {
                // Optimistic highlight: claude emits no effort echo, so the streaming
                // catalog push reads the current level from this override.
                self.runtime.set_effort_override(value.to_string());
                // Persist the chosen effort into `config_selections` so it survives a
                // respawn/resume. Unlike mode/model (persisted by the pump on
                // ConfigChanged), claude emits no ConfigChanged for effort, so this is
                // the ONLY place the choice is durably recorded. Backend already accepted
                // + validated it (dispatch above); best-effort persist (a DB failure must
                // not fail the switch the CLI already applied).
                self.persist_effort(value).await;
            }
            _ => {
                return Ok(roseui_api_types::SetConfigOptionResponse {
                    confirmation: roseui_api_types::ConfigOptionConfirmation::CommandAck,
                    config_options: None,
                });
            }
        }
        let snapshot = self.get_config_options().await?;
        // Effort is emitted under the canonical id `reasoning_effort` (category
        // `thought_level`); a caller may address it via any of its aliases, so match by
        // category for the effort axis and by id otherwise.
        let is_effort_alias = matches!(option_id, "effort" | "reasoning_effort" | "thought_level");
        let observed = snapshot
            .config_options
            .iter()
            .find(|o| {
                if is_effort_alias {
                    o.category.as_deref() == Some("thought_level")
                } else {
                    o.id == option_id
                }
            })
            .and_then(|o| o.current_value.as_deref())
            == Some(value);
        Ok(roseui_api_types::SetConfigOptionResponse {
            confirmation: if observed {
                roseui_api_types::ConfigOptionConfirmation::Observed
            } else {
                roseui_api_types::ConfigOptionConfirmation::CommandAck
            },
            config_options: Some(snapshot.config_options),
        })
    }

    /// Persist the chosen effort level into `acp_session.config_selections` (under
    /// [`EFFORT_CONFIG_KEY`]) so it survives a respawn/resume. Reads the existing
    /// selections first and MERGES (rather than overwriting the whole map) so any other
    /// future config key is preserved. Best-effort: a repo miss/failure is logged, not
    /// propagated — the backend already applied the effort, and losing only the
    /// persistence (not the live switch) is the safe degradation. No-op without a repo.
    async fn persist_effort(&self, value: &str) {
        let Some(repo) = self.session_repo.as_ref() else {
            return;
        };
        // Merge into the existing selection map (preserve unrelated keys).
        let mut selections: std::collections::HashMap<String, String> = match repo
            .load_runtime_state_for_user(&self.user_id, &self.conversation_id)
            .await
        {
            Ok(Some(state)) => state
                .config_selections_json
                .as_deref()
                .and_then(|raw| serde_json::from_str(raw).ok())
                .unwrap_or_default(),
            Ok(None) => std::collections::HashMap::new(),
            Err(err) => {
                tracing::warn!(conversation_id = %self.conversation_id, error = %err, "persist_effort: load_runtime_state failed; skipping effort persist");
                return;
            }
        };
        selections.insert(EFFORT_CONFIG_KEY.to_owned(), value.to_owned());
        let json = match serde_json::to_string(&selections) {
            Ok(j) => j,
            Err(err) => {
                tracing::warn!(conversation_id = %self.conversation_id, error = %err, "persist_effort: encode config_selections failed");
                return;
            }
        };
        let params = SaveRuntimeStateParams {
            config_selections_json: Some(Some(&json)),
            ..Default::default()
        };
        if let Err(err) = repo
            .save_runtime_state_for_user(&self.user_id, &self.conversation_id, &params)
            .await
        {
            tracing::warn!(conversation_id = %self.conversation_id, error = %err, "persist_effort: save_runtime_state failed");
        }
    }

    /// Session usage snapshot (`used` / `size` tokens). For backends that
    /// advertise `query_session_info` (claude/codex) we issue a one-shot
    /// `QuerySessionInfo{ContextUsage}` and await the `SessionInfo` reply on the
    /// event stream; otherwise fall back to `None` (the live `UsageDelta` stream
    /// is the alternative source for non-capable backends).
    pub async fn get_usage(&self) -> Result<Option<serde_json::Value>, AgentError> {
        use roseui_session::SessionInfoKind;
        use tokio::time::{Duration, timeout};

        if !self.backend.capabilities().supported_commands.query_session_info {
            return Ok(None);
        }

        // Subscribe a fresh view of the broadcast event stream, then dispatch the
        // query. The reply arrives as `SessionEvent::SessionInfo{context_usage}`.
        let mut events = self.backend.events();
        self.backend
            .dispatch(Command::QuerySessionInfo {
                kind: SessionInfoKind::ContextUsage,
            })
            .await
            .map_err(|e| AgentError::internal(format!("query_session_info dispatch failed: {e}")))?;

        // Bound the wait so a silent backend never hangs the HTTP handler.
        let wait = timeout(Duration::from_secs(5), async {
            use futures_util::StreamExt;
            while let Some(env) = events.next().await {
                if let roseui_session::SessionEvent::SessionInfo { context_usage: Some(usage), .. } = env.event {
                    return Some(usage);
                }
            }
            None
        })
        .await
        .unwrap_or(None);

        let Some(usage) = wait else {
            return Ok(None);
        };

        let mut value = serde_json::json!({
            "used": usage.used,
            "size": usage.max,
        });
        roseui_common::normalize_keys_to_snake_case(&mut value);
        Ok(Some(value))
    }

    /// Slash commands from the live capabilities snapshot.
    pub async fn get_slash_commands(&self) -> Result<Vec<roseui_api_types::SlashCommandItem>, AgentError> {
        let caps = self.backend.capabilities();
        Ok(caps
            .slash_commands
            .iter()
            .map(|c| roseui_api_types::SlashCommandItem {
                command: c.name.clone(),
                description: c.description.clone().unwrap_or_default(),
                completion_behavior: None,
                empty_turn_tip_code: None,
                empty_turn_tip_params: None,
            })
            .collect())
    }
}

#[async_trait::async_trait]
impl IAgentTask for SessionAgentTask {
    fn agent_type(&self) -> AgentType {
        self.agent_type
    }

    fn conversation_id(&self) -> &str {
        &self.conversation_id
    }

    fn workspace(&self) -> &str {
        &self.workspace
    }

    fn status(&self) -> Option<ConversationStatus> {
        self.runtime.status.lock().ok().and_then(|g| *g)
    }

    fn last_activity_at(&self) -> TimestampMs {
        self.runtime.last_activity_ms.load(Ordering::Relaxed)
    }

    fn subscribe(&self) -> broadcast::Receiver<AgentStreamEvent> {
        self.runtime.tx.subscribe()
    }

    async fn send_message(&self, data: SendMessageData) -> Result<(), AgentSendError> {
        self.runtime.touch();
        let mut content: Vec<ContentBlock> = Vec::new();
        if !data.content.is_empty() {
            content.push(ContentBlock::Text(data.content));
        }
        for path in data.files {
            // File paths ride as resource links; the claude/codex adapters resolve
            // them (Read tool / base64) at dispatch time.
            content.push(ContentBlock::ResourceLink {
                uri: path,
                mime_type: None,
            });
        }
        // DEV (`--dump-prompts`): borrow the final blocks BEFORE they move into
        // Command::Send. No-op / best-effort — never affects the dispatch.
        self.dump_session_cli_final_input(&content, Some(data.msg_id.as_str()));

        let cmd = Command::Send {
            content,
            metadata: CommandMeta {
                command_id: self.next_command_id(),
                cwd: None,
                extra_args: Vec::new(),
                client_msg_id: Some(data.msg_id),
            },
        };
        // Emit the turn-start lifecycle frame BEFORE dispatch, exactly like the ACP
        // path (agent_session_flow.rs emits Start{session_id} right before prompt()).
        // The backend's own turn-start signal (claude/codex PromptAccepted) arrives
        // AFTER the first text delta, so it cannot drive an at-the-front Start — the
        // send call is the correct, ordering-stable anchor. session_id is None on the
        // very first turn (backend not yet bound) and filled on every subsequent turn.
        let _ = self.runtime.tx.send(AgentStreamEvent::Start(StartEventData {
            session_id: self.runtime.session_id(),
        }));
        self.runtime.set_status(ConversationStatus::Running);
        match self.backend.dispatch(cmd).await {
            Ok(_receipt) => Ok(()),
            // Dead resume anchor surfaced at DISPATCH time (codex rejected the
            // thread/resume → bound-thread poison, ELECTRON-3Q0): the stored anchor
            // can never resume again — clear it NOW so the auto-replay (and any
            // later send) rebuilds Fresh. The stream-side self-heal
            // (`is_dead_resume_anchor`) cannot cover this path: a dispatch error
            // never becomes a `TurnResult` on the event pump.
            Err(BackendError::SessionNotFound(detail)) => {
                if let Some(repo) = self.session_repo.as_ref() {
                    match repo
                        .clear_session_id_for_user(&self.user_id, &self.conversation_id)
                        .await
                    {
                        Ok(_) => tracing::info!(
                            conversation_id = %self.conversation_id,
                            "send: cleared dead resume anchor (backend session not found) — next attempt opens Fresh"
                        ),
                        Err(err) => tracing::warn!(
                            conversation_id = %self.conversation_id,
                            error = %err,
                            "send: clear_session_id failed"
                        ),
                    }
                }
                // The "Session not found" prefix classifies as
                // `UserAgentSessionNotFound` (retryable) so `TurnRecoveryPolicy`
                // auto-replays once — with the anchor cleared above, the replay
                // opens Fresh and recovers transparently.
                Err(AgentSendError::from_agent_error(AgentError::not_found(format!(
                    "Session not found: {detail}"
                ))))
            }
            Err(e) => Err(AgentSendError::from_agent_error(AgentError::bad_gateway(e.to_string()))),
        }
    }

    async fn cancel(&self) -> Result<(), AgentError> {
        self.runtime.touch();
        self.backend
            .dispatch(Command::Cancel {
                target: roseui_session::CancelTarget::Turn,
            })
            .await
            .map(|_| ())
            .map_err(|e| AgentError::internal(e.to_string()))
    }

    fn kill(&self, reason: Option<AgentKillReason>) -> Result<(), AgentError> {
        // For the `UserCancelTimeout` force-kill path, Drop-driven teardown is NOT
        // enough: during an in-flight turn the orchestrator legitimately holds an
        // `Arc<SessionAgentTask>` clone, so `tasks.remove()` does not drop the last
        // reference, `Drop` never fires, and the CLI + its workflow keep running
        // until the workflow ends naturally (~minutes) — the user is gated the whole
        // time (ELECTRON-3RW). So we (1) emit a clean `Finish` FIRST to converge the
        // turn (gate recovers in seconds, no crash card), then (2) delegate real
        // process teardown to the backend, which kills the process tree WITHOUT
        // waiting for the last Arc to drop.
        if matches!(reason, Some(AgentKillReason::UserCancelTimeout)) {
            tracing::info!(
                conversation_id = %self.conversation_id,
                "session kill(UserCancelTimeout): emitted clean Finish + delegating backend terminate (was Drop-only no-op)"
            );
            // 1) clean converge FIRST: relay breaks → orchestrator releases the turn
            //    claim → `cancelling` cleared → gate recovers (no red crash card).
            self.runtime.emit_finish_once();
            // 2) then real process teardown, fire-and-forget on this sync entry
            //    (the awaitable variant is `kill_and_wait`).
            let backend = self.backend.clone();
            tokio::spawn(async move {
                backend.terminate().await;
            });
            return Ok(());
        }
        // Non-`UserCancelTimeout` (idle cleanup): unchanged Drop-driven teardown. The
        // `SessionBackend` trait exposes no close/shutdown command; the sole reaper is
        // `Drop for ClaudeSessionBackend` / `CodexSessionBackend`, which aborts the
        // reader and `kill_on_drop`s the child once the last `Arc` is released. An idle
        // kill has no in-flight orchestrator Arc, so that removal drops the last Arc and
        // fires the backend's `Drop`. Nothing synchronous to do here.
        Ok(())
    }
}

impl SessionAgentTask {
    /// Awaitable force-kill (aligns with `AcpAgentManager::kill_and_wait`): the
    /// awaitable teardown entry the `UserCancel` watchdog uses. For a
    /// `UserCancelTimeout` kill it emits the clean `Finish` synchronously FIRST
    /// (so the gate recovers before the returned future is even polled), then
    /// returns a future that awaits the backend's real process teardown. Any
    /// other reason keeps the pre-existing Drop-driven semantics (nothing to
    /// await). The strict order — clean converge, THEN terminate — is what keeps
    /// the kill from surfacing as a crash (see `SuspendController::terminate`).
    pub fn kill_and_wait(
        &self,
        reason: Option<AgentKillReason>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> {
        if matches!(reason, Some(AgentKillReason::UserCancelTimeout)) {
            tracing::info!(
                conversation_id = %self.conversation_id,
                "session kill_and_wait(UserCancelTimeout): emitted clean Finish + awaiting backend terminate"
            );
            self.runtime.emit_finish_once(); // clean converge FIRST (sync)
            let backend = self.backend.clone();
            Box::pin(async move { backend.terminate().await }) // then terminate
        } else {
            Box::pin(std::future::ready(())) // unchanged (Drop-driven)
        }
    }
}

#[cfg(test)]
mod build_mapping_tests {
    //! Ported from clean-slate `session_runtime::tests` (the `spec_and_config` +
    //! `catalog_partial_from_caps` + `codex_sandbox`/`approval` + `session_server_to_spec`
    //! suite), adapted to the port's decomposed `spec_mode_model` inputs (AcpBuildExtra +
    //! PersistedSessionState) instead of a `ConversationRow`. Same assertions.
    use super::*;
    use crate::shared_kernel::{ModeId, ModelId};
    use roseui_session::SessionSpec;

    fn snapshot(mode: Option<&str>, model: Option<&str>) -> PersistedSessionState {
        PersistedSessionState {
            current_mode_id: mode.map(ModeId::new),
            current_model_id: model.map(ModelId::new),
            ..Default::default()
        }
    }

    #[test]
    fn assemble_spawn_env_orders_agent_overrides_before_runtime_context() {
        let agent_env = vec![
            roseui_api_types::AgentEnvEntry {
                name: "AWS_PROFILE".into(),
                value: "pionex".into(),
                description: None,
            },
            roseui_api_types::AgentEnvEntry {
                name: "HTTPS_PROXY".into(),
                value: "http://proxy:8080".into(),
                description: None,
            },
        ];
        let runtime_env = vec![
            ("ROSEUI_USER_ID".to_owned(), "user-1".to_owned()),
            ("ROSEUI_RUNTIME_TOKEN".to_owned(), "tok".to_owned()),
        ];
        let env = assemble_spawn_env(&agent_env, &runtime_env);
        let names: Vec<&str> = env.iter().map(|e| e.name.as_str()).collect();
        // Agent overrides first, runtime context after (later wins in
        // ManagedProcess::spawn, so the runtime context can never be shadowed).
        assert_eq!(
            names,
            ["AWS_PROFILE", "HTTPS_PROXY", "ROSEUI_USER_ID", "ROSEUI_RUNTIME_TOKEN"]
        );
        assert_eq!(env[0].value, "pionex");
        assert_eq!(env[3].value, "tok");
        assert!(
            assemble_spawn_env(&[], &[]).is_empty(),
            "empty in = empty out (inherit-only spawn)"
        );
    }

    #[test]
    fn session_cli_config_dump_value_captures_resolved_surface() {
        use roseui_session::{McpServerSpec, McpTransport, SessionConfig, SessionInit};
        let cfg = SessionConfig {
            model: Some("opus".into()),
            mode: Some("plan".into()),
            sandbox_mode: Some("danger-full-access".into()),
            approval_policy: Some("never".into()),
            cli_program: Some(std::path::PathBuf::from("/opt/claude")),
            spawn_env: vec![roseui_common::EnvVar {
                name: "ANTHROPIC_AUTH_TOKEN".into(),
                value: "raw-token".into(),
            }],
            init: SessionInit {
                preset_context: Some("SYSTEM PROMPT BODY".into()),
                skills: vec!["writer".into()],
                mcp_servers: vec![McpServerSpec {
                    name: "team".into(),
                    transport: McpTransport::Stdio {
                        command: "node".into(),
                        args: vec!["srv.js".into()],
                        env: vec![("K".into(), "V".into())],
                    },
                }],
                resume: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let v = build_session_cli_config_dump_value("claude", &cfg);
        assert_eq!(v["input"]["backend"], "claude");
        assert_eq!(v["input"]["model"], "opus");
        assert_eq!(v["input"]["mode"], "plan");
        assert_eq!(v["input"]["sandbox_mode"], "danger-full-access");
        assert_eq!(v["input"]["approval_policy"], "never");
        assert_eq!(v["input"]["cli_program"], "/opt/claude");
        assert_eq!(v["input"]["resume"], true);
        // RAW, no redaction (dev-only), matching the acp dump behavior.
        assert_eq!(v["resolved_context"]["preset_context"], "SYSTEM PROMPT BODY");
        assert_eq!(v["resolved_context"]["skills"][0], "writer");
        assert_eq!(v["resolved_context"]["spawn_env"][0]["name"], "ANTHROPIC_AUTH_TOKEN");
        assert_eq!(v["resolved_context"]["spawn_env"][0]["value"], "raw-token");
        assert_eq!(v["resolved_context"]["mcp_servers"][0]["name"], "team");
        assert_eq!(v["resolved_context"]["mcp_servers"][0]["transport"]["type"], "stdio");
    }

    // Minimal catalog row for spec_mode_model's mode-normalize step. `backend` +
    // `yolo_id` drive the alias mapping; everything else is inert here.
    fn test_metadata(backend: Option<&str>, yolo_id: Option<&str>) -> roseui_api_types::AgentMetadata {
        use roseui_api_types::{AgentHandshake, AgentMetadata, AgentSource, AgentSourceInfo, BehaviorPolicy};
        use roseui_common::AgentType;
        AgentMetadata {
            id: "test".into(),
            icon: None,
            name: "Test".into(),
            name_i18n: None,
            description: None,
            description_i18n: None,
            backend: backend.map(ToOwned::to_owned),
            agent_type: AgentType::Acp,
            agent_source: AgentSource::Builtin,
            agent_source_info: AgentSourceInfo::default(),
            enabled: true,
            available: true,
            command: None,
            resolved_command: None,
            args: vec![],
            env: vec![],
            native_skills_dirs: None,
            behavior_policy: BehaviorPolicy::default(),
            yolo_id: yolo_id.map(ToOwned::to_owned),
            sort_order: 0,
            team_capable: false,
            last_check_status: None,
            last_check_kind: None,
            last_check_error_code: None,
            last_check_error_message: None,
            last_check_error_details: None,
            last_check_guidance: None,
            last_check_latency_ms: None,
            last_check_at: None,
            last_success_at: None,
            last_failure_at: None,
            handshake: AgentHandshake::default(),
            has_command_override: false,
            env_override_key_count: 0,
        }
    }

    #[test]
    fn spec_fresh_when_no_anchor() {
        let cfg = AcpBuildExtra::default();
        let (spec, mode, model) = spec_mode_model("conv_1", None, &cfg, None, &test_metadata(Some("claude"), None));
        assert!(matches!(spec, SessionSpec::Fresh { session_id } if session_id == "conv_1"));
        assert_eq!(mode, None);
        assert_eq!(model, None);
    }

    #[test]
    fn spec_resume_when_anchor_present() {
        let cfg = AcpBuildExtra {
            session_mode: Some("plan".into()),
            current_model_id: Some("claude-x".into()),
            ..Default::default()
        };
        let (spec, mode, model) = spec_mode_model(
            "conv_1",
            Some("bsid-xyz".into()),
            &cfg,
            None,
            &test_metadata(Some("claude"), None),
        );
        assert!(matches!(
            spec,
            SessionSpec::Resume { backend_session_id: Some(b), .. } if b == "bsid-xyz"
        ));
        assert_eq!(mode.as_deref(), Some("plan"));
        assert_eq!(model.as_deref(), Some("claude-x"));
    }

    // The interactive-switch-persisted snapshot selection MUST win over the
    // create-time config values on resume — else the user's choice is dropped on
    // respawn. (Clean-slate: spec_and_config_runtime_model_overrides_stale_model_column.)
    #[test]
    fn snapshot_mode_model_override_create_time_config() {
        let cfg = AcpBuildExtra {
            session_mode: Some("default".into()),
            current_model_id: Some("claude-sonnet-4-6".into()),
            ..Default::default()
        };
        let snap = snapshot(Some("plan"), Some("claude-opus-4-8"));
        let (_spec, mode, model) = spec_mode_model(
            "conv_1",
            Some("bsid".into()),
            &cfg,
            Some(&snap),
            &test_metadata(Some("claude"), None),
        );
        assert_eq!(mode.as_deref(), Some("plan"), "snapshot mode wins");
        assert_eq!(model.as_deref(), Some("claude-opus-4-8"), "snapshot model wins");
    }

    // Empty strings are filtered → None so the backend safe-defaults (never an empty
    // model/mode token on the wire).
    #[test]
    fn empty_selections_filter_to_none() {
        let cfg = AcpBuildExtra {
            session_mode: Some(String::new()),
            current_model_id: Some(String::new()),
            ..Default::default()
        };
        let (_spec, mode, model) = spec_mode_model("conv_1", None, &cfg, None, &test_metadata(Some("claude"), None));
        assert_eq!(mode, None);
        assert_eq!(model, None);
    }

    // HIGH-1 regression guard (equivalence audit): a persisted generic mode alias must
    // be normalized to the backend-native id via the catalog row — the SAME transform
    // the ACP path applies. Without it the raw alias reaches the backend on resume
    // (claude rejects an unknown permission-mode; codex mis-policies).
    #[test]
    fn mode_alias_is_normalized_via_catalog() {
        // codex: yoloNoSandbox → the row's yolo_id (full-access); default → auto.
        let codex = test_metadata(Some("codex"), Some("full-access"));
        let yolo_cfg = AcpBuildExtra {
            session_mode: Some("yoloNoSandbox".into()),
            ..Default::default()
        };
        let (_s, mode, _m) = spec_mode_model("c", None, &yolo_cfg, None, &codex);
        assert_eq!(
            mode.as_deref(),
            Some("full-access"),
            "yoloNoSandbox → codex native yolo_id"
        );

        let def_cfg = AcpBuildExtra {
            session_mode: Some("default".into()),
            ..Default::default()
        };
        let (_s, mode, _m) = spec_mode_model("c", None, &def_cfg, None, &codex);
        assert_eq!(mode.as_deref(), Some("auto"), "codex default → auto");

        // A native / non-alias mode passes through unchanged.
        let plan_cfg = AcpBuildExtra {
            session_mode: Some("plan".into()),
            ..Default::default()
        };
        let (_s, mode, _m) = spec_mode_model("c", None, &plan_cfg, None, &test_metadata(Some("claude"), None));
        assert_eq!(mode.as_deref(), Some("plan"), "non-alias mode unchanged");
    }

    /// G5: a discovered catalog projects mode/model as ACP `configOptions[]` + slash
    /// commands as `available_commands`; an empty catalog projects `None` (never
    /// clobbers the stored catalog). Ported verbatim from clean-slate.
    #[test]
    fn catalog_partial_projects_discovered_modes_models_commands() {
        use roseui_session::{ModeInfo, ModelInfo, SlashCommandInfo};
        let caps = roseui_session::Capabilities {
            available_modes: vec![ModeInfo {
                id: "plan".into(),
                name: "Plan".into(),
                description: Some("Planning".into()),
            }],
            current_mode: Some("plan".into()),
            available_models: vec![ModelInfo {
                id: "opus".into(),
                name: "Opus".into(),
                description: None,
                reasoning_efforts: Vec::new(),
            }],
            current_model: Some("opus".into()),
            slash_commands: vec![SlashCommandInfo {
                name: "review".into(),
                description: Some("Review a PR".into()),
            }],
            ..Default::default()
        };

        let partial = catalog_partial_from_caps(&caps).expect("a discovered catalog projects a partial");
        let cfg = partial.config_options.expect("config_options present");
        let opts = cfg.as_array().unwrap();
        assert_eq!(opts[0]["id"], "mode");
        assert_eq!(opts[0]["currentValue"], "plan");
        assert_eq!(opts[0]["options"][0]["value"], "plan");
        assert_eq!(opts[1]["id"], "model");
        assert_eq!(opts[1]["currentValue"], "opus");
        let cmds = partial.available_commands.expect("commands present");
        assert_eq!(cmds.as_array().unwrap()[0]["name"], "review");

        let empty = roseui_session::Capabilities::default();
        assert!(
            catalog_partial_from_caps(&empty).is_none(),
            "empty catalog projects nothing"
        );
    }

    /// Full-access / yolo escalates the codex sandbox to `danger-full-access`;
    /// read-only RESTRICTS it to `read-only` (so the FIRST turn is already locked
    /// down — the SetMode permission profile only lands on the NEXT turn); the
    /// workspace/auto middle tier keeps None ⇒ workspace-write.
    #[test]
    fn codex_sandbox_maps_full_access_and_read_only_modes() {
        // #608 canonical full-access id (migration 021 + normalize_requested_mode).
        assert_eq!(
            codex_sandbox_for_mode(Some("agent-full-access")),
            Some("danger-full-access")
        );
        // Plan B legacy bare token (pre-021 persisted data).
        assert_eq!(codex_sandbox_for_mode(Some("full-access")), Some("danger-full-access"));
        // The colon profile id (e.g. a readback that skipped bare-mapping) stays recognized.
        assert_eq!(
            codex_sandbox_for_mode(Some(":danger-full-access")),
            Some("danger-full-access")
        );
        assert_eq!(
            codex_sandbox_for_mode(Some("yoloNoSandbox")),
            Some("danger-full-access")
        );
        assert_eq!(
            codex_sandbox_for_mode(Some("  :danger-full-access  ")),
            Some("danger-full-access")
        );
        // read-only RESTRICTS the sandbox at OPEN time (regression fix: seeding this
        // at thread/start is what makes the first-turn write actually blocked; the
        // SetMode permission profile alone applies too late). Both the bare token and
        // the colon id (and surrounding whitespace) are recognized.
        assert_eq!(codex_sandbox_for_mode(Some("read-only")), Some("read-only"));
        assert_eq!(codex_sandbox_for_mode(Some(":read-only")), Some("read-only"));
        assert_eq!(codex_sandbox_for_mode(Some("  read-only  ")), Some("read-only"));
        // The workspace/auto middle tier keeps the safe workspace-write default.
        assert_eq!(codex_sandbox_for_mode(Some(":workspace")), None);
        assert_eq!(codex_sandbox_for_mode(Some("plan")), None);
        assert_eq!(codex_sandbox_for_mode(Some("default")), None);
        assert_eq!(codex_sandbox_for_mode(None), None);
    }

    /// Sibling of the sandbox map: a full-access / yolo mode drops approvals
    /// (→ "never"); everything else stays at on-request (None). Ported verbatim.
    #[test]
    fn codex_approval_maps_only_full_access_modes() {
        assert_eq!(codex_approval_for_mode(Some(":danger-full-access")), Some("never"));
        assert_eq!(codex_approval_for_mode(Some("agent-full-access")), Some("never"));
        assert_eq!(codex_approval_for_mode(Some("full-access")), Some("never"));
        assert_eq!(codex_approval_for_mode(Some("yoloNoSandbox")), Some("never"));
        assert_eq!(codex_approval_for_mode(Some("  :danger-full-access  ")), Some("never"));
        assert_eq!(codex_approval_for_mode(Some(":read-only")), None);
        assert_eq!(codex_approval_for_mode(Some(":workspace")), None);
        assert_eq!(codex_approval_for_mode(Some("plan")), None);
        assert_eq!(codex_approval_for_mode(Some("default")), None);
        assert_eq!(codex_approval_for_mode(None), None);
    }

    #[test]
    fn session_server_to_spec_collapses_4_transports_to_3_and_sorts_kv() {
        use roseui_api_types::{SessionMcpServer, SessionMcpTransport};
        use roseui_session::McpTransport;
        use std::collections::HashMap;

        let stdio = session_server_to_spec(&SessionMcpServer {
            id: "1".into(),
            name: "fs".into(),
            transport: SessionMcpTransport::Stdio {
                command: "/node".into(),
                args: vec!["s.js".into()],
                env: HashMap::from([("B".into(), "2".into()), ("A".into(), "1".into())]),
            },
        });
        assert_eq!(stdio.name, "fs");
        match stdio.transport {
            McpTransport::Stdio { command, env, .. } => {
                assert_eq!(command, "/node");
                assert_eq!(
                    env,
                    vec![("A".into(), "1".into()), ("B".into(), "2".into())],
                    "env sorted by key"
                );
            }
            other => panic!("expected Stdio, got {other:?}"),
        }

        for t in [
            SessionMcpTransport::StreamableHttp {
                url: "https://x".into(),
                headers: HashMap::new(),
            },
            SessionMcpTransport::Http {
                url: "https://x".into(),
                headers: HashMap::new(),
            },
        ] {
            let spec = session_server_to_spec(&SessionMcpServer {
                id: "2".into(),
                name: "h".into(),
                transport: t,
            });
            assert!(
                matches!(spec.transport, McpTransport::Http { .. }),
                "Http+StreamableHttp → Http"
            );
        }
    }
}

#[cfg(test)]
mod translate_tests {
    use super::*;
    use crate::protocol::events::tool_call::{ToolCallEventData, ToolCallStatus};
    use roseui_session::PermissionKind;

    fn tool_call(call_id: &str, name: &str, status: ToolCallStatus) -> AgentStreamEvent {
        AgentStreamEvent::ToolCall(ToolCallEventData {
            call_id: call_id.into(),
            name: name.into(),
            args: serde_json::Value::Null,
            status,
            input: None,
            output: None,
            description: None,
        })
    }

    // The bug: a tool's terminal ToolResult frame (and any codex ToolOutputDelta)
    // carries no name, so — persisted by upsert on call_id — it clobbered the tool
    // name to "" and the frontend rendered a nameless tool line. `stamp_tool_name`
    // remembers the name from the Running frame and refills the empty follow-ups.
    #[test]
    fn tool_name_survives_the_empty_name_result_frame() {
        let mut names = std::collections::HashMap::new();

        // Running frame carries the name; the map learns it.
        let mut running = tool_call("call-1", "Read", ToolCallStatus::Running);
        stamp_tool_name(&mut names, &mut running);
        assert_eq!(names.get("call-1").map(String::as_str), Some("Read"));

        // Codex live-output frame arrives with an empty name → refilled.
        let mut delta = tool_call("call-1", "", ToolCallStatus::Running);
        stamp_tool_name(&mut names, &mut delta);
        let AgentStreamEvent::ToolCall(d) = &delta else {
            unreachable!()
        };
        assert_eq!(d.name, "Read", "live-output frame must keep the name");

        // Terminal result frame arrives with an empty name → refilled, NOT clobbered.
        let mut result = tool_call("call-1", "", ToolCallStatus::Completed);
        stamp_tool_name(&mut names, &mut result);
        let AgentStreamEvent::ToolCall(r) = &result else {
            unreachable!()
        };
        assert_eq!(r.name, "Read", "result frame must keep the name, not go blank");
        assert_eq!(r.status, ToolCallStatus::Completed);
    }

    // A result frame for a call_id we never saw a name for stays empty (no panic,
    // no cross-call bleed) — and a different call_id is never cross-filled.
    #[test]
    fn stamp_tool_name_does_not_bleed_across_call_ids() {
        let mut names = std::collections::HashMap::new();
        let mut a = tool_call("call-a", "Bash", ToolCallStatus::Running);
        stamp_tool_name(&mut names, &mut a);

        let mut orphan = tool_call("call-b", "", ToolCallStatus::Completed);
        stamp_tool_name(&mut names, &mut orphan);
        let AgentStreamEvent::ToolCall(o) = &orphan else {
            unreachable!()
        };
        assert_eq!(o.name, "", "unknown call_id must not inherit another tool's name");
    }

    #[test]
    fn permission_surfaces_as_acp_permission_keyed_on_request_id() {
        let events = translate_event(
            SessionEvent::Permission {
                request_id: "req-42".into(),
                kind: PermissionKind::Tool,
                metadata: None,
                tool_name: Some("Bash".into()),
                input: Some(serde_json::json!({"command": "ls"})),
            },
            "conv-1",
            false,
        );
        assert_eq!(events.len(), 1, "permission must project to exactly one card");
        let crate::protocol::events::AgentStreamEvent::AcpPermission(
            crate::protocol::events::AcpPermissionEventData::Request(req),
        ) = &events[0]
        else {
            panic!("expected AcpPermission Request, got {:?}", events[0]);
        };
        // The confirm() path answers AnswerPermission keyed on this id — it MUST
        // equal the originating request_id or the approval never resolves.
        assert_eq!(req.tool_call.tool_call_id, "req-42");
        assert_eq!(req.tool_call.title.as_deref(), Some("Bash"));
        assert!(req.tool_call.raw_input.is_some(), "tool input rides as raw_input");
        // A NON-AskUserQuestion tool approval offers the generic Allow/AllowAlways/Reject.
        assert_eq!(req.options.len(), 3);
        let ids: Vec<&str> = req.options.iter().map(|o| o.option_id.as_str()).collect();
        assert_eq!(ids, vec!["allow", "allow_always", "reject"]);
    }

    // An AskUserQuestion permission must surface the QUESTION's own option labels as
    // the card choices (so the user answers the question), NOT a generic allow/deny.
    // This is the fix for "AskUserQuestion rendered as an allow box": the frontend
    // card renders whatever options[] the backend sends.
    #[test]
    fn ask_user_question_surfaces_question_options_not_allow_deny() {
        let events = translate_event(
            SessionEvent::Permission {
                request_id: "req-ask".into(),
                kind: PermissionKind::Tool,
                metadata: None,
                tool_name: Some("AskUserQuestion".into()),
                input: Some(serde_json::json!({
                    "questions": [{
                        "question": "Which database?",
                        "header": "DB",
                        "multiSelect": false,
                        "options": [
                            {"label": "Postgres", "description": "relational"},
                            {"label": "SQLite", "description": "embedded"}
                        ]
                    }]
                })),
            },
            "conv-1",
            false,
        );
        let crate::protocol::events::AgentStreamEvent::AcpPermission(
            crate::protocol::events::AcpPermissionEventData::Request(req),
        ) = &events[0]
        else {
            panic!("expected AcpPermission Request, got {:?}", events[0]);
        };
        let ids: Vec<&str> = req.options.iter().map(|o| o.option_id.as_str()).collect();
        let names: Vec<&str> = req.options.iter().map(|o| o.name.as_str()).collect();
        // The card offers the question's labels — option_id == label so confirm() can
        // forward the pick as the AnswerPermission answer label.
        assert_eq!(
            ids,
            vec!["Postgres", "SQLite"],
            "must render question options, not allow/deny"
        );
        assert_eq!(names, vec!["Postgres", "SQLite"]);
        assert!(!ids.contains(&"allow"), "must NOT be a generic allow/deny card");
    }

    // A malformed / optionless AskUserQuestion falls back to the generic card rather
    // than rendering an unanswerable empty option list.
    #[test]
    fn ask_user_question_without_options_falls_back_to_generic() {
        let events = translate_event(
            SessionEvent::Permission {
                request_id: "req-ask2".into(),
                kind: PermissionKind::Tool,
                metadata: None,
                tool_name: Some("AskUserQuestion".into()),
                input: Some(serde_json::json!({"questions": []})),
            },
            "conv-1",
            false,
        );
        let crate::protocol::events::AgentStreamEvent::AcpPermission(
            crate::protocol::events::AcpPermissionEventData::Request(req),
        ) = &events[0]
        else {
            panic!("expected AcpPermission Request");
        };
        let ids: Vec<&str> = req.options.iter().map(|o| o.option_id.as_str()).collect();
        assert_eq!(ids, vec!["allow", "allow_always", "reject"], "fallback to generic");
    }

    #[test]
    fn usage_delta_surfaces_as_context_usage() {
        let events = translate_event(
            SessionEvent::UsageDelta {
                input_tokens: 10,
                output_tokens: 20,
                total_tokens: 30,
                cost_usd: Some(0.5),
            },
            "conv-1",
            false,
        );
        assert_eq!(events.len(), 1);
        let crate::protocol::events::AgentStreamEvent::AcpContextUsage(v) = &events[0] else {
            panic!("expected AcpContextUsage, got {:?}", events[0]);
        };
        // Frontend ContextUsageIndicator reads `used` (not `total_tokens`) — the
        // shape the ACP path forwards. `cost` rides as {amount,currency}.
        assert_eq!(v.get("used").and_then(|x| x.as_u64()), Some(30), "used = total_tokens");
        assert_eq!(
            v.get("cost").and_then(|c| c.get("amount")).and_then(|x| x.as_f64()),
            Some(0.5)
        );
        assert_eq!(
            v.get("cost").and_then(|c| c.get("currency")).and_then(|x| x.as_str()),
            Some("USD")
        );
    }

    // A backend Notice (a rejected mode/model/effort set, or a codex out-of-turn
    // warning/deprecation) must NOT be silently dropped at the seam — the backends emit
    // it precisely so the failure is visible. It surfaces as a `Tips` frame the frontend
    // already renders, carrying the notice level → TipType and the message verbatim.
    #[test]
    fn notice_surfaces_as_tips() {
        for (level, expected) in [
            (roseui_session::NoticeLevel::Warning, TipType::Warning),
            (roseui_session::NoticeLevel::Info, TipType::Info),
        ] {
            let events = translate_event(
                SessionEvent::Notice {
                    level,
                    message: "set effort: rejected by agent".into(),
                },
                "conv-1",
                false,
            );
            assert_eq!(events.len(), 1, "a Notice must produce exactly one Tips frame");
            let crate::protocol::events::AgentStreamEvent::Tips(tip) = &events[0] else {
                panic!("expected Tips, got {:?}", events[0]);
            };
            assert_eq!(tip.content, "set effort: rejected by agent");
            assert_eq!(tip.tip_type, expected, "notice level maps to tip severity");
            assert!(tip.code.is_none(), "ad-hoc notice carries no i18n code");
        }
    }

    // A ConfigChanged must NOT produce any stream frame: the origin frontend's mode/
    // model pickers track selection in local state (optimistic on the PUT + its REST
    // response), and an `acp_config_option` frame would fall into origin useAcpMessage's
    // `default:` arm and light a spurious turn timer bar. The selection is still
    // persisted separately (see persist_tests::config_changed_persists_mode_and_model).
    #[test]
    fn config_changed_emits_no_frame() {
        let events = translate_event(
            SessionEvent::ConfigChanged {
                mode: Some("plan".into()),
                model: Some("claude-opus-4-8".into()),
            },
            "conv-1",
            false,
        );
        assert!(
            events.is_empty(),
            "ConfigChanged must emit no stream frame, got {events:?}"
        );
    }

    // A codex Plan surfaces as AgentStreamEvent::Plan with the frontend's expected
    // entry shape: content + snake_case status (the MessagePlan renderer ticks on
    // status === 'completed'). A raw serde dump of PlanStatus would send PascalCase
    // and the card would never tick — this guards the explicit mapping.
    #[test]
    fn plan_surfaces_with_snake_case_status() {
        use roseui_session::{PlanEntry, PlanStatus};
        let events = translate_event(
            SessionEvent::Plan {
                entries: vec![
                    PlanEntry {
                        content: "step one".into(),
                        status: PlanStatus::Completed,
                        priority: None,
                    },
                    PlanEntry {
                        content: "step two".into(),
                        status: PlanStatus::InProgress,
                        priority: None,
                    },
                ],
                explanation: None,
            },
            "conv-1",
            false,
        );
        assert_eq!(events.len(), 1);
        let crate::protocol::events::AgentStreamEvent::Plan(data) = &events[0] else {
            panic!("expected Plan, got {:?}", events[0]);
        };
        assert_eq!(data.entries.len(), 2);
        assert_eq!(data.entries[0]["content"], "step one");
        assert_eq!(
            data.entries[0]["status"], "completed",
            "status must be snake_case for the frontend"
        );
        assert_eq!(data.entries[1]["status"], "in_progress");
    }

    // A user-cancelled turn must NOT surface an error Tips (claude reports the
    // interrupt as an is_error result, but the user asked for it — surfacing it pops
    // a spurious red bubble on every cancel). Only a plain Finish is emitted (no Error).
    #[test]
    fn cancelled_turn_emits_finish_without_error() {
        use roseui_session::{CancelReason, TurnOutcome};
        let events = translate_event(
            SessionEvent::TurnResult {
                is_error: true,
                api_error_status: None,
                result_text: "error_during_execution".into(),
                epoch: 0,
                outcome: TurnOutcome::Cancelled {
                    reason: CancelReason::UserCancel,
                },
            },
            "conv-1",
            false,
        );
        assert!(
            !events.iter().any(|e| matches!(e, AgentStreamEvent::Error(_))),
            "a cancelled turn must not emit an Error, got {events:?}"
        );
        assert!(
            events.iter().any(|e| matches!(e, AgentStreamEvent::Finish(_))),
            "a cancelled turn still finishes"
        );
    }

    // A genuine (non-cancel) error terminates as AgentStreamEvent::Error carrying the
    // full origin error model (code/ownership/retryable), NOT a plain Tips and NOT a
    // Finish (Error is itself the relay terminal). This is what lets the relay
    // classify + auto-replay and the frontend render ownership/feedback.
    #[test]
    fn errored_turn_emits_rich_error_terminal() {
        use roseui_session::{StopReason, TurnOutcome};
        let events = translate_event(
            SessionEvent::TurnResult {
                is_error: true,
                api_error_status: Some(500),
                result_text: "upstream exploded".into(),
                epoch: 0,
                outcome: TurnOutcome::Completed {
                    stop_reason: StopReason::EndTurn,
                },
            },
            "conv-1",
            false,
        );
        assert_eq!(
            events.len(),
            1,
            "a real error is a single Error terminal, got {events:?}"
        );
        let AgentStreamEvent::Error(data) = &events[0] else {
            panic!("expected Error terminal, got {:?}", events[0]);
        };
        // Classified through the origin error path → carries a code + ownership +
        // retryable (not a bare message). The exact code depends on the classifier;
        // the contract is that these fields are POPULATED, not None.
        assert!(!data.message.is_empty());
        assert!(data.code.is_some(), "error must carry a classified code");
        assert!(data.retryable.is_some(), "error must carry a retryable flag");
        // Must NOT also emit a Finish (Error is the terminal).
        assert!(!events.iter().any(|e| matches!(e, AgentStreamEvent::Finish(_))));
    }

    // A mid-turn process crash (Detached with a signal / non-zero / unknown exit and
    // NO prior terminal result) surfaces as a rich AgentStreamEvent::Error carrying the
    // legacy `UserAgentDisconnected` classification (code + retryable + ownership), with
    // the allowlisted redacted_summary appended to the message. This restores the ACP
    // path's `AcpError::Disconnected` terminal the direct-CLI bridge had collapsed to a
    // bare Finish (a dead CLI used to render as a normal empty completion).
    #[test]
    fn detached_crash_surfaces_as_rich_disconnect_error() {
        use roseui_session::ExitStatusLite;
        let events = translate_event(
            SessionEvent::Detached {
                exit: Some(ExitStatusLite {
                    code: None,
                    signal: Some(9),
                }),
                redacted_summary: Some("process killed (SIGKILL)".into()),
            },
            // No terminal TurnResult was seen this turn → a genuine mid-turn crash.
            "conv-1",
            false,
        );
        assert_eq!(events.len(), 1, "a crash is a single Error terminal, got {events:?}");
        let AgentStreamEvent::Error(data) = &events[0] else {
            panic!("expected Error terminal, got {:?}", events[0]);
        };
        // Classified through the SAME path legacy used → carries code + retryable, and
        // the allowlisted summary rides the user-facing message.
        assert!(data.code.is_some(), "crash must carry a classified code");
        assert!(data.retryable.is_some(), "crash must carry a retryable flag");
        assert!(
            data.message.contains("process killed (SIGKILL)"),
            "redacted_summary must ride the message, got {:?}",
            data.message
        );
        assert!(
            !events.iter().any(|e| matches!(e, AgentStreamEvent::Finish(_))),
            "Error is the terminal — must NOT also emit Finish"
        );
    }

    // A Detached that ARRIVES AFTER a terminal TurnResult is an absorbed teardown (the
    // reducer's I10 / `crash_outcome → FollowResult`), NOT a crash — it must end with a
    // plain Finish, never an error card (otherwise every normal turn's process exit
    // would pop a spurious "disconnected" error).
    #[test]
    fn detached_after_terminal_result_is_plain_finish() {
        use roseui_session::ExitStatusLite;
        let events = translate_event(
            SessionEvent::Detached {
                exit: Some(ExitStatusLite {
                    code: Some(1),
                    signal: None,
                }),
                redacted_summary: Some("exited".into()),
            },
            // The turn already reached its terminal TurnResult.
            "conv-1",
            true,
        );
        assert!(
            !events.iter().any(|e| matches!(e, AgentStreamEvent::Error(_))),
            "a post-terminal Detached must not emit an Error, got {events:?}"
        );
        assert!(
            events.iter().any(|e| matches!(e, AgentStreamEvent::Finish(_))),
            "an absorbed teardown still finishes, got {events:?}"
        );
    }

    // A clean exit-0 with no result (a genuinely blank turn) is NOT a crash — it ends
    // with a plain Finish (the empty-turn Tip rides the pump's status match, not here).
    #[test]
    fn detached_clean_exit_zero_is_plain_finish() {
        use roseui_session::ExitStatusLite;
        let events = translate_event(
            SessionEvent::Detached {
                exit: Some(ExitStatusLite {
                    code: Some(0),
                    signal: None,
                }),
                redacted_summary: None,
            },
            "conv-1",
            false,
        );
        assert!(
            !events.iter().any(|e| matches!(e, AgentStreamEvent::Error(_))),
            "a clean exit-0 must not emit an Error, got {events:?}"
        );
        assert!(
            events.iter().any(|e| matches!(e, AgentStreamEvent::Finish(_))),
            "a clean exit-0 finishes, got {events:?}"
        );
    }

    // --- empty-turn (blank-reply) diagnostic Tip, mirroring the ACP path ---

    fn tip_code(outcome: roseui_session::TurnOutcome) -> Option<(TipType, String)> {
        empty_turn_tip(&outcome).map(|t| (t.tip_type, t.code.unwrap()))
    }

    #[test]
    fn empty_turn_endturn_is_info_generic_code() {
        use roseui_session::{StopReason, TurnOutcome};
        // Both the legacy default `EndTurn` and the modern `Completed{EndTurn}` map
        // to the informational "no reply" note.
        for outcome in [
            TurnOutcome::EndTurn,
            TurnOutcome::Completed {
                stop_reason: StopReason::EndTurn,
            },
        ] {
            assert_eq!(tip_code(outcome), Some((TipType::Info, "ACP_EMPTY_TURN".to_owned())));
        }
    }

    #[test]
    fn empty_turn_truncation_and_refusal_map_to_acp_warning_codes() {
        use roseui_session::{StopReason, TruncationKind, TurnOutcome};
        // Exactly the codes the ACP path emits (agent_session_flow.rs empty_finish_tip_code).
        assert_eq!(
            tip_code(TurnOutcome::Completed {
                stop_reason: StopReason::Truncated(TruncationKind::MaxTokens),
            }),
            Some((TipType::Warning, "ACP_EMPTY_TURN_MAX_TOKENS".to_owned()))
        );
        assert_eq!(
            tip_code(TurnOutcome::Completed {
                stop_reason: StopReason::Truncated(TruncationKind::MaxTurns),
            }),
            Some((TipType::Warning, "ACP_EMPTY_TURN_MAX_TURN_REQUESTS".to_owned()))
        );
        assert_eq!(
            tip_code(TurnOutcome::Completed {
                stop_reason: StopReason::Refused { category: None },
            }),
            Some((TipType::Warning, "ACP_EMPTY_TURN_REFUSAL".to_owned()))
        );
    }

    #[test]
    fn empty_turn_other_truncation_and_failed_fall_back_to_generic_warning() {
        use roseui_session::{StopReason, TruncationKind, TurnOutcome};
        // Truncation kinds with no dedicated ACP code, plus a clean Failed, still
        // warn the user rather than silently rendering an empty bubble.
        for outcome in [
            TurnOutcome::Completed {
                stop_reason: StopReason::Truncated(TruncationKind::ContextWindow),
            },
            TurnOutcome::Completed {
                stop_reason: StopReason::Truncated(TruncationKind::Budget),
            },
            TurnOutcome::Failed,
        ] {
            assert_eq!(tip_code(outcome), Some((TipType::Warning, "ACP_EMPTY_TURN".to_owned())));
        }
    }

    #[test]
    fn empty_turn_cancelled_never_tips() {
        use roseui_session::{CancelReason, TurnOutcome};
        // A user interrupt is not a blank reply — no spurious tip.
        assert!(
            empty_turn_tip(&TurnOutcome::Cancelled {
                reason: CancelReason::UserCancel,
            })
            .is_none()
        );
    }

    #[test]
    fn user_visible_output_predicate_matches_renderable_frames() {
        // Frames that render in chat count as visible; lifecycle/metadata frames do not.
        assert!(event_is_user_visible_output(&AgentStreamEvent::Text(TextEventData {
            content: "hi".into(),
        })));
        assert!(event_is_user_visible_output(&tool_call(
            "c",
            "Read",
            ToolCallStatus::Running
        )));
        assert!(!event_is_user_visible_output(&AgentStreamEvent::Finish(
            FinishEventData::default()
        )));
        assert!(!event_is_user_visible_output(&AgentStreamEvent::SegmentBreak));
    }
}

#[cfg(test)]
mod persist_tests {
    //! The pump's persistence hookup — the writes the legacy ACP path performed via
    //! `AcpSessionSyncService` but which this direct-CLI path must do itself. Without
    //! these the resume anchor + mode/model precedence source are never written.
    use super::*;
    use roseui_db::{CreateAcpSessionParams, IAcpSessionRepository, SqliteAcpSessionRepository, init_database_memory};

    // Returns both the repo and the owning Database — the caller binds the Database
    // for the test's lifetime (the cloned SqlitePool keeps the in-memory DB alive).
    async fn seeded_repo() -> (Arc<dyn IAcpSessionRepository>, roseui_db::Database) {
        let db = init_database_memory().await.unwrap();
        // The scoped repo authorizes through the conversations parent chain, so
        // seed the owning user + conversation the acp_session row hangs off.
        sqlx::query(
            "INSERT INTO users (id, username, password_hash, created_at, updated_at) \
             VALUES ('user-1', 'user-1', 'hash', 0, 0)",
        )
        .execute(db.pool())
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO conversations (id, user_id, name, type, status, created_at, updated_at, extra) \
             VALUES ('conv-1', 'user-1', 'conv-1', 'acp', 'pending', 0, 0, '{}')",
        )
        .execute(db.pool())
        .await
        .unwrap();
        let repo: Arc<dyn IAcpSessionRepository> = Arc::new(SqliteAcpSessionRepository::new(db.pool().clone()));
        repo.create(&CreateAcpSessionParams {
            user_id: "user-1",
            conversation_id: "conv-1",
            agent_source: "builtin",
            agent_id: "claude",
        })
        .await
        .unwrap();
        (repo, db)
    }

    #[tokio::test]
    async fn backend_bound_persists_resume_anchor() {
        let (repo, _db) = seeded_repo().await;
        persist_side_effects(
            repo.as_ref(),
            "user-1",
            "conv-1",
            &SessionEvent::BackendBound {
                backend_session_id: Some("bsid-abc".into()),
            },
        )
        .await;
        let row = repo
            .get_for_user("user-1", "conv-1")
            .await
            .unwrap()
            .expect("row exists");
        assert_eq!(
            row.session_id.as_deref(),
            Some("bsid-abc"),
            "BackendBound must write the resume anchor build_session_instance reads back"
        );
    }

    #[tokio::test]
    async fn backend_bound_none_does_not_clobber_anchor() {
        let (repo, _db) = seeded_repo().await;
        repo.update_session_id_for_user("user-1", "conv-1", "bsid-existing")
            .await
            .unwrap();
        persist_side_effects(
            repo.as_ref(),
            "user-1",
            "conv-1",
            &SessionEvent::BackendBound {
                backend_session_id: None,
            },
        )
        .await;
        let row = repo
            .get_for_user("user-1", "conv-1")
            .await
            .unwrap()
            .expect("row exists");
        assert_eq!(
            row.session_id.as_deref(),
            Some("bsid-existing"),
            "BackendBound{{None}} (lost-backend self-heal) must leave the stored anchor intact"
        );
    }

    #[tokio::test]
    async fn config_changed_persists_mode_and_model() {
        let (repo, _db) = seeded_repo().await;
        persist_side_effects(
            repo.as_ref(),
            "user-1",
            "conv-1",
            &SessionEvent::ConfigChanged {
                mode: Some("plan".into()),
                model: Some("claude-opus-4-8".into()),
            },
        )
        .await;
        let state = repo
            .load_runtime_state_for_user("user-1", "conv-1")
            .await
            .unwrap()
            .expect("runtime state");
        assert_eq!(state.current_mode_id.as_deref(), Some("plan"));
        assert_eq!(state.current_model_id.as_deref(), Some("claude-opus-4-8"));
    }

    // #4: a `set_config_option("effort", ...)` must persist the level into
    // config_selections — claude emits NO ConfigChanged for effort, so unless
    // set_config_option writes it directly, effort is lost across respawn/resume.
    // This drives the real chokepoint (a task built around a NoStreamBackend + the
    // seeded repo) and asserts the level lands in the persisted config_selections.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn set_config_option_effort_persists_into_config_selections() {
        use super::pump_tests::StaticCapsBackend;
        let (repo, _db) = seeded_repo().await;
        let backend: Arc<dyn SessionBackend> = Arc::new(StaticCapsBackend);
        let task = SessionAgentTask::new(
            AgentType::Acp,
            "conv-1".into(),
            "user-1".into(),
            "/w".into(),
            backend,
            Some(repo.clone()),
        );

        let resp = task.set_config_option("effort", "high").await.unwrap();
        assert!(
            matches!(
                resp.confirmation,
                roseui_api_types::ConfigOptionConfirmation::CommandAck
            ),
            "effort reports CommandAck (no picker current_value to observe)"
        );

        // Persisted under the effort key so build_session_instance can re-apply it.
        let state = repo
            .load_runtime_state_for_user("user-1", "conv-1")
            .await
            .unwrap()
            .expect("runtime state");
        let selections: std::collections::HashMap<String, String> = serde_json::from_str(
            state
                .config_selections_json
                .as_deref()
                .expect("config_selections persisted"),
        )
        .unwrap();
        assert_eq!(
            selections.get(EFFORT_CONFIG_KEY).map(String::as_str),
            Some("high"),
            "the chosen effort must be persisted into config_selections"
        );
    }

    /// A backend whose dispatch reports the session as gone (codex resume-poison:
    /// `bound_thread_within` fails fast after a rejected thread/resume).
    struct DeadSessionBackend;

    #[async_trait::async_trait]
    impl SessionBackend for DeadSessionBackend {
        async fn dispatch(&self, _c: Command) -> Result<CommandReceipt, BackendError> {
            Err(BackendError::SessionNotFound(
                "codex thread/resume failed: no rollout found for thread id th-dead".into(),
            ))
        }
        fn events(&self) -> BoxStream<'static, SessionEnvelope> {
            use futures_util::StreamExt as _;
            futures_util::stream::empty().boxed()
        }
        fn capabilities(&self) -> Capabilities {
            Capabilities::default()
        }
    }

    // ELECTRON-3Q0 fix B2: a DISPATCH-time dead-session error never becomes a
    // `TurnResult` on the event pump, so the stream-side self-heal
    // (`is_dead_resume_anchor`) cannot clear the anchor for it. send_message must
    // clear it directly and classify the failure as the retryable
    // UserAgentSessionNotFound so the turn orchestrator auto-replays (Fresh).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn send_dispatch_session_not_found_clears_anchor_and_classifies() {
        let (repo, _db) = seeded_repo().await;
        repo.update_session_id_for_user("user-1", "conv-1", "dead-anchor")
            .await
            .unwrap();
        let backend: Arc<dyn SessionBackend> = Arc::new(DeadSessionBackend);
        let task = SessionAgentTask::new(
            AgentType::Acp,
            "conv-1".into(),
            "user-1".into(),
            "/w".into(),
            backend,
            Some(repo.clone()),
        );

        let err = crate::agent_task::IAgentTask::send_message(
            task.as_ref(),
            SendMessageData {
                content: "hi".into(),
                msg_id: "m1".into(),
                turn_id: None,
                files: Vec::new(),
                inject_skills: Vec::new(),
            },
        )
        .await
        .expect_err("dispatch fails with SessionNotFound");

        assert_eq!(
            err.code(),
            Some(roseui_api_types::AgentErrorCode::UserAgentSessionNotFound),
            "classified as the retryable session-not-found so TurnRecoveryPolicy replays once"
        );
        let row = repo
            .get_for_user("user-1", "conv-1")
            .await
            .unwrap()
            .expect("row exists");
        assert!(
            row.session_id.is_none(),
            "a dispatch-time dead session must clear the resume anchor — the replay/next send opens Fresh"
        );
    }

    // A backend that advertises per-model reasoning efforts (claude `supportedEffortLevels`),
    // used to prove the effort axis is surfaced to the frontend picker. The prior gap:
    // `get_config_options` emitted only mode+model, so the origin frontend's
    // `deriveSelectOption(..., 'thought_level', ['reasoning_effort'])` found nothing and the
    // top-right selector never showed a thinking/effort group even though claude advertises
    // it and the backend can set it.
    use roseui_session::{
        Admission, BackendError, Capabilities, Command, CommandReceipt, SessionBackend, SessionEnvelope,
    };
    use futures_util::stream::BoxStream;

    struct EffortCapsBackend;

    #[async_trait::async_trait]
    impl SessionBackend for EffortCapsBackend {
        async fn dispatch(&self, _c: Command) -> Result<CommandReceipt, BackendError> {
            Ok(CommandReceipt {
                accepted: true,
                admission: Admission::NoTurn,
                turn_gen: 0,
            })
        }
        fn events(&self) -> BoxStream<'static, SessionEnvelope> {
            use futures_util::StreamExt as _;
            futures_util::stream::empty().boxed()
        }
        fn capabilities(&self) -> Capabilities {
            use roseui_session::ModelInfo;
            Capabilities {
                available_models: vec![
                    ModelInfo {
                        id: "opus".into(),
                        name: "Opus".into(),
                        description: None,
                        reasoning_efforts: vec!["low".into(), "medium".into(), "high".into(), "max".into()],
                    },
                    ModelInfo {
                        id: "haiku".into(),
                        name: "Haiku".into(),
                        description: None,
                        reasoning_efforts: vec![],
                    },
                ],
                current_model: Some("opus".into()),
                ..Default::default()
            }
        }
    }

    // The effort ("thought level") axis MUST be surfaced as a config option so the origin
    // frontend renders it in the top-right model selector (parity with the ACP path's
    // `thought_level` option). Asserts the exact shape the frontend keys off: category
    // `thought_level`, id `reasoning_effort`, and the current model's advertised levels.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn get_config_options_surfaces_reasoning_effort_for_current_model() {
        let backend: Arc<dyn SessionBackend> = Arc::new(EffortCapsBackend);
        let task = SessionAgentTask::new(
            AgentType::Acp,
            "conv-1".into(),
            "user-1".into(),
            "/w".into(),
            backend,
            None,
        );
        let snapshot = task.get_config_options().await.unwrap();
        let effort = snapshot
            .config_options
            .iter()
            .find(|o| o.category.as_deref() == Some("thought_level"))
            .expect("effort axis must be surfaced as a thought_level config option");
        assert_eq!(
            effort.id, "reasoning_effort",
            "canonical id the frontend fallback matches"
        );
        assert_eq!(effort.option_type, "select");
        let values: Vec<&str> = effort.options.iter().map(|o| o.value.as_str()).collect();
        assert_eq!(
            values,
            vec!["low", "medium", "high", "max"],
            "the current model's advertised efforts"
        );
    }

    // A model with no advertised efforts (claude `haiku`) must NOT get an effort option —
    // an empty select would render a dead, choice-less group.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn get_config_options_omits_effort_when_current_model_has_none() {
        struct HaikuCurrentBackend;
        #[async_trait::async_trait]
        impl SessionBackend for HaikuCurrentBackend {
            async fn dispatch(&self, _c: Command) -> Result<CommandReceipt, BackendError> {
                Ok(CommandReceipt {
                    accepted: true,
                    admission: Admission::NoTurn,
                    turn_gen: 0,
                })
            }
            fn events(&self) -> BoxStream<'static, SessionEnvelope> {
                use futures_util::StreamExt as _;
                futures_util::stream::empty().boxed()
            }
            fn capabilities(&self) -> Capabilities {
                use roseui_session::ModelInfo;
                Capabilities {
                    available_models: vec![ModelInfo {
                        id: "haiku".into(),
                        name: "Haiku".into(),
                        description: None,
                        reasoning_efforts: vec![],
                    }],
                    current_model: Some("haiku".into()),
                    ..Default::default()
                }
            }
        }
        let backend: Arc<dyn SessionBackend> = Arc::new(HaikuCurrentBackend);
        let task = SessionAgentTask::new(
            AgentType::Acp,
            "conv-1".into(),
            "user-1".into(),
            "/w".into(),
            backend,
            None,
        );
        let snapshot = task.get_config_options().await.unwrap();
        assert!(
            snapshot
                .config_options
                .iter()
                .all(|o| o.category.as_deref() != Some("thought_level")),
            "a model with no advertised efforts must not surface an (empty) effort option"
        );
    }

    // Setting effort must read back as Observed with the requested level (the frontend's
    // `hasObservedValue` contract), driven by the optimistic override — claude emits no
    // effort echo, so without the override the switch would downgrade to command_ack and
    // the frontend would reject it.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn set_config_option_effort_returns_observed_via_override() {
        let (repo, _db) = seeded_repo().await;
        let backend: Arc<dyn SessionBackend> = Arc::new(EffortCapsBackend);
        let task = SessionAgentTask::new(
            AgentType::Acp,
            "conv-1".into(),
            "user-1".into(),
            "/w".into(),
            backend,
            Some(repo),
        );
        let resp = task.set_config_option("reasoning_effort", "high").await.unwrap();
        assert!(
            matches!(resp.confirmation, roseui_api_types::ConfigOptionConfirmation::Observed),
            "effort switch must be Observed (optimistic override), not command_ack"
        );
        let effort = resp
            .config_options
            .as_ref()
            .and_then(|opts| opts.iter().find(|o| o.category.as_deref() == Some("thought_level")))
            .expect("effort option in the observed snapshot");
        assert_eq!(
            effort.current_value.as_deref(),
            Some("high"),
            "the requested level is highlighted"
        );
    }

    // ── Defect 2: dead-resume-anchor self-heal ────────────────────────────
    // A turn that fails *because* the stored backend session no longer resolves must
    // NULL that anchor, or every subsequent send re-resumes the same dead id and the
    // conversation wedges forever. This restores the self-heal the direct-CLI path
    // dropped (clean-slate `Orchestrator` BackendBound{None}; legacy ACP
    // rebuild_after_session_not_found → clear_session_id).

    fn errored_turn(text: &str) -> SessionEvent {
        use roseui_session::{StopReason, TurnOutcome};
        SessionEvent::TurnResult {
            is_error: true,
            api_error_status: None,
            result_text: text.into(),
            epoch: 0,
            outcome: TurnOutcome::Completed {
                stop_reason: StopReason::EndTurn,
            },
        }
    }

    #[tokio::test]
    async fn no_conversation_found_clears_dead_anchor() {
        let (repo, _db) = seeded_repo().await;
        repo.update_session_id_for_user("user-1", "conv-1", "dead-sid")
            .await
            .unwrap();
        persist_side_effects(
            repo.as_ref(),
            "user-1",
            "conv-1",
            &errored_turn("No conversation found with session ID dead-sid"),
        )
        .await;
        let row = repo
            .get_for_user("user-1", "conv-1")
            .await
            .unwrap()
            .expect("row exists");
        assert_eq!(
            row.session_id, None,
            "an unrecoverable resume error must null the dead anchor so the next turn opens Fresh"
        );
    }

    #[tokio::test]
    async fn error_during_execution_clears_dead_anchor() {
        let (repo, _db) = seeded_repo().await;
        repo.update_session_id_for_user("user-1", "conv-1", "dead-sid")
            .await
            .unwrap();
        persist_side_effects(
            repo.as_ref(),
            "user-1",
            "conv-1",
            &errored_turn("error_during_execution"),
        )
        .await;
        let row = repo
            .get_for_user("user-1", "conv-1")
            .await
            .unwrap()
            .expect("row exists");
        assert_eq!(
            row.session_id, None,
            "error_during_execution is a structural resume failure"
        );
    }

    #[tokio::test]
    async fn ordinary_error_keeps_anchor() {
        let (repo, _db) = seeded_repo().await;
        repo.update_session_id_for_user("user-1", "conv-1", "live-sid")
            .await
            .unwrap();
        // A normal tool/turn error is NOT a resume failure — the anchor is still good.
        persist_side_effects(
            repo.as_ref(),
            "user-1",
            "conv-1",
            &errored_turn("the Bash tool exited with code 1"),
        )
        .await;
        let row = repo
            .get_for_user("user-1", "conv-1")
            .await
            .unwrap()
            .expect("row exists");
        assert_eq!(
            row.session_id.as_deref(),
            Some("live-sid"),
            "an ordinary error must NOT clear a still-valid resume anchor"
        );
    }

    #[tokio::test]
    async fn cancelled_turn_keeps_anchor_even_with_matching_text() {
        use roseui_session::{CancelReason, TurnOutcome};
        let (repo, _db) = seeded_repo().await;
        repo.update_session_id_for_user("user-1", "conv-1", "live-sid")
            .await
            .unwrap();
        // claude reports a user interrupt as is_error with cancel-noise text; the
        // anchor is still good, so a cancel must never trigger the self-heal.
        persist_side_effects(
            repo.as_ref(),
            "user-1",
            "conv-1",
            &SessionEvent::TurnResult {
                is_error: true,
                api_error_status: None,
                result_text: "error_during_execution".into(),
                epoch: 0,
                outcome: TurnOutcome::Cancelled {
                    reason: CancelReason::UserCancel,
                },
            },
        )
        .await;
        let row = repo
            .get_for_user("user-1", "conv-1")
            .await
            .unwrap()
            .expect("row exists");
        assert_eq!(
            row.session_id.as_deref(),
            Some("live-sid"),
            "a user-cancelled turn must NOT clear the anchor"
        );
    }

    // Pure classification matrix for the FCIS core, independent of the DB.
    #[test]
    fn is_dead_resume_anchor_matrix() {
        use roseui_session::{CancelReason, StopReason, TurnOutcome};
        let completed = |is_error: bool, text: &str| SessionEvent::TurnResult {
            is_error,
            api_error_status: None,
            result_text: text.into(),
            epoch: 0,
            outcome: TurnOutcome::Completed {
                stop_reason: StopReason::EndTurn,
            },
        };
        // Structural resume failures → dead anchor.
        assert!(is_dead_resume_anchor(&completed(
            true,
            "No conversation found: dead-sid"
        )));
        assert!(is_dead_resume_anchor(&completed(true, "error_during_execution")));
        // is_error:false is never a dead anchor, even with matching text.
        assert!(!is_dead_resume_anchor(&completed(false, "No conversation found")));
        // Ordinary error text is not a resume failure.
        assert!(!is_dead_resume_anchor(&completed(true, "tool call failed")));
        // A user-cancel is excluded even when the noise text matches.
        assert!(!is_dead_resume_anchor(&SessionEvent::TurnResult {
            is_error: true,
            api_error_status: None,
            result_text: "error_during_execution".into(),
            epoch: 0,
            outcome: TurnOutcome::Cancelled {
                reason: CancelReason::UserCancel,
            },
        }));
        // Non-TurnResult events are never dead anchors.
        assert!(!is_dead_resume_anchor(&SessionEvent::BackendBound {
            backend_session_id: Some("x".into()),
        }));
    }
}

#[cfg(test)]
mod pump_tests {
    //! End-to-end pump tests over a scripted `SessionBackend`: they assert the
    //! forwarded `AgentStreamEvent` sequence for a realistic claude event stream,
    //! locking in the ACP-alignment fixes found by the live frame-by-frame A/B
    //! (Start emitted by send_message before dispatch; opening ConfigChanged
    //! suppressed; Finish carries the CLI session id learned from BackendBound).
    use super::*;
    use roseui_session::{
        Admission, BackendError, Capabilities, Command, CommandReceipt, SessionBackend, SessionEnvelope, SessionEvent,
    };
    use futures_util::stream::BoxStream;

    /// Emits a fixed script on `events()`; `dispatch(Send)` admits a turn.
    struct ScriptBackend(Vec<SessionEnvelope>);

    #[async_trait::async_trait]
    impl SessionBackend for ScriptBackend {
        async fn dispatch(&self, c: Command) -> Result<CommandReceipt, BackendError> {
            let admission = match c {
                Command::Send { .. } => Admission::Started,
                _ => Admission::NoTurn,
            };
            Ok(CommandReceipt {
                accepted: true,
                admission,
                turn_gen: 1,
            })
        }
        fn events(&self) -> BoxStream<'static, SessionEnvelope> {
            use futures_util::StreamExt as _;
            futures_util::stream::iter(self.0.clone()).boxed()
        }
        fn capabilities(&self) -> Capabilities {
            Capabilities::default()
        }
    }

    fn env(event: SessionEvent) -> SessionEnvelope {
        SessionEnvelope {
            session_id: "conv-1".into(),
            turn_gen: 1,
            event,
        }
    }

    /// Like `ScriptBackend`, but holds every scripted frame until its `gate` is
    /// released — so a test can `subscribe()` to the task BEFORE any frame is
    /// pumped. This deterministically closes a subscribe-vs-pump race: the pump is
    /// spawned inside `build()` (it calls `events()` and starts polling at once), and
    /// on a `multi_thread` runtime it can otherwise emit — and DROP — frames before the
    /// test's `subscribe()` runs, because the broadcast channel keeps no pre-subscribe
    /// buffer. `drain_script` subscribes first, THEN releases the gate. This mirrors
    /// production, where the backend's event stream is subscribed at open but stays
    /// silent until the CLI emits (well after the frontend's WS subscribe).
    struct GatedScriptBackend {
        script: Vec<SessionEnvelope>,
        gate: Arc<tokio::sync::Notify>,
    }

    #[async_trait::async_trait]
    impl SessionBackend for GatedScriptBackend {
        async fn dispatch(&self, c: Command) -> Result<CommandReceipt, BackendError> {
            let admission = match c {
                Command::Send { .. } => Admission::Started,
                _ => Admission::NoTurn,
            };
            Ok(CommandReceipt {
                accepted: true,
                admission,
                turn_gen: 1,
            })
        }
        fn events(&self) -> BoxStream<'static, SessionEnvelope> {
            use futures_util::StreamExt as _;
            let gate = self.gate.clone();
            let script = self.script.clone();
            // First poll parks on the gate; only once released does the scripted
            // sequence flow — so no frame can predate the test's subscribe().
            futures_util::stream::once(async move { gate.notified().await })
                .flat_map(move |_| futures_util::stream::iter(script.clone()))
                .boxed()
        }
        fn capabilities(&self) -> Capabilities {
            Capabilities::default()
        }
    }

    // Build a task over a gated script, subscribe, THEN release the gate, and collect
    // every frame the task forwards until its (finite) event stream drains. Subscribing
    // before releasing is what makes the collection deterministic — see
    // `GatedScriptBackend`.
    async fn drain_script(script: Vec<SessionEnvelope>) -> Vec<AgentStreamEvent> {
        let gate = Arc::new(tokio::sync::Notify::new());
        let backend: Arc<dyn SessionBackend> = Arc::new(GatedScriptBackend {
            script,
            gate: gate.clone(),
        });
        let task = SessionAgentTask::new(
            AgentType::Acp,
            "conv-1".into(),
            "user-1".into(),
            "/w".into(),
            backend,
            None,
        );
        let mut rx = crate::agent_task::IAgentTask::subscribe(task.as_ref());
        // Release only AFTER subscribing: the pump cannot emit before we listen.
        gate.notify_one();
        let mut out = Vec::new();
        // The scripted stream is finite; once the pump drains it, no more frames
        // arrive, so a short bounded poll settles the collection (no live agent).
        while let Ok(Ok(ev)) = tokio::time::timeout(std::time::Duration::from_millis(300), rx.recv()).await {
            out.push(ev);
        }
        // Keep the task (hence the backend + pump) alive through the whole drain.
        drop(task);
        out
    }

    fn frame_name(ev: &AgentStreamEvent) -> &'static str {
        match ev {
            AgentStreamEvent::Start(_) => "start",
            AgentStreamEvent::Text(_) => "content",
            AgentStreamEvent::Finish(_) => "finish",
            AgentStreamEvent::AcpConfigOption(_) => "config",
            AgentStreamEvent::AcpContextUsage(_) => "usage",
            AgentStreamEvent::SegmentBreak => "SegmentBreak",
            _ => "other",
        }
    }

    // A ConfigChanged never produces a stream frame (it would fall into origin
    // useAcpMessage's `default:` arm and light a spurious timer bar), and the
    // BackendBound session id still reaches the Finish frame. (Mirrors the live A/B
    // finding: the session path must NOT emit a stray acp_config_option.)
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn config_change_emits_no_frame_and_bound_id_reaches_finish() {
        let script = vec![
            env(SessionEvent::BackendBound {
                backend_session_id: Some("sid-xyz".into()),
            }),
            env(SessionEvent::ConfigChanged {
                mode: Some("default".into()),
                model: None,
            }),
            env(SessionEvent::MessageDelta {
                item_id: "m".into(),
                text: "hi".into(),
            }),
            env(SessionEvent::TurnResult {
                is_error: false,
                api_error_status: None,
                result_text: String::new(),
                epoch: 0,
                outcome: roseui_session::TurnOutcome::EndTurn,
            }),
        ];
        let frames = drain_script(script).await;
        let seq: Vec<&str> = frames.iter().map(frame_name).collect();
        // No leading "config"; the turn body + terminal come through.
        assert!(
            !seq.contains(&"config"),
            "opening ConfigChanged must be suppressed, got {seq:?}"
        );
        assert_eq!(seq, vec!["content", "finish"], "got {seq:?}");
        // The Finish carries the CLI session id learned from BackendBound.
        let finish = frames.iter().rev().find(|f| matches!(f, AgentStreamEvent::Finish(_)));
        let AgentStreamEvent::Finish(data) = finish.expect("finish present") else {
            unreachable!()
        };
        assert_eq!(
            data.session_id.as_deref(),
            Some("sid-xyz"),
            "resume anchor rides Finish"
        );
    }

    // THE FIX (stuck View-Steps spinner after interrupt): a tool call whose turn
    // ends without a terminal `ToolResult` (user cancel / crash / dropped result)
    // must be closed with a `Canceled` frame BEFORE the Finish — otherwise the
    // persisted tool_call row stays status "work" forever and the frontend spinner
    // (`hasRunningToolMessages`) never stops, surviving reloads.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn cancelled_turn_closes_open_tool_calls_as_canceled_before_finish() {
        use roseui_session::{CancelReason, SubagentKind, TurnOutcome};
        let script = vec![
            env(SessionEvent::TurnStarted { epoch: 1 }),
            env(SessionEvent::ToolCall {
                tool_use_id: "call-1".into(),
                name: "Bash".into(),
                subagent: SubagentKind::Inline,
                input: serde_json::Value::Null,
                parent_tool_use_id: None,
            }),
            env(SessionEvent::TurnResult {
                is_error: true,
                api_error_status: None,
                result_text: "error_during_execution".into(),
                epoch: 1,
                outcome: TurnOutcome::Cancelled {
                    reason: CancelReason::UserCancel,
                },
            }),
        ];
        let frames = drain_script(script).await;
        let canceled_pos = frames.iter().position(|f| {
            matches!(f, AgentStreamEvent::ToolCall(d)
                if d.call_id == "call-1" && d.status == ToolCallStatus::Canceled && d.name == "Bash")
        });
        let finish_pos = frames.iter().position(|f| matches!(f, AgentStreamEvent::Finish(_)));
        let canceled_pos = canceled_pos.expect("open tool call must be closed with a Canceled frame");
        let finish_pos = finish_pos.expect("cancelled turn still finishes");
        assert!(
            canceled_pos < finish_pos,
            "Canceled must precede Finish (relay breaks the turn on Finish)"
        );
    }

    // A tool call that DID receive its ToolResult must NOT get a trailing Canceled
    // frame at turn end — only calls left open are closed.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn completed_tool_call_is_not_recanceled_at_turn_end() {
        use roseui_session::SubagentKind;
        let script = vec![
            env(SessionEvent::TurnStarted { epoch: 1 }),
            env(SessionEvent::ToolCall {
                tool_use_id: "call-1".into(),
                name: "Bash".into(),
                subagent: SubagentKind::Inline,
                input: serde_json::Value::Null,
                parent_tool_use_id: None,
            }),
            env(SessionEvent::ToolResult {
                tool_use_id: "call-1".into(),
                is_error: false,
                content: vec![],
                parent_tool_use_id: None,
            }),
            env(SessionEvent::TurnResult {
                is_error: false,
                api_error_status: None,
                result_text: "done".into(),
                epoch: 1,
                outcome: roseui_session::TurnOutcome::EndTurn,
            }),
        ];
        let frames = drain_script(script).await;
        assert!(
            !frames
                .iter()
                .any(|f| { matches!(f, AgentStreamEvent::ToolCall(d) if d.status == ToolCallStatus::Canceled) }),
            "a resolved tool call must not be closed again as Canceled, got {frames:?}"
        );
    }

    // The FIX (async catalog-arrival push): a `CatalogUpdated` (the direct-CLI
    // analogue of ACP's `emit_snapshot_events`) MUST project to exactly one
    // `AcpConfigOption` frame carrying BOTH the model and mode categories — the
    // frontend's `useAcpConfigOptions` replaces its whole snapshot on this frame, so
    // omitting a sibling category would wipe that picker. Before this the catalog
    // arrived ~6s after open with no upward frame, so the model selector stayed
    // disabled. Unlike `ConfigChanged` (suppressed), this frame is the intended signal.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn catalog_updated_projects_config_option_with_model_and_mode() {
        use roseui_session::{ModeInfo, ModelInfo};
        let script = vec![env(SessionEvent::CatalogUpdated {
            models: vec![
                ModelInfo {
                    id: "default".into(),
                    name: "Default".into(),
                    description: None,
                    reasoning_efforts: Vec::new(),
                },
                ModelInfo {
                    id: "opus".into(),
                    name: "Opus".into(),
                    description: None,
                    reasoning_efforts: Vec::new(),
                },
            ],
            modes: vec![ModeInfo {
                id: "plan".into(),
                name: "Plan".into(),
                description: None,
            }],
            slash_commands: Vec::new(),
        })];
        let frames = drain_script(script).await;
        let config = frames
            .iter()
            .find_map(|f| match f {
                AgentStreamEvent::AcpConfigOption(v) => Some(v),
                _ => None,
            })
            .expect("CatalogUpdated must project to an AcpConfigOption frame");
        let options = config
            .get("config_options")
            .and_then(|v| v.as_array())
            .expect("config_options array");
        let categories: Vec<&str> = options
            .iter()
            .filter_map(|o| o.get("category").and_then(|c| c.as_str()))
            .collect();
        assert!(
            categories.contains(&"model") && categories.contains(&"mode"),
            "both categories must ride the snapshot (else a sibling picker is wiped), got {categories:?}"
        );
        // The model category carries the parsed catalog so `canSwitch` derives true.
        let model_opt = options
            .iter()
            .find(|o| o.get("category").and_then(|c| c.as_str()) == Some("model"))
            .expect("model category");
        let model_values: Vec<&str> = model_opt
            .get("options")
            .and_then(|v| v.as_array())
            .expect("model options array")
            .iter()
            .filter_map(|o| o.get("value").and_then(|v| v.as_str()))
            .collect();
        assert_eq!(
            model_values,
            vec!["default", "opus"],
            "the parsed model ids ride the frame"
        );
    }

    // The FIX (async slash-command arrival push): claude advertises its command
    // list in the same late `initialize` response that carries the model/mode
    // catalog. The frontend's mount-time REST read returns empty before that
    // lands, and the legacy ACP path recovers via a live `AvailableCommands`
    // push — so the direct-CLI pump MUST emit one too, or the `/` menu stays
    // empty until a manual refetch. A CatalogUpdated carrying slash_commands
    // projects to an AvailableCommands frame whose commands carry name+description.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn catalog_updated_projects_available_commands_frame() {
        use roseui_session::SlashCommandInfo;
        let script = vec![env(SessionEvent::CatalogUpdated {
            models: Vec::new(),
            modes: Vec::new(),
            slash_commands: vec![
                SlashCommandInfo {
                    name: "compact".into(),
                    description: Some("Compact the conversation".into()),
                },
                SlashCommandInfo {
                    name: "clear".into(),
                    description: None,
                },
            ],
        })];
        let frames = drain_script(script).await;
        let commands = frames
            .iter()
            .find_map(|f| match f {
                AgentStreamEvent::AvailableCommands(data) => Some(&data.commands),
                _ => None,
            })
            .expect("CatalogUpdated with slash_commands must project to an AvailableCommands frame");
        let names: Vec<&str> = commands.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["compact", "clear"], "both commands ride the frame");
        assert_eq!(
            commands[0].description, "Compact the conversation",
            "description carries through"
        );
        assert_eq!(commands[1].description, "", "missing description becomes empty string");
    }

    // A CatalogUpdated with NO slash commands must not emit a spurious empty
    // AvailableCommands frame (which would clobber a REST-loaded menu).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn catalog_updated_without_slash_commands_emits_no_available_commands() {
        use roseui_session::ModelInfo;
        let script = vec![env(SessionEvent::CatalogUpdated {
            models: vec![ModelInfo {
                id: "opus".into(),
                name: "Opus".into(),
                description: None,
                reasoning_efforts: Vec::new(),
            }],
            modes: Vec::new(),
            slash_commands: Vec::new(),
        })];
        let frames = drain_script(script).await;
        assert!(
            !frames
                .iter()
                .any(|f| matches!(f, AgentStreamEvent::AvailableCommands(_))),
            "empty slash_commands must not emit an AvailableCommands frame"
        );
    }

    // send_message emits Start (before dispatch) stamped with the learned session id,
    // and PromptAccepted does NOT double-emit a Start.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn send_message_emits_single_leading_start_with_session_id() {
        // Pre-seed the backend-bound id via a script event, then let the pump learn it.
        let backend: Arc<dyn SessionBackend> = Arc::new(ScriptBackend(vec![env(SessionEvent::BackendBound {
            backend_session_id: Some("sid-abc".into()),
        })]));
        let task = SessionAgentTask::new(
            AgentType::Acp,
            "conv-1".into(),
            "user-1".into(),
            "/w".into(),
            backend,
            None,
        );
        // Let the pump process the BackendBound so session_id is known.
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let mut rx = crate::agent_task::IAgentTask::subscribe(task.as_ref());
        crate::agent_task::IAgentTask::send_message(
            task.as_ref(),
            SendMessageData {
                content: "hi".into(),
                msg_id: "m1".into(),
                turn_id: None,
                files: Vec::new(),
                inject_skills: Vec::new(),
            },
        )
        .await
        .unwrap();
        // The very next frame must be exactly one Start, carrying the session id.
        let first = tokio::time::timeout(std::time::Duration::from_millis(300), rx.recv())
            .await
            .expect("a frame")
            .expect("ok");
        let AgentStreamEvent::Start(data) = first else {
            panic!("expected Start first, got {}", frame_name(&first));
        };
        assert_eq!(data.session_id.as_deref(), Some("sid-abc"));
    }

    /// Read the single `.json` dump written under `dir`.
    fn read_only_json_dump(dir: &std::path::Path) -> serde_json::Value {
        let path = std::fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| p.extension().map(|x| x == "json").unwrap_or(false))
            .expect("a dump file must exist");
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    // DEV (`--dump-prompts`): send_message dumps the final input blocks as a
    // `session-cli-final-input` JSON, with the vendor label and raw text.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn send_message_dumps_final_input_when_enabled() {
        let tmp = tempfile::tempdir().unwrap();
        let backend: Arc<dyn SessionBackend> = Arc::new(ScriptBackend(vec![]));
        let task = SessionAgentTask::build(
            AgentType::Acp,
            "conv-1".into(),
            "user-1".into(),
            "/w".into(),
            backend,
            None,
            CatalogPreload::default(),
            Some(SessionPromptDump {
                dir: tmp.path().to_path_buf(),
                backend: "claude",
            }),
        );
        crate::agent_task::IAgentTask::send_message(
            task.as_ref(),
            SendMessageData {
                content: "hello".into(),
                msg_id: "m-1".into(),
                turn_id: None,
                files: Vec::new(),
                inject_skills: Vec::new(),
            },
        )
        .await
        .unwrap();

        let dump = read_only_json_dump(tmp.path());
        assert_eq!(dump["kind"], "session-cli-final-input");
        assert_eq!(dump["backend"], "claude");
        assert_eq!(dump["conversation_id"], "conv-1");
        assert_eq!(dump["msg_id"], "m-1");
        assert_eq!(dump["input"]["content"][0]["type"], "text");
        assert_eq!(dump["input"]["content"][0]["text"], "hello");
    }

    // Multimodal blocks are dumped RAW: an Image block keeps its full base64 body
    // (no redaction / no byte-len summary), per the dev-only "keep original" rule.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn send_message_dumps_image_block_raw_base64() {
        use base64::Engine as _;
        let tmp = tempfile::tempdir().unwrap();
        let backend: Arc<dyn SessionBackend> = Arc::new(ScriptBackend(vec![]));
        let task = SessionAgentTask::build(
            AgentType::Acp,
            "conv-1".into(),
            "user-1".into(),
            "/w".into(),
            backend,
            None,
            CatalogPreload::default(),
            Some(SessionPromptDump {
                dir: tmp.path().to_path_buf(),
                backend: "codex",
            }),
        );
        // Inject an image directly onto the task's dump path via a content slice
        // containing an Image block.
        task.dump_session_cli_final_input(
            &[ContentBlock::Image {
                data: vec![1u8, 2, 3],
                media_type: "image/png".into(),
            }],
            Some("m-img"),
        );

        let dump = read_only_json_dump(tmp.path());
        assert_eq!(dump["backend"], "codex");
        assert_eq!(dump["input"]["content"][0]["type"], "image");
        assert_eq!(dump["input"]["content"][0]["media_type"], "image/png");
        assert_eq!(
            dump["input"]["content"][0]["data"],
            base64::engine::general_purpose::STANDARD.encode([1u8, 2, 3])
        );
    }

    // With `--dump-prompts` off (`prompt_dump == None`), send_message writes nothing.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn send_message_no_dump_when_disabled() {
        let tmp = tempfile::tempdir().unwrap();
        let backend: Arc<dyn SessionBackend> = Arc::new(ScriptBackend(vec![]));
        let task = SessionAgentTask::build(
            AgentType::Acp,
            "conv-1".into(),
            "user-1".into(),
            "/w".into(),
            backend,
            None,
            CatalogPreload::default(),
            None,
        );
        crate::agent_task::IAgentTask::send_message(
            task.as_ref(),
            SendMessageData {
                content: "hi".into(),
                msg_id: "m-2".into(),
                turn_id: None,
                files: Vec::new(),
                inject_skills: Vec::new(),
            },
        )
        .await
        .unwrap();
        assert_eq!(
            std::fs::read_dir(tmp.path()).unwrap().count(),
            0,
            "no dump when --dump-prompts is off"
        );
    }

    // claude's non-blocking Workflow turn emits MULTIPLE `result` frames: a LAUNCH
    // result while subagents still run, then a TERMINAL result after every
    // `task_notification{completed}`. The pump must suppress the launch result's
    // Finish (else the relay closes and the workflow's completion message is lost)
    // and forward exactly ONE Finish — the terminal one, after the workflow drains.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn workflow_launch_result_finish_is_suppressed_until_workflow_completes() {
        use roseui_session::SubagentStatus;
        let script = vec![
            env(SessionEvent::ToolCall {
                tool_use_id: "toolu_wf".into(),
                name: "Task".into(),
                subagent: roseui_session::SubagentKind::Workflow,
                input: serde_json::Value::Null,
                parent_tool_use_id: None,
            }),
            // Workflow starts running (in-flight).
            env(SessionEvent::SubagentUpdate {
                r#ref: "task-1".into(),
                label: Some("wf".into()),
                status: SubagentStatus::Running,
                parent_ref: Some("toolu_wf".into()),
            }),
            env(SessionEvent::MessageDelta {
                item_id: "m".into(),
                text: "launching workflow".into(),
            }),
            // LAUNCH result — arrives while the workflow is still in flight. Its
            // Finish MUST be suppressed.
            env(SessionEvent::TurnResult {
                is_error: false,
                api_error_status: None,
                result_text: String::new(),
                epoch: 0,
                outcome: roseui_session::TurnOutcome::EndTurn,
            }),
            // Workflow completes (matches the fixture invariant: completed precedes
            // the terminal result).
            env(SessionEvent::SubagentUpdate {
                r#ref: "task-1".into(),
                label: Some("wf".into()),
                status: SubagentStatus::Completed,
                parent_ref: Some("toolu_wf".into()),
            }),
            env(SessionEvent::MessageDelta {
                item_id: "m2".into(),
                text: "workflow done".into(),
            }),
            // TERMINAL result — workflow drained, so this Finish is forwarded.
            env(SessionEvent::TurnResult {
                is_error: false,
                api_error_status: None,
                result_text: String::new(),
                epoch: 0,
                outcome: roseui_session::TurnOutcome::EndTurn,
            }),
        ];
        let frames = drain_script(script).await;
        let seq: Vec<&str> = frames.iter().map(frame_name).collect();
        // Exactly ONE finish, and BOTH text segments (launch reply + completion
        // message) reach the frontend before it.
        let finish_count = frames
            .iter()
            .filter(|f| matches!(f, AgentStreamEvent::Finish(_)))
            .count();
        assert_eq!(
            finish_count, 1,
            "only the terminal result's Finish is forwarded, got {seq:?}"
        );
        let text_count = frames.iter().filter(|f| matches!(f, AgentStreamEvent::Text(_))).count();
        assert_eq!(
            text_count, 2,
            "both the launch reply and the workflow completion text survive, got {seq:?}"
        );
        // The single Finish is LAST — the completion text precedes it.
        assert!(
            matches!(frames.last(), Some(AgentStreamEvent::Finish(_))),
            "the terminal Finish comes after the workflow completion message, got {seq:?}"
        );
        // The suppressed launch result emits exactly one SegmentBreak so the relay
        // closes the launch text segment and the completion reply renders as a
        // separate bubble instead of being concatenated under one msg_id.
        let break_count = frames
            .iter()
            .filter(|f| matches!(f, AgentStreamEvent::SegmentBreak))
            .count();
        assert_eq!(
            break_count, 1,
            "the suppressed launch result emits one SegmentBreak, got {seq:?}"
        );
        // The SegmentBreak sits BETWEEN the launch text and the completion text.
        // (frame_name maps Text -> "content".)
        let first_text = seq.iter().position(|k| *k == "content").unwrap();
        let seg_break = seq.iter().position(|k| *k == "SegmentBreak").unwrap();
        let last_text = seq.iter().rposition(|k| *k == "content").unwrap();
        assert!(
            first_text < seg_break && seg_break < last_text,
            "SegmentBreak must separate the two text batches, got {seq:?}"
        );
    }

    // A workflow-launch result that is itself an ERROR is NOT suppressed — the user
    // must see a genuine failure even mid-workflow (suppression covers only clean
    // completion ordering, per the fixture invariant).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn errored_result_is_not_suppressed_even_with_inflight_workflow() {
        use roseui_session::SubagentStatus;
        let script = vec![
            env(SessionEvent::SubagentUpdate {
                r#ref: "task-1".into(),
                label: Some("wf".into()),
                status: SubagentStatus::Running,
                parent_ref: None,
            }),
            env(SessionEvent::TurnResult {
                is_error: true,
                api_error_status: None,
                result_text: "provider exploded".into(),
                epoch: 0,
                outcome: roseui_session::TurnOutcome::EndTurn,
            }),
        ];
        let frames = drain_script(script).await;
        assert!(
            frames.iter().any(|f| matches!(f, AgentStreamEvent::Error(_))),
            "an error result terminates the turn even while a workflow is in flight, got {:?}",
            frames.iter().map(frame_name).collect::<Vec<_>>()
        );
    }

    /// Backend that reports one scripted pending permission — models a permission
    /// raised before the client subscribed, which the REST /confirmations recovery
    /// path must be able to rebuild.
    struct PendingPermBackend(roseui_session::PendingPermissionView);

    #[async_trait::async_trait]
    impl SessionBackend for PendingPermBackend {
        async fn dispatch(&self, _c: Command) -> Result<CommandReceipt, BackendError> {
            Ok(CommandReceipt {
                accepted: true,
                admission: Admission::NoTurn,
                turn_gen: 0,
            })
        }
        fn events(&self) -> BoxStream<'static, SessionEnvelope> {
            use futures_util::StreamExt as _;
            futures_util::stream::empty().boxed()
        }
        fn capabilities(&self) -> Capabilities {
            Capabilities::default()
        }
        fn pending_permission_requests(&self) -> Vec<roseui_session::PendingPermissionView> {
            vec![self.0.clone()]
        }
    }

    // get_confirmations must recover a pending AskUserQuestion as a question card
    // (its options), not an empty/allow-deny card — else a page refresh loses the
    // question and the turn hangs. (Regression guard: get_confirmations used to
    // return an empty Vec unconditionally.)
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn get_confirmations_recovers_pending_ask_user_question() {
        let backend: Arc<dyn SessionBackend> = Arc::new(PendingPermBackend(roseui_session::PendingPermissionView {
            request_id: "req-recover".into(),
            tool_name: "AskUserQuestion".into(),
            questions: Some(serde_json::json!({
                "questions": [{
                    "question": "Which?",
                    "options": [{"label": "A"}, {"label": "B"}]
                }]
            })),
        }));
        let task = SessionAgentTask::new(
            AgentType::Acp,
            "conv-1".into(),
            "user-1".into(),
            "/w".into(),
            backend,
            None,
        );
        let confs = task.get_confirmations();
        assert_eq!(confs.len(), 1, "the pending permission must be recovered");
        assert_eq!(
            confs[0].call_id, "req-recover",
            "card id == request_id for live/recovered de-dup"
        );
        let labels: Vec<&str> = confs[0].options.iter().map(|o| o.label.as_str()).collect();
        assert_eq!(
            labels,
            vec!["A", "B"],
            "recovered as the question's options, not allow/deny"
        );
    }

    // A pending ordinary tool permission recovers as the generic allow/deny card.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn get_confirmations_recovers_generic_permission() {
        let backend: Arc<dyn SessionBackend> = Arc::new(PendingPermBackend(roseui_session::PendingPermissionView {
            request_id: "req-tool".into(),
            tool_name: "Bash".into(),
            questions: None,
        }));
        let task = SessionAgentTask::new(
            AgentType::Acp,
            "conv-1".into(),
            "user-1".into(),
            "/w".into(),
            backend,
            None,
        );
        let confs = task.get_confirmations();
        assert_eq!(confs.len(), 1);
        let vals: Vec<String> = confs[0]
            .options
            .iter()
            .filter_map(|o| o.value.as_str().map(str::to_owned))
            .collect();
        assert_eq!(vals, vec!["allow", "allow_always", "reject"]);
    }

    /// Backend whose capabilities advertise modes+models but whose current_* never
    /// changes — models the claude constraint that an in-band switch is NOT reflected
    /// in capabilities(). Proves set_config_option's optimistic override makes the
    /// response satisfy the frontend's Observed contract regardless.
    pub(super) struct StaticCapsBackend;

    #[async_trait::async_trait]
    impl SessionBackend for StaticCapsBackend {
        async fn dispatch(&self, _c: Command) -> Result<CommandReceipt, BackendError> {
            Ok(CommandReceipt {
                accepted: true,
                admission: Admission::NoTurn,
                turn_gen: 0,
            })
        }
        fn events(&self) -> BoxStream<'static, SessionEnvelope> {
            use futures_util::StreamExt as _;
            futures_util::stream::empty().boxed()
        }
        fn capabilities(&self) -> Capabilities {
            use roseui_session::{ModeInfo, ModelInfo};
            Capabilities {
                available_modes: vec![
                    ModeInfo {
                        id: "default".into(),
                        name: "Default".into(),
                        description: None,
                    },
                    ModeInfo {
                        id: "plan".into(),
                        name: "Plan".into(),
                        description: None,
                    },
                ],
                current_mode: Some("default".into()),
                available_models: vec![
                    ModelInfo {
                        id: "opus".into(),
                        name: "Opus".into(),
                        description: None,
                        reasoning_efforts: vec![],
                    },
                    ModelInfo {
                        id: "sonnet".into(),
                        name: "Sonnet".into(),
                        description: None,
                        reasoning_efforts: vec![],
                    },
                ],
                current_model: Some("opus".into()),
                ..Default::default()
            }
        }
    }

    // set_config_option("mode") must return Observed with the requested value even
    // though capabilities().current_mode never moves — the optimistic override drives
    // the observed re-read. (Regression guard for the "switching mode → command_ack"
    // error the origin frontend rejects.)
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn set_config_option_mode_returns_observed_via_override() {
        let backend: Arc<dyn SessionBackend> = Arc::new(StaticCapsBackend);
        let task = SessionAgentTask::new(
            AgentType::Acp,
            "conv-1".into(),
            "user-1".into(),
            "/w".into(),
            backend,
            None,
        );
        let resp = task.set_config_option("mode", "plan").await.unwrap();
        assert!(
            matches!(resp.confirmation, roseui_api_types::ConfigOptionConfirmation::Observed),
            "mode switch must be Observed, got {:?}",
            resp.confirmation
        );
        let opts = resp.config_options.expect("config_options present");
        let mode_opt = opts.iter().find(|o| o.id == "mode").expect("mode option");
        assert_eq!(
            mode_opt.current_value.as_deref(),
            Some("plan"),
            "current_value reflects the switch"
        );
    }

    // Same for model — critical because claude gives set_model NO confirmation wire,
    // so ONLY the override can make it read back as observed.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn set_config_option_model_returns_observed_via_override() {
        let backend: Arc<dyn SessionBackend> = Arc::new(StaticCapsBackend);
        let task = SessionAgentTask::new(
            AgentType::Acp,
            "conv-1".into(),
            "user-1".into(),
            "/w".into(),
            backend,
            None,
        );
        let resp = task.set_config_option("model", "sonnet").await.unwrap();
        assert!(
            matches!(resp.confirmation, roseui_api_types::ConfigOptionConfirmation::Observed),
            "model switch must be Observed, got {:?}",
            resp.confirmation
        );
        // And get_model reflects the override too (picker highlight follows).
        let m = task.get_model().await.unwrap().model_info.expect("model_info");
        assert_eq!(m.current_model_id.as_deref(), Some("sonnet"));
    }

    // #3: a runtime switch to a value NOT in the advertised catalog is REJECTED
    // (bad_request), not silently dropped and not dispatched — the user's chosen
    // reject-and-report behavior. Non-empty catalog that omits the value → reject.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn set_config_option_rejects_invalid_mode_and_model() {
        let backend: Arc<dyn SessionBackend> = Arc::new(StaticCapsBackend);
        let task = SessionAgentTask::new(
            AgentType::Acp,
            "conv-1".into(),
            "user-1".into(),
            "/w".into(),
            backend,
            None,
        );

        let mode_err = task
            .set_config_option("mode", "no-such-mode")
            .await
            .expect_err("a mode outside the catalog must be rejected");
        assert!(
            matches!(mode_err, AgentError::BadRequest(_)),
            "invalid mode → BadRequest, got {mode_err:?}"
        );

        let model_err = task
            .set_config_option("model", "no-such-model")
            .await
            .expect_err("a model outside the catalog must be rejected");
        assert!(
            matches!(model_err, AgentError::BadRequest(_)),
            "invalid model → BadRequest, got {model_err:?}"
        );

        // The optimistic overrides must NOT have moved (nothing was dispatched).
        assert!(
            task.runtime.mode_override().is_none() && task.runtime.model_override().is_none(),
            "a rejected switch must not set an optimistic override"
        );
    }

    // codex ToolOutputDelta (streamed command stdout) must surface as tool_call
    // frames carrying the CUMULATIVE output (the frontend REPLACES output on merge,
    // so sending raw deltas would show only the last chunk). Each frame keys on the
    // item_id so the frontend appends to the right tool.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn tool_output_delta_accumulates_cumulative_output() {
        let script = vec![
            env(SessionEvent::ToolOutputDelta {
                item_id: "call_0".into(),
                text: "line-1\n".into(),
            }),
            env(SessionEvent::ToolOutputDelta {
                item_id: "call_0".into(),
                text: "line-2\n".into(),
            }),
        ];
        let frames = drain_script(script).await;
        let outputs: Vec<String> = frames
            .iter()
            .filter_map(|f| match f {
                AgentStreamEvent::ToolCall(d) if d.call_id == "call_0" => d.output.clone(),
                _ => None,
            })
            .collect();
        // Two frames: first the 1st chunk, then the cumulative 1st+2nd (not just "line-2").
        assert_eq!(outputs, vec!["line-1\n".to_string(), "line-1\nline-2\n".to_string()]);
    }

    // ── Defect 1: process-reap on task drop ───────────────────────────────
    // Faithfully models `ClaudeSessionBackend`: `events()` subscribes to a
    // broadcast `Sender` the backend struct OWNS, so the event stream stays open
    // (pending, never Closed) exactly as long as the backend is alive — and reaping
    // the child CLI happens in the backend's `Drop`. If `spawn_event_pump` captured a
    // backend `Arc`, that Arc would keep `event_tx` alive after the task Arc is
    // dropped, so the stream would never Close, the pump loop would never exit, the
    // backend would never Drop, and the child CLI would leak. This test proves the
    // pump holds ONLY the stream: dropping the sole task Arc must fire the backend's
    // Drop (i.e. reap) promptly.
    struct ReapBackend {
        // Owning this sender keeps `events()` subscribers pending while the backend
        // lives, mirroring the real backend's `event_tx` field.
        event_tx: broadcast::Sender<SessionEnvelope>,
        // Fired from `Drop` — stands in for "the child process was reaped".
        reap_signal: std::sync::Mutex<Option<tokio::sync::oneshot::Sender<()>>>,
    }

    #[async_trait::async_trait]
    impl SessionBackend for ReapBackend {
        async fn dispatch(&self, _c: Command) -> Result<CommandReceipt, BackendError> {
            Ok(CommandReceipt {
                accepted: true,
                admission: Admission::NoTurn,
                turn_gen: 0,
            })
        }
        fn events(&self) -> BoxStream<'static, SessionEnvelope> {
            use futures_util::StreamExt as _;
            // Subscribe here (like ClaudeSessionBackend::events): the returned stream
            // captures ONLY the Receiver, never `self`. It yields nothing and only
            // ends when every Sender — i.e. the field below — is dropped.
            let rx = self.event_tx.subscribe();
            futures_util::stream::unfold(rx, |mut rx| async move {
                match rx.recv().await {
                    Ok(env) => Some((env, rx)),
                    Err(_) => None,
                }
            })
            .boxed()
        }
        fn capabilities(&self) -> Capabilities {
            Capabilities::default()
        }
    }

    impl Drop for ReapBackend {
        fn drop(&mut self) {
            if let Some(tx) = self.reap_signal.lock().unwrap().take() {
                let _ = tx.send(());
            }
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn dropping_task_reaps_backend() {
        let (reaped_tx, reaped_rx) = tokio::sync::oneshot::channel();
        let (event_tx, _keep) = broadcast::channel(8);
        let backend: Arc<dyn SessionBackend> = Arc::new(ReapBackend {
            event_tx,
            reap_signal: std::sync::Mutex::new(Some(reaped_tx)),
        });
        // `_keep` is dropped here, so the ONLY remaining Sender is the backend's field
        // — the reap now hinges purely on the backend being dropped.
        drop(_keep);
        let task = SessionAgentTask::new(
            AgentType::Acp,
            "conv-1".into(),
            "user-1".into(),
            "/w".into(),
            backend,
            None,
        );
        // Let the pump subscribe and settle into its await.
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Drop the sole strong task Arc. Post-fix, this drops `task.backend` (the only
        // long-lived backend Arc) → ReapBackend::drop fires. Pre-fix, the pump held a
        // backend Arc and this would hang.
        drop(task);

        tokio::time::timeout(std::time::Duration::from_secs(2), reaped_rx)
            .await
            .expect("backend must be dropped (reaped) promptly after the task Arc is dropped")
            .expect("reap signal delivered");
    }

    // Build a persisted-handshake value in the SHAPE `spawn_catalog_writeback` stores
    // (the top-level `{available_models:[{id,label}], current_model_id}` column shape).
    fn handshake_with_catalog() -> roseui_api_types::AgentHandshake {
        roseui_api_types::AgentHandshake {
            available_models: Some(serde_json::json!({
                "available_models": [
                    {"id": "opus", "label": "Opus"},
                    {"id": "sonnet", "label": "Sonnet"},
                ],
                "current_model_id": "sonnet",
            })),
            available_modes: Some(serde_json::json!({
                "available_modes": [
                    {"id": "default", "name": "Default"},
                    {"id": "plan", "name": "Plan"},
                ],
                "current_mode_id": "plan",
            })),
            ..Default::default()
        }
    }

    // Cold-start resume: the backend's live capabilities() is still empty (initialize
    // round-trip not landed), but the persisted-handshake preload populates the picker
    // so it is NOT blank. This is the fix for "session-port history-open shows an empty
    // model list for ~seconds while ACP's persisted preload keeps it filled".
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn preload_serves_catalog_when_live_capabilities_empty() {
        // ScriptBackend has empty capabilities() → live catalog is absent.
        let backend: Arc<dyn SessionBackend> = Arc::new(ScriptBackend(Vec::new()));
        let task = SessionAgentTask::new_with_preload(
            AgentType::Acp,
            "conv-1".into(),
            "user-1".into(),
            "/w".into(),
            backend,
            None,
            &handshake_with_catalog(),
            None,
        );

        // get_model serves the preloaded catalog + persisted current model.
        let m = task
            .get_model()
            .await
            .unwrap()
            .model_info
            .expect("model_info from preload");
        assert_eq!(
            m.available_models.iter().map(|e| e.id.as_str()).collect::<Vec<_>>(),
            vec!["opus", "sonnet"],
        );
        assert_eq!(m.current_model_id.as_deref(), Some("sonnet"));

        // get_config_options renders both mode+model selects from the preload.
        let opts = task.get_config_options().await.unwrap().config_options;
        let model_opt = opts
            .iter()
            .find(|o| o.id == "model")
            .expect("model select from preload");
        assert_eq!(
            model_opt.options.iter().map(|o| o.value.as_str()).collect::<Vec<_>>(),
            vec!["opus", "sonnet"],
        );
        assert_eq!(model_opt.current_value.as_deref(), Some("sonnet"));
        let mode_opt = opts.iter().find(|o| o.id == "mode").expect("mode select from preload");
        assert_eq!(mode_opt.current_value.as_deref(), Some("plan"));

        // mode() serves the preloaded current mode.
        assert_eq!(task.mode().await.unwrap().mode, "plan");
    }

    // The live catalog OVERWRITES the preload the instant it is present: even though a
    // (stale) preload is supplied, a backend with non-empty capabilities() serves the
    // live values — matching ACP's "fill-when-empty, live-overwrites" semantics and
    // preventing a stale persisted catalog from masking a fresh engine's model list.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn live_capabilities_overwrite_stale_preload() {
        use super::pump_tests::StaticCapsBackend;
        // Preload advertises a codex-shaped catalog; live StaticCapsBackend advertises
        // opus/sonnet with current=opus. Live must win on every axis.
        let stale = roseui_api_types::AgentHandshake {
            available_models: Some(serde_json::json!({
                "available_models": [{"id": "stale-model", "label": "Stale"}],
                "current_model_id": "stale-model",
            })),
            ..Default::default()
        };
        let backend: Arc<dyn SessionBackend> = Arc::new(StaticCapsBackend);
        let task = SessionAgentTask::new_with_preload(
            AgentType::Acp,
            "conv-1".into(),
            "user-1".into(),
            "/w".into(),
            backend,
            None,
            &stale,
            None,
        );
        let m = task.get_model().await.unwrap().model_info.expect("model_info");
        assert_eq!(
            m.available_models.iter().map(|e| e.id.as_str()).collect::<Vec<_>>(),
            vec!["opus", "sonnet"],
            "live capabilities must overwrite the stale preload"
        );
        assert_eq!(m.current_model_id.as_deref(), Some("opus"));
    }

    // Cold-start pre-init: the backend has already seeded `current_model`/`current_mode`
    // from the RESOLVED snapshot (claude_conn spawn: `caps.current_model = config.model`,
    // config.model = persisted `current_model_id`), but `available_models` is still empty
    // (the initialize round-trip has not landed). The persisted-handshake preload's
    // current is STALE (frozen at the prior session's write-back, which does not re-run on
    // a mid-turn switch). The picker must show the backend's snapshot-seeded current — the
    // model claude actually runs — NOT the stale preload current, even though the LIST is
    // served from preload in this same window. This is the per-axis-independent fallback.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn caps_current_wins_over_stale_preload_while_list_is_empty() {
        // Backend: current seeded (user last switched to opus), but lists still empty.
        struct CurrentOnlyBackend;
        #[async_trait::async_trait]
        impl SessionBackend for CurrentOnlyBackend {
            async fn dispatch(&self, _c: Command) -> Result<CommandReceipt, BackendError> {
                Ok(CommandReceipt {
                    accepted: true,
                    admission: Admission::NoTurn,
                    turn_gen: 0,
                })
            }
            fn events(&self) -> BoxStream<'static, SessionEnvelope> {
                use futures_util::StreamExt as _;
                futures_util::stream::empty().boxed()
            }
            fn capabilities(&self) -> Capabilities {
                Capabilities {
                    current_model: Some("opus".into()),
                    current_mode: Some("default".into()),
                    ..Default::default()
                }
            }
        }

        // Preload (prior write-back) still says sonnet/plan — the pre-switch values.
        let backend: Arc<dyn SessionBackend> = Arc::new(CurrentOnlyBackend);
        let task = SessionAgentTask::new_with_preload(
            AgentType::Acp,
            "conv-1".into(),
            "user-1".into(),
            "/w".into(),
            backend,
            None,
            &handshake_with_catalog(),
            None,
        );

        let m = task.get_model().await.unwrap().model_info.expect("model_info");
        // LIST comes from the preload (backend list is empty pre-init)...
        assert_eq!(
            m.available_models.iter().map(|e| e.id.as_str()).collect::<Vec<_>>(),
            vec!["opus", "sonnet"],
            "list falls back to preload while the backend list is empty"
        );
        // ...but the CURRENT is the backend's snapshot-seeded model, NOT the stale preload.
        assert_eq!(
            m.current_model_id.as_deref(),
            Some("opus"),
            "current_model must be the backend's snapshot-seeded value, not the stale preload's sonnet"
        );

        let opts = task.get_config_options().await.unwrap().config_options;
        let model_opt = opts.iter().find(|o| o.id == "model").expect("model select");
        assert_eq!(model_opt.current_value.as_deref(), Some("opus"));
        let mode_opt = opts.iter().find(|o| o.id == "mode").expect("mode select");
        assert_eq!(
            mode_opt.current_value.as_deref(),
            Some("default"),
            "current_mode must be the backend's snapshot-seeded value, not the stale preload's plan"
        );
        assert_eq!(task.mode().await.unwrap().mode, "default");
    }
}

/// Layer D force-kill regression (spec §10.1/§10.2/§10.3/§10.7, plan §5 T4–T6):
/// the direct-CLI `SessionAgentTask` kill path. Before this fix `kill` was a
/// Drop-only no-op that silently failed while an orchestrator held an `Arc`
/// clone of the task (ELECTRON-3RW). These tests pin the fixed behavior: a
/// `UserCancelTimeout` kill emits a clean `Finish` AND really terminates the
/// backend, even with an external `Arc` held; and it is idempotent + isolated
/// from the non-force-kill reasons.
#[cfg(test)]
mod force_kill_tests {
    use super::*;
    use crate::agent_task::{AgentInstance, IAgentTask};
    use crate::types::SendMessageData;
    use roseui_common::{AgentKillReason, ConversationStatus};
    use roseui_session::{
        Admission, BackendError, Capabilities, Command, CommandReceipt, SessionBackend, SessionEnvelope,
    };
    use futures_util::stream::BoxStream;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// A backend whose `events()` NEVER terminates (the turn stays in flight —
    /// no natural `Finish`, exactly the workflow-in-progress window), and whose
    /// `terminate()` bumps a shared counter so the force-kill delegation is
    /// observable without a real process.
    struct TerminateCountingBackend {
        terminate_calls: Arc<AtomicUsize>,
    }

    #[async_trait::async_trait]
    impl SessionBackend for TerminateCountingBackend {
        async fn dispatch(&self, c: Command) -> Result<CommandReceipt, BackendError> {
            let admission = match c {
                Command::Send { .. } => Admission::Started,
                _ => Admission::NoTurn,
            };
            Ok(CommandReceipt {
                accepted: true,
                admission,
                turn_gen: 1,
            })
        }
        fn events(&self) -> BoxStream<'static, SessionEnvelope> {
            use futures_util::StreamExt as _;
            // Never yields → the turn never converges on its own; only kill can.
            futures_util::stream::pending().boxed()
        }
        fn capabilities(&self) -> Capabilities {
            Capabilities::default()
        }
        async fn terminate(&self) {
            self.terminate_calls.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn build_task_with_counter() -> (Arc<SessionAgentTask>, Arc<AtomicUsize>) {
        let counter = Arc::new(AtomicUsize::new(0));
        let backend: Arc<dyn SessionBackend> = Arc::new(TerminateCountingBackend {
            terminate_calls: Arc::clone(&counter),
        });
        let task = SessionAgentTask::new(
            AgentType::Acp,
            "conv-1".into(),
            "user-1".into(),
            "/w".into(),
            backend,
            None,
        );
        (task, counter)
    }

    /// Drive a turn to `Running` (emits `Start` on the runtime channel).
    async fn start_turn(task: &SessionAgentTask) {
        IAgentTask::send_message(
            task,
            SendMessageData {
                content: "hello".into(),
                msg_id: "m1".into(),
                turn_id: None,
                files: Vec::new(),
                inject_skills: Vec::new(),
            },
        )
        .await
        .expect("send accepted");
    }

    /// Return the next TERMINAL frame (`Finish`/`Error`), skipping lifecycle
    /// frames like `Start`. `None` if none arrives within the bounded window.
    async fn next_terminal(rx: &mut broadcast::Receiver<AgentStreamEvent>) -> Option<AgentStreamEvent> {
        loop {
            match tokio::time::timeout(std::time::Duration::from_millis(500), rx.recv()).await {
                Ok(Ok(ev)) => match ev {
                    AgentStreamEvent::Finish(_) | AgentStreamEvent::Error(_) => return Some(ev),
                    _ => continue,
                },
                _ => return None,
            }
        }
    }

    /// T4: `UserCancelTimeout` on the Session path forces a clean `Finish` AND a
    /// real backend `terminate`, even while an external `Arc<SessionAgentTask>`
    /// (the orchestrator) is still held — the case the old Drop-only kill missed.
    #[tokio::test]
    async fn user_cancel_timeout_forces_clean_finish_and_terminate_with_external_arc() {
        let (task, counter) = build_task_with_counter();
        let mut rx = IAgentTask::subscribe(task.as_ref());
        start_turn(task.as_ref()).await;
        assert_eq!(
            IAgentTask::status(task.as_ref()),
            Some(ConversationStatus::Running),
            "turn should be in flight before kill"
        );

        // An orchestrator legitimately holds a clone of the Arc for the whole turn.
        let orchestrator_hold = Arc::clone(&task);

        let inst = AgentInstance::Session(Arc::clone(&task));
        inst.kill_and_wait(Some(AgentKillReason::UserCancelTimeout)).await;

        // (a) clean Finish broadcast — NOT a crash Error.
        let terminal = next_terminal(&mut rx).await.expect("a terminal frame after kill");
        assert!(
            matches!(terminal, AgentStreamEvent::Finish(_)),
            "kill must broadcast a clean Finish (not Error), got {terminal:?}"
        );
        // (b) runtime converged to Finished.
        assert_eq!(IAgentTask::status(task.as_ref()), Some(ConversationStatus::Finished));
        // (c) backend really terminated once — independent of the still-held Arc.
        assert_eq!(
            counter.load(Ordering::SeqCst),
            1,
            "backend.terminate must be awaited exactly once"
        );
        assert!(
            Arc::strong_count(&task) >= 2,
            "orchestrator still holds the Arc — Drop did NOT fire, yet terminate ran"
        );

        drop(orchestrator_hold);
        drop(inst);
    }

    /// T5: idempotence — a repeated force-kill (or a late real Finish) does not
    /// re-broadcast; status stays `Finished`.
    #[tokio::test]
    async fn repeated_user_cancel_kill_does_not_double_broadcast() {
        let (task, _counter) = build_task_with_counter();
        let mut rx = IAgentTask::subscribe(task.as_ref());
        start_turn(task.as_ref()).await;

        let inst = AgentInstance::Session(Arc::clone(&task));
        inst.kill_and_wait(Some(AgentKillReason::UserCancelTimeout)).await;
        let first = next_terminal(&mut rx).await.expect("first Finish");
        assert!(matches!(first, AgentStreamEvent::Finish(_)));
        assert_eq!(IAgentTask::status(task.as_ref()), Some(ConversationStatus::Finished));

        // Second force-kill → emit_finish_once is a no-op in the Finished state.
        inst.kill_and_wait(Some(AgentKillReason::UserCancelTimeout)).await;
        let again = next_terminal(&mut rx).await;
        assert!(again.is_none(), "no second Finish broadcast, got {again:?}");
        assert_eq!(IAgentTask::status(task.as_ref()), Some(ConversationStatus::Finished));
    }

    /// T6: isolation — non-`UserCancelTimeout` reasons keep the original
    /// Drop-driven no-op: no injected Finish, no forced terminate, status
    /// unchanged.
    #[tokio::test]
    async fn non_user_cancel_reason_keeps_drop_driven_noop() {
        let (task, counter) = build_task_with_counter();
        let mut rx = IAgentTask::subscribe(task.as_ref());
        start_turn(task.as_ref()).await;

        let inst = AgentInstance::Session(Arc::clone(&task));
        inst.kill_and_wait(Some(AgentKillReason::IdleTimeout)).await;
        assert_eq!(
            counter.load(Ordering::SeqCst),
            0,
            "idle kill must NOT force backend terminate"
        );
        // The explicit `None` kill forwarder is likewise a Drop-driven no-op.
        assert!(IAgentTask::kill(task.as_ref(), None).is_ok());
        assert_eq!(
            counter.load(Ordering::SeqCst),
            0,
            "explicit None kill must NOT force backend terminate"
        );

        // Neither non-force-kill broadcast a terminal frame; status stays Running.
        let terminal = next_terminal(&mut rx).await;
        assert!(
            terminal.is_none(),
            "non-UserCancel kill must not broadcast Finish/Error, got {terminal:?}"
        );
        assert_eq!(IAgentTask::status(task.as_ref()), Some(ConversationStatus::Running));
    }
}
