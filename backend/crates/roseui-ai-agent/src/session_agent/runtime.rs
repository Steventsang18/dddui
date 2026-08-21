//! Runtime types: SessionRuntime, CatalogPreload, SessionPromptDump.
use super::*;

/// Resolve the reasoning-effort catalog to surface for the effort picker, mirroring the
/// backend's `effort_is_supported` current-model precedence: the efforts of the resolved
/// current model if it can be pinned, else the union across all advertised models (so we
/// don't hide a level some selectable model supports when the current model is ambiguous /
/// not-yet-known). Empty result = no effort axis → the caller omits the option entirely.
pub(crate) fn resolve_current_model_efforts(
    models: &[roseui_session::ModelInfo],
    current_model: Option<&str>,
) -> Vec<String> {
    if let Some(model) = current_model.and_then(|id| models.iter().find(|m| m.id == id)) {
        return model.reasoning_efforts.clone();
    }
    let mut union: Vec<String> = Vec::new();
    for m in models {
        for e in &m.reasoning_efforts {
            if !union.contains(e) {
                union.push(e.clone());
            }
        }
    }
    union
}

/// Shared, cheaply-cloneable runtime state for a session task: the broadcast sender
/// the translator writes and `subscribe()` reads, plus liveness bookkeeping.
pub(crate) struct SessionRuntime {
    pub(crate) tx: broadcast::Sender<AgentStreamEvent>,
    pub(crate) last_activity_ms: AtomicI64,
    /// Coarse status derived from the FSM edge the translator observes.
    pub(crate) status: std::sync::Mutex<Option<ConversationStatus>>,
    /// The CLI-assigned backend session id, learned from `BackendBound`. The ACP
    /// path stamps every Start/Finish with its session id; we mirror that so the
    /// frontend + resume-anchor consumer see the same id. `None` until the backend
    /// binds (first turn); a resume seeds it via the first BackendBound echo.
    pub(crate) session_id: std::sync::Mutex<Option<String>>,
    /// Optimistic mode/model selections set via `set_config_option`. The frontend's
    /// `hasObservedValue` contract requires set_config_option to return
    /// `confirmation: Observed` AND the option's `current_value == requested` — but
    /// claude's `capabilities()` does NOT reflect an in-band switch synchronously
    /// (set_model has NO confirmation wire at all; set_permission_mode confirms only
    /// asynchronously via a later `system/status`). So we cache the requested value
    /// here at dispatch time and have `get_config_options`/`mode`/`get_model` prefer
    /// it over the (stale) capabilities snapshot — the same optimistic-override the
    /// clean-slate runtime applies. Cleared/overwritten on the next switch.
    pub(crate) mode_override: std::sync::Mutex<Option<String>>,
    pub(crate) model_override: std::sync::Mutex<Option<String>>,
    /// Optimistic reasoning-effort ("thought level") selection, symmetric with
    /// mode/model. claude emits NO `ConfigChanged`/echo for effort (unlike model/mode),
    /// so the streaming catalog push — which runs in the backend-Arc-free event pump and
    /// cannot read `capabilities().current_effort` — reads the highlight from here. REST
    /// (`get_config_options`) prefers this over the (synchronously-seeded) caps value so
    /// the observed re-read confirms the switch. `None` until the user picks a level.
    pub(crate) effort_override: std::sync::Mutex<Option<String>>,
}

impl SessionRuntime {
    pub(crate) fn touch(&self) {
        self.last_activity_ms.store(now_ms(), Ordering::Relaxed);
    }
    pub(crate) fn set_status(&self, s: ConversationStatus) {
        if let Ok(mut g) = self.status.lock() {
            *g = Some(s);
        }
    }
    pub(crate) fn set_session_id(&self, id: String) {
        if let Ok(mut g) = self.session_id.lock() {
            *g = Some(id);
        }
    }
    pub(crate) fn session_id(&self) -> Option<String> {
        self.session_id.lock().ok().and_then(|g| g.clone())
    }
    pub(crate) fn set_mode_override(&self, mode: String) {
        if let Ok(mut g) = self.mode_override.lock() {
            *g = Some(mode);
        }
    }
    pub(crate) fn mode_override(&self) -> Option<String> {
        self.mode_override.lock().ok().and_then(|g| g.clone())
    }
    pub(crate) fn set_model_override(&self, model: String) {
        if let Ok(mut g) = self.model_override.lock() {
            *g = Some(model);
        }
    }
    pub(crate) fn model_override(&self) -> Option<String> {
        self.model_override.lock().ok().and_then(|g| g.clone())
    }
    pub(crate) fn set_effort_override(&self, effort: String) {
        if let Ok(mut g) = self.effort_override.lock() {
            *g = Some(effort);
        }
    }
    pub(crate) fn effort_override(&self) -> Option<String> {
        self.effort_override.lock().ok().and_then(|g| g.clone())
    }

