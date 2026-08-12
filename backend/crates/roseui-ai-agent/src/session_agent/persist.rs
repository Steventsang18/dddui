//! Persistence: side-effect writes, dead-resume detection, permission options.
use super::*;

/// Pure decision (FCIS core): does this terminal `TurnResult` prove the stored
/// resume anchor is dead, so the next turn must open Fresh?
///
/// A resume against a backend session the CLI no longer knows fails with a
/// structural error ("No conversation found" / `error_during_execution`), NOT an
/// ordinary tool/turn error (those terminate `is_error:false` or with other text).
/// Classified through the SAME single-source predicate the clean-slate
/// `Orchestrator` uses (`roseui_session::is_unrecoverable_resume_error`), so a
/// backend wording change is fixed in one place. A user-cancelled turn is excluded:
/// claude reports an interrupt as `is_error` with cancel-noise text, but the anchor
/// is still good.
pub(crate) fn is_dead_resume_anchor(event: &SessionEvent) -> bool {
    use roseui_session::{TurnOutcome, is_unrecoverable_resume_error};
    let SessionEvent::TurnResult {
        is_error,
        result_text,
        outcome,
        ..
    } = event
    else {
        return false;
    };
    if !is_error || matches!(outcome, TurnOutcome::Cancelled { .. }) {
        return false;
    }
    let reason = roseui_session::ErrorReason::Backend {
        api_error_status: None,
        message: result_text.clone(),
    };
    is_unrecoverable_resume_error(&reason)
}

/// Persist the backend-observed session identity + config to `acp_session`, the
/// SAME writes the legacy `AcpSessionSyncService` domain-event consumer performed
/// for the ACP-manager path. Without this the resume anchor
/// (`build_session_instance` GAP #1) and the mode/model precedence source (GAP #2)
/// are never written, so a restart always loses continuity.
pub(crate) async fn persist_side_effects(
    repo: &dyn IAcpSessionRepository,
    user_id: &str,
    conversation_id: &str,
    event: &SessionEvent,
) {
    // Self-heal a dead resume anchor: a turn that failed *because* the stored
    // backend session id no longer resolves must null that id, or every subsequent
    // send re-resumes the same dead session and the conversation wedges forever.
    // Nulling (not deleting) keeps config/runtime state; the next open reads a
    // `None` anchor → Fresh → rebinds a live id. This restores the self-heal the
    // direct-CLI path dropped: clean-slate `Orchestrator` emits `BackendBound{None}`
    // and legacy ACP does `rebuild_after_session_not_found` → `clear_session_id`.
    if is_dead_resume_anchor(event) {
        match repo.clear_session_id_for_user(user_id, conversation_id).await {
            Ok(_) => tracing::info!(
                conversation_id,
                "session-sync: cleared dead resume anchor (unrecoverable resume error) — next turn opens Fresh"
            ),
            Err(err) => {
                tracing::warn!(conversation_id, error = %err, "session-sync: clear_session_id failed")
            }
        }
    }
    match event {
        // The CLI-echoed backend session id — written immediately (no debounce) so
        // the next turn takes the resume path even if the process crashes. `None`
        // (lost-backend self-heal) leaves the stored anchor as-is; a fresh rebind
        // happens on the next open.
        SessionEvent::BackendBound {
            backend_session_id: Some(bid),
        } => {
            if let Err(err) = repo.update_session_id_for_user(user_id, conversation_id, bid).await {
                tracing::warn!(conversation_id, error = %err, "session-sync: update_session_id failed");
            }
        }
        // A confirmed mode/model switch → persist so the next respawn/resume seeds
        // the user's selection (mirrors ObservedModeSynced / ObservedModelSynced).
        SessionEvent::ConfigChanged { mode, model } if mode.is_some() || model.is_some() => {
            let params = SaveRuntimeStateParams {
                current_mode_id: mode.as_ref().map(|m| Some(m.as_str())),
                current_model_id: model.as_ref().map(|m| Some(m.as_str())),
                config_selections_json: None,
                context_usage_json: None,
            };
            if let Err(err) = repo
                .save_runtime_state_for_user(user_id, conversation_id, &params)
                .await
            {
                tracing::warn!(conversation_id, error = %err, "session-sync: save_runtime_state failed");
            }
        }
        _ => {}
    }
}

/// Extract the picked option id from the confirm `data` payload. The frontend sends
/// either a bare string (the option_id) or an object `{option_id|optionId|value}`.
/// Mirrors the ACP path's `confirm_option_id`.
pub(crate) fn confirm_option_id(data: &serde_json::Value) -> Option<String> {
    match data {
        serde_json::Value::String(v) => Some(v.clone()),
        serde_json::Value::Object(map) => map
            .get("option_id")
            .or_else(|| map.get("optionId"))
            .or_else(|| map.get("value"))
            .and_then(|v| v.as_str())
            .map(ToOwned::to_owned),
        _ => None,
    }
}

/// Generic allow / allow-always / reject options for an ordinary tool-approval
/// permission card. `confirm()` maps these option ids back to a `PermissionDecision`.
pub(crate) fn default_permission_options() -> Vec<crate::protocol::events::AcpPermissionOptionData> {
    use crate::protocol::events::{AcpPermissionOptionData, AcpPermissionOptionKind};
    vec![
        AcpPermissionOptionData {
            option_id: PERM_ALLOW.to_owned(),
            name: "Allow".to_owned(),
            kind: AcpPermissionOptionKind::AllowOnce,
            meta: None,
        },
        AcpPermissionOptionData {
            option_id: PERM_ALLOW_ALWAYS.to_owned(),
            name: "Allow Always".to_owned(),
            kind: AcpPermissionOptionKind::AllowAlways,
            meta: None,
        },
        AcpPermissionOptionData {
            option_id: PERM_REJECT.to_owned(),
            name: "Reject".to_owned(),
            kind: AcpPermissionOptionKind::RejectOnce,
            meta: None,
        },
    ]
}

/// Project an AskUserQuestion tool `input` into permission-card options the user can
/// pick. `input` shape (claude, live-captured): `{questions:[{question, header,
/// options:[{label, description}], multiSelect}]}`. The frontend card is single-select
/// (one radio group, one confirm), so we surface the FIRST question's option labels as
/// the choices — `option_id == label` so `confirm()` can pass the picked label straight
/// into `AnswerPermission.selected` (claude keys the answer by label). A multi-question
/// AskUserQuestion degrades to answering the first question (a known single-select
/// frontend limitation — the remaining questions claude silently drops, same as the
/// legacy single-question path). Returns empty when the shape is absent/unparseable, so
/// the caller falls back to allow/deny.
pub(crate) fn ask_user_question_options(
    input: Option<&serde_json::Value>,
) -> Vec<crate::protocol::events::AcpPermissionOptionData> {
    use crate::protocol::events::{AcpPermissionOptionData, AcpPermissionOptionKind};
    let Some(first_q) = input
        .and_then(|i| i.get("questions"))
        .and_then(|q| q.as_array())
        .and_then(|arr| arr.first())
    else {
        return Vec::new();
    };
    let Some(opts) = first_q.get("options").and_then(|o| o.as_array()) else {
        return Vec::new();
    };
    opts.iter()
        .filter_map(|o| o.get("label").and_then(|l| l.as_str()))
        .map(|label| AcpPermissionOptionData {
            // option_id == label: confirm() forwards it as the chosen answer label.
            option_id: label.to_owned(),
            name: label.to_owned(),
            kind: AcpPermissionOptionKind::AllowOnce,
            meta: None,
        })
        .collect()
}
