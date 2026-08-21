//! Catalog writeback + codex sandbox/approval helpers.
use super::*;

/// (claude/codex) backend. The ACP catalog (modes/models/commands) lands a beat
/// AFTER `open_session` returns (the session/new|load response is parsed
/// asynchronously by the reader), so this waits for a discovery (bounded to ~5s),
/// then forwards the projected partial via the registry's `CatalogSender`
/// (best-effort — re-discovery on the next open is the idempotent fallback). Off
/// the open hot path. Without this the `/api/agents` model/mode picker never
/// refreshes for claude/codex sessions (the exact "codex 无法选择模型" regression).
///
/// Verbatim port of clean-slate `session_runtime::spawn_catalog_writeback`: wait
/// for MODELS specifically before committing (codex answers modes before models),
/// forwarding the best model-less partial only if the window elapses.
pub fn spawn_catalog_writeback(
    agent_id: String,
    user_id: String,
    backend: Arc<dyn roseui_session::SessionBackend>,
    catalog_tx: crate::registry::CatalogSender,
) {
    tokio::spawn(async move {
        let mut best_partial = None;
        for _ in 0..100 {
            let caps = backend.capabilities();
            if let Some(partial) = catalog_partial_from_caps(&caps) {
                if !caps.available_models.is_empty() {
                    // Complete enough — models present → commit the full catalog.
                    catalog_tx.send_partial(user_id.clone(), agent_id, partial);
                    return;
                }
                // Modes/commands only so far — remember it, keep waiting for models.
                best_partial = Some(partial);
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        if let Some(partial) = best_partial {
            catalog_tx.send_partial(user_id, agent_id, partial);
        }
    });
}

/// Project a backend's discovered `Capabilities` (modes / models / slash commands)
/// into an `AgentHandshake` partial for the `agent_metadata` catalog. Verbatim port
/// of clean-slate `session_runtime::catalog_partial_from_caps`: emits both the ACP
/// `config_options[]` wire shape AND the top-level `available_modes`/`available_models`
/// columns directly (the shape-stable path that keeps the codex model picker from
/// going empty).
pub(crate) fn catalog_partial_from_caps(
    caps: &roseui_session::Capabilities,
) -> Option<roseui_api_types::AgentHandshake> {
    let mut config_options = Vec::new();
    if !caps.available_modes.is_empty() {
        config_options.push(serde_json::json!({
            "id": "mode",
            "category": "mode",
            "type": "select",
            "currentValue": caps.current_mode,
            "options": caps.available_modes.iter().map(|m| serde_json::json!({
                "value": m.id, "name": m.name, "description": m.description,
            })).collect::<Vec<_>>(),
        }));
    }
    if !caps.available_models.is_empty() {
        config_options.push(serde_json::json!({
            "id": "model",
            "category": "model",
            "type": "select",
            "currentValue": caps.current_model,
            "options": caps.available_models.iter().map(|m| serde_json::json!({
                "value": m.id, "name": m.name, "description": m.description,
            })).collect::<Vec<_>>(),
        }));
    }
    let available_commands = if caps.slash_commands.is_empty() {
        None
    } else {
        Some(serde_json::json!(
            caps.slash_commands
                .iter()
                .map(|c| serde_json::json!({
                    "name": c.name, "description": c.description,
                }))
                .collect::<Vec<_>>()
        ))
    };
    if config_options.is_empty() && available_commands.is_none() {
        return None;
    }
    let config_options = if config_options.is_empty() {
        None
    } else {
        Some(serde_json::Value::Array(config_options))
    };
    // Also project the top-level `available_modes`/`available_models` fields directly
    // (shape: `{available_models:[{id,label}]}`), which `apply_handshake` persists to
    // the catalog columns VERBATIM — the authoritative, shape-stable path (matches what
    // a live claude handshake stores), so the codex model picker never goes empty.
    let available_modes = (!caps.available_modes.is_empty()).then(|| {
        serde_json::json!({
            "available_modes": caps.available_modes.iter().map(|m| serde_json::json!({
                "id": m.id, "name": m.name, "description": m.description,
            })).collect::<Vec<_>>(),
            "current_mode_id": caps.current_mode,
        })
    });
    let available_models = (!caps.available_models.is_empty()).then(|| {
        serde_json::json!({
            "available_models": caps.available_models.iter().map(|m| serde_json::json!({
                "id": m.id, "label": m.name,
            })).collect::<Vec<_>>(),
            "current_model_id": caps.current_model,
        })
    });
    Some(roseui_api_types::AgentHandshake {
        config_options,
        available_modes,
        available_models,
        available_commands,
        ..Default::default()
    })
}

/// Map a conversation's requested mode → the codex `thread/start.sandbox` string
/// (`SandboxMode`: `read-only` / `workspace-write` / `danger-full-access`, verified
/// `codex-cli/0.137.0/schema-full/ClientRequest.json` §SandboxMode), or `None` to keep
/// the backend's safe default (`unwrap_or("workspace-write")`).
///
/// This runs at OPEN time and pre-seeds `thread/start.sandbox` — the sandbox axis the
/// tier reaches the FIRST turn through, since `thread/start` carries no `permissions`
/// field and `permissions` is mutually exclusive with `sandbox` (U1). The post-open
/// `reconcile_codex_mode` only applies the matching permission profile via SetMode, and
/// that `thread/settings/update{permissions}` write "applies to the NEXT turn"
/// (codex_conn `Command::SetMode`) — so WITHOUT seeding the restrictive sandbox here, a
/// read-only conversation's first turn would run under the permissive `workspace-write`
/// default and a write would succeed before the profile lands. We therefore seed BOTH
/// escalation (`full-access` → `danger-full-access`) AND restriction (`read-only` →
/// `read-only`) at the sandbox axis; the middle `workspace`/`auto` tier keeps the
/// `workspace-write` default (returned as `None`).
///
/// The mode value reaching this boot helper is the persisted/config selection, which
/// under feature 012 "Plan B" is the LEGACY bare token (`full-access` / `read-only`);
/// the colon profile id (`:danger-full-access` / `:read-only`, e.g. from a readback that
/// skipped bare-mapping) and the legacy `yoloNoSandbox` alias stay recognized for
/// robustness. Kept in lockstep with `codex_conn::codex_perm::{normalize_to_profile_id,
/// profile_id_to_legacy_value}`.
pub(crate) fn codex_sandbox_for_mode(mode: Option<&str>) -> Option<&'static str> {
    match mode.map(str::trim) {
        // `agent-full-access` is the canonical codex full-access mode id since #608
        // (migration 021 rewrote builtin `yolo_id` `full-access`→`agent-full-access`, and
        // `normalize_requested_mode` now resolves yolo aliases to it). The legacy
        // `full-access` / `:danger-full-access` / `yoloNoSandbox` stay recognized for
        // pre-021 persisted data — all four are the same danger-full-access tier.
        Some(":danger-full-access" | "agent-full-access" | "full-access" | "yoloNoSandbox") => {
            Some("danger-full-access")
        }
        Some(":read-only" | "read-only") => Some("read-only"),
        _ => None,
    }
}

/// Map a conversation's requested mode → the codex `approvalPolicy` string, or
/// `None` to keep the default (`on-request`). Sibling of `codex_sandbox_for_mode`:
/// a full-access / yolo agent runs unattended → `"never"`. Recognizes the legacy bare
/// token `full-access` (the Plan B canonical value), the colon id `:danger-full-access`,
/// and the legacy `yoloNoSandbox` alias. Verbatim port of clean-slate
/// `session_runtime::codex_approval_for_mode`.
pub(crate) fn codex_approval_for_mode(mode: Option<&str>) -> Option<&'static str> {
    match mode.map(str::trim) {
        // Recognizes the #608 canonical `agent-full-access` alongside the legacy
        // `full-access` / `:danger-full-access` / `yoloNoSandbox` (see codex_sandbox_for_mode).
        Some(":danger-full-access" | "agent-full-access" | "full-access" | "yoloNoSandbox") => Some("never"),
        _ => None,
    }
}