    /// Atomic clean-converge frame: if not already `Finished`, set status ←
    /// `Finished` AND broadcast a clean `Finish` on `tx`. Idempotent in the
    /// `Finished` absorbing state (a repeat cancel / a late real Finish is a
    /// no-op — no second broadcast). This is the precise isomorph of the ACP
    /// path's `AgentRuntime::emit_finish`: it drives the SAME convergence chain
    /// (relay break → orchestrator releases the turn claim → `cancelling`
    /// cleared → `Idle`) so the gate recovers in seconds on the `UserCancel`
    /// force-kill path, WITHOUT waiting for the workflow to finish naturally.
    /// It is emitted for a `UserCancelTimeout` kill BEFORE the process is torn
    /// down, so the relay returns clean (a "cancelled", not a red crash card).
    pub(crate) fn emit_finish_once(&self) {
        let already = {
            let mut g = self.status.lock().unwrap_or_else(|e| e.into_inner());
            let was = matches!(*g, Some(ConversationStatus::Finished));
            if !was {
                *g = Some(ConversationStatus::Finished);
            }
            was
        };
        if already {
            return;
        }
        let _ = self.tx.send(AgentStreamEvent::Finish(FinishEventData {
            session_id: self.session_id(),
            ..Default::default()
        }));
    }
}

/// Cold-start catalog snapshot extracted from a persisted `agent_metadata`
/// handshake, in the SAME `roseui_session` shape the getters read off live
/// `capabilities()` — so serving the preload is a drop-in fallback with no shape
/// translation at read time. Empty vectors + `None` currents = nothing persisted.
#[derive(Default, Clone)]
pub(crate) struct CatalogPreload {
    pub(crate) available_models: Vec<roseui_session::ModelInfo>,
    pub(crate) current_model: Option<String>,
    pub(crate) available_modes: Vec<roseui_session::ModeInfo>,
    pub(crate) current_mode: Option<String>,
}

impl CatalogPreload {
    /// Parse the persisted handshake's `available_models` / `available_modes`
    /// columns into the live-capabilities shape. Reuses the ACP path's
    /// `extract_models_from_value` / `extract_modes_from_value` (the same
    /// multi-shape parser that accepts both the `{available_models:[{id,label}]}`
    /// column shape `spawn_catalog_writeback` persists AND a live-claude handshake),
    /// so the two paths stay byte-compatible. `reasoning_efforts` is intentionally
    /// dropped: the handshake catalog does not carry per-model efforts, and the
    /// getters this feeds do not surface efforts.
    pub(crate) fn from_handshake(handshake: &roseui_api_types::AgentHandshake) -> Self {
        use crate::manager::acp::config_option_catalog::{extract_models_from_value, extract_modes_from_value};
        let (available_models, current_model) = handshake
            .available_models
            .as_ref()
            .and_then(extract_models_from_value)
            .map(|state| {
                let models = state
                    .available_models
                    .iter()
                    .map(|m| roseui_session::ModelInfo {
                        id: m.model_id.to_string(),
                        name: m.name.clone(),
                        description: m.description.clone(),
                        reasoning_efforts: Vec::new(),
                    })
                    .collect::<Vec<_>>();
                let current = state.current_model_id.to_string();
                (models, (!current.is_empty()).then_some(current))
            })
            .unwrap_or_default();
        let (available_modes, current_mode) = handshake
            .available_modes
            .as_ref()
            .and_then(extract_modes_from_value)
            .map(|state| {
                let modes = state
                    .available_modes
                    .iter()
                    .map(|m| roseui_session::ModeInfo {
                        id: m.id.to_string(),
                        name: m.name.clone(),
                        description: m.description.clone(),
                    })
                    .collect::<Vec<_>>();
                let current = state.current_mode_id.to_string();
                (modes, (!current.is_empty()).then_some(current))
            })
            .unwrap_or_default();
        Self {
            available_models,
            current_model,
            available_modes,
            current_mode,
        }
    }
}

/// Resolved prompt-dump target for the direct-CLI (claude/codex) path.
///
/// Lifecycle: written once at task `build` time from the already-resolved
/// `<data_dir>/prompt-dumps` dir + the vendor label; read only in
/// `send_message`; never invalidated. `None` = `--dump-prompts` off (the
/// production default), in which case dumping is skipped with zero effect.
/// Carries `backend` because a `SessionBackend` exposes no vendor accessor and
/// both claude/codex present as `AgentType::Acp`, so the send-point dump could
/// not otherwise label the vendor.
#[derive(Debug, Clone)]
pub struct SessionPromptDump {
    pub(crate) dir: std::path::PathBuf,
    pub(crate) backend: &'static str,
}
/// Map the canonical multimodal block slice to raw JSON for a prompt dump.
/// Dev-only artifact: contents are kept RAW (image/audio base64 in full, text
/// verbatim) — the dump only runs under an explicit `--dump-prompts`.
pub(crate) fn session_content_blocks_to_json(content: &[ContentBlock]) -> Vec<serde_json::Value> {
    use base64::Engine as _;
    content
        .iter()
        .map(|b| match b {
            ContentBlock::Text(t) => serde_json::json!({ "type": "text", "text": t }),
            ContentBlock::Image { data, media_type } => serde_json::json!({
                "type": "image",
                "media_type": media_type,
                "data": base64::engine::general_purpose::STANDARD.encode(data),
            }),
            ContentBlock::Audio { data, media_type } => serde_json::json!({
                "type": "audio",
                "media_type": media_type,
                "data": base64::engine::general_purpose::STANDARD.encode(data),
            }),
            ContentBlock::ResourceLink { uri, mime_type } => serde_json::json!({
                "type": "resource_link",
                "uri": uri,
                "mime_type": mime_type,
            }),
            ContentBlock::AtMention { user_id } => serde_json::json!({
                "type": "at_mention",
                "user_id": user_id,
            }),
        })
        .collect()
}
