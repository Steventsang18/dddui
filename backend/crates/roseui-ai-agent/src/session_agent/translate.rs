//! Event translation: SessionEvent → AgentStreamEvent with tool stamping.
use super::*;

/// Keep a tool's name alive across the multiple `AgentStreamEvent::ToolCall` frames
/// that share one `call_id` over its lifecycle.
///
/// A single tool call surfaces as several frames keyed by the same `call_id`: the
/// initial `ToolCall` (status Running, name known); any codex `ToolOutputDelta`
/// (streamed stdout, name absent on the wire); and the terminal `ToolResult` (the
/// wire `tool_result` block carries only `tool_use_id`, never the name — so
/// `translate_event` leaves it empty). The frontend persists tool_call rows by
/// upsert on `call_id` (`stream_persistence::persist_tool_call`), so a later
/// empty-name frame would OVERWRITE the row's name to `""` and the tool would render
/// nameless.
///
/// This learns the name from the first frame that carries one and stamps it back
/// onto any later empty-name frame for the same `call_id`, mirroring the reference
/// `BackendOutputSink::emit_tool_result`, which re-sends the name on completion.
/// `names` is the pump-local map (cleared per turn); non-`ToolCall` events are inert.
pub(crate) fn stamp_tool_name(names: &mut std::collections::HashMap<String, String>, ev: &mut AgentStreamEvent) {
    let AgentStreamEvent::ToolCall(data) = ev else {
        return;
    };
    if data.name.is_empty() {
        if let Some(known) = names.get(&data.call_id) {
            data.name = known.clone();
        }
    } else {
        names.insert(data.call_id.clone(), data.name.clone());
    }
}

/// Translate one clean-slate `SessionEvent` into zero or more origin
/// `AgentStreamEvent`s. The fold SHAPE mirrors the clean-slate TurnFinalizer, but
/// the output targets origin's `AgentStreamEvent` enum instead of `ConvDomainEvent`.
/// Whether a translated stream event represents user-visible turn output —
/// anything that renders in chat. Mirrors the ACP path's
/// `event_is_user_visible_output` (agent_session_flow.rs) so the direct-CLI
/// empty-turn detection uses the same definition of "the turn said something".
pub(crate) fn event_is_user_visible_output(event: &AgentStreamEvent) -> bool {
    matches!(
        event,
        AgentStreamEvent::Text(_)
            | AgentStreamEvent::Thinking(_)
            | AgentStreamEvent::ToolCall(_)
            | AgentStreamEvent::AcpToolCall(_)
            | AgentStreamEvent::ToolGroup(_)
            | AgentStreamEvent::Plan(_)
            | AgentStreamEvent::Permission(_)
            | AgentStreamEvent::AcpPermission(_)
    )
}

/// Build the empty-turn diagnostic Tip for a clean terminal that produced no
/// user-visible output, mirroring the ACP path (agent_session_flow.rs:388-448):
/// a normal `EndTurn` is an informational "no reply" note; any other stop reason
/// (truncation / refusal / failure) is a warning naming the cause. Codes match
/// the `conversation.agentTip.codes.*` i18n keys the frontend `MessageTips`
/// renderer localizes. Cancelled is `None` (never a blank-reply; the caller also
/// guards it) so a user interrupt never surfaces a spurious tip.
pub(crate) fn empty_turn_tip(outcome: &roseui_session::TurnOutcome) -> Option<TipsEventData> {
    use roseui_session::{StopReason, TruncationKind, TurnOutcome};
    let (tip_type, code) = match outcome {
        TurnOutcome::EndTurn
        | TurnOutcome::Completed {
            stop_reason: StopReason::EndTurn,
        } => (TipType::Info, "ACP_EMPTY_TURN"),
        TurnOutcome::Completed {
            stop_reason: StopReason::Truncated(TruncationKind::MaxTokens),
        } => (TipType::Warning, "ACP_EMPTY_TURN_MAX_TOKENS"),
        TurnOutcome::Completed {
            stop_reason: StopReason::Truncated(TruncationKind::MaxTurns),
        } => (TipType::Warning, "ACP_EMPTY_TURN_MAX_TURN_REQUESTS"),
        TurnOutcome::Completed {
            stop_reason: StopReason::Refused { .. },
        } => (TipType::Warning, "ACP_EMPTY_TURN_REFUSAL"),
        // Other truncation kinds (context window / budget / bare wire-end) and a
        // clean `Failed` have no dedicated ACP code — surface the generic warning
        // so the user still sees "the turn ended without a reply" with a hint.
        TurnOutcome::Completed { .. } | TurnOutcome::Failed => (TipType::Warning, "ACP_EMPTY_TURN"),
        TurnOutcome::Cancelled { .. } => return None,
    };
    Some(TipsEventData {
        content: String::new(),
        tip_type,
        code: Some(code.to_owned()),
        params: None,
    })
}

/// `terminal_result_seen`: did the current turn already reach a terminal
/// `TurnResult`? Only consulted for `Detached` — it lets this stateless pub(crate) fn /// replicate the reducer's `crash_outcome` guard (a Detached AFTER the turn's
/// result is an absorbed teardown, not a crash). Immaterial for every other arm.
pub(crate) fn translate_event(
    event: SessionEvent,
    conversation_id: &str,
    terminal_result_seen: bool,
) -> Vec<AgentStreamEvent> {
    match event {
        // NOTE: the Start lifecycle frame is emitted by `send_message` (before
        // dispatch), mirroring the ACP path which emits Start right before prompt().
        // The backend's own turn-start signals — claude/codex `PromptAccepted`
        // (arrives AFTER the first text delta) and the orchestrator-lowered
        // `TurnStarted` (never reaches this stream) — are therefore NOT re-projected
        // to Start here, or the frontend would see a late/duplicate turn boundary.
        SessionEvent::PromptAccepted { .. } | SessionEvent::TurnStarted { .. } => Vec::new(),
        SessionEvent::MessageDelta { text, .. } => {
            vec![AgentStreamEvent::Text(TextEventData { content: text })]
        }
        SessionEvent::ThoughtDelta { text, .. } => {
            vec![AgentStreamEvent::Thinking(ThinkingEventData {
                content: text,
                subject: None,
                duration: None,
                status: Some("thinking".into()),
            })]
        }
        SessionEvent::ToolCall {
            tool_use_id,
            name,
            input,
            ..
        } => {
            vec![AgentStreamEvent::ToolCall(ToolCallEventData {
                call_id: tool_use_id,
                name,
                args: input.clone(),
                status: ToolCallStatus::Running,
                input: Some(input),
                output: None,
                description: None,
            })]
        }
        SessionEvent::ToolResult {
            tool_use_id,
            is_error,
            content,
            ..
        } => {
            let output = tool_result_text(&content);
            vec![AgentStreamEvent::ToolCall(ToolCallEventData {
                call_id: tool_use_id,
                name: String::new(),
                args: serde_json::Value::Null,
                status: if is_error {
                    ToolCallStatus::Error
                } else {
                    ToolCallStatus::Completed
                },
                input: None,
                output,
                description: None,
            })]
        }
        SessionEvent::TurnResult {
            is_error,
            result_text,
            outcome,
            ..
        } => {
            // A user-cancelled turn is NOT an error: claude reports its interrupt as an
            // is_error result (e.g. `error_during_execution` / an aborted-tool
            // diagnostic), but the user asked for it — so a cancel ends with a plain
            // Finish, no error (the origin frontend lacks the clean-slate cancel-noise
            // suppression, so we suppress at the source).
            let is_cancel = matches!(outcome, roseui_session::TurnOutcome::Cancelled { .. });
            if is_error && !is_cancel && !result_text.trim().is_empty() {
                // A genuine turn error terminates as AgentStreamEvent::Error carrying the
                // FULL origin error model (code / ownership / retryable /
                // feedback_recommended), NOT a plain Tips. The relay reads
                // Error{code,retryable} to drive auto-replay + error classification
                // (stream_relay::terminal_from_event) and the frontend renders ownership/
                // feedback from it; a Tips carries none of that and is not even seen as a
                // terminal. Classify the result text through the SAME path the ACP empty-
                // turn error uses (AgentError::bad_gateway → classify_upstream_detail), so
                // provider/billing/rate-limit/lifecycle errors are categorized identically.
                // Error IS the terminal (relay breaks on it), so we do NOT also emit Finish.
                let stream_error =
                    AgentSendError::from_agent_error(AgentError::bad_gateway(result_text)).into_stream_error();
                return vec![AgentStreamEvent::Error(stream_error)];
            }
            vec![AgentStreamEvent::Finish(FinishEventData::default())]
        }
        SessionEvent::Detached { exit, redacted_summary } => {
            // A process exit is only an ERROR when it is a genuine mid-turn crash.
            // Classify it EXACTLY as the clean-slate reducer's `crash_outcome` does
            // (the pump consumes raw pre-reducer events, so it must replicate that
            // pure pub(crate) fn rather than inherit its verdict):
            //   - terminal result already seen this turn → absorbed teardown (I10);
            //   - clean exit-0, no result → EmptyTurn-class (a blank turn, not a
            //     crash) — the empty-turn Tip already rode the status match above;
            //   - signal / non-zero / unknown(None) exit, no result → CRASH.
            // Only the crash case surfaces as an error; the rest end with a plain
            // Finish (behaviour-preserving). This restores the legacy ACP path's
            // `AcpError::Disconnected → UserAgentDisconnected` terminal that the
            // direct-CLI bridge previously dropped: a CLI that dies mid-reply used
            // to render as a normal (empty) completion instead of a "disconnected,
            // reconnect" error card.
            match roseui_session::crash_outcome(terminal_result_seen, exit) {
                roseui_session::Outcome::Crashed => {
                    // Route through the SAME classifier legacy used, so the frontend
                    // gets the identical code/ownership/retryable/resolution. The
                    // allowlisted `redacted_summary` (already stripped of secrets at
                    // the backend boundary) rides the error message as the user-facing
                    // reason — mirroring `CloseReason::ProcessExited: {summary}`;
                    // without it the card shows only a bare exit code.
                    let acp_err = crate::protocol::error::AcpError::Disconnected {
                        exit_code: exit.and_then(|e| e.code),
                        signal: exit.and_then(|e| e.signal).map(|s| s.to_string()),
                        stderr: redacted_summary.clone().unwrap_or_default(),
                    };
                    let mut stream_error =
                        AgentSendError::from_agent_error(AgentError::Acp(acp_err)).into_stream_error();
                    if let Some(summary) = redacted_summary.filter(|s| !s.trim().is_empty()) {
                        stream_error.message = format!("{}: {summary}", stream_error.message);
                    }
                    vec![AgentStreamEvent::Error(stream_error)]
                }
                roseui_session::Outcome::CleanNoResult | roseui_session::Outcome::FollowResult => {
                    vec![AgentStreamEvent::Finish(FinishEventData::default())]
                }
            }
        }
        // Interactive tool approval: surface as an AcpPermission Request so the
        // frontend renders the allow/deny card. The `tool_call_id` MUST equal the
        // `request_id` — `SessionAgentTask::confirm` dispatches `AnswerPermission`
        // keyed on the same id (the frontend echoes the `call_id` it received here).
        // `input` (the raised tool's raw input — a Bash `command`, AskUserQuestion
        // `{questions:[…]}`) rides as `raw_input` so the card can show the approver
        // what they are approving (RoseUi issue #3779); the generic
        // `Approved`/`Denied` options let the reducer + card render.
        SessionEvent::Permission {
            request_id,
            tool_name,
            input,
            ..
        } => {
            // The frontend permission card renders whatever `options[]` we send as the
            // selectable choices (MessageAcpPermission maps each to a radio). So the
            // options MUST reflect what the user is actually choosing between:
            //   - AskUserQuestion → the question's own options (labels), so the user
            //     answers the question. `confirm()` maps the picked label to the
            //     AnswerPermission `selected` (claude keys the answer by it).
            //   - any other tool approval → generic Allow / Allow Always / Reject.
            // (Before, EVERY permission — including AskUserQuestion — was hard-coded to
            // allow/deny, so a question rendered as an allow/deny card. TIO: the question
            // content in `input` is user-facing, not a sensitive tool body.)
            let is_ask = tool_name.as_deref() == Some("AskUserQuestion");
            let options = if is_ask {
                ask_user_question_options(input.as_ref())
            } else {
                Vec::new()
            };
            let options = if options.is_empty() {
                default_permission_options()
            } else {
                options
            };
            vec![AgentStreamEvent::AcpPermission(
                crate::protocol::events::AcpPermissionEventData::Request(
                    crate::protocol::events::AcpPermissionRequestData {
                        session_id: conversation_id.to_owned(),
                        tool_call: crate::protocol::events::AcpPermissionToolCall {
                            tool_call_id: request_id,
                            status: None,
                            title: tool_name,
                            kind: None,
                            raw_input: input,
                            raw_output: None,
                            content: None,
                            locations: None,
                            meta: None,
                        },
                        options,
                        meta: None,
                    },
                ),
            )]
        }
        // Per-turn usage/cost → the AcpContextUsage passthrough frame the frontend
        // usage indicator reads (shape: cumulative token counters).
        SessionEvent::UsageDelta {
            input_tokens,
            output_tokens,
            total_tokens,
            cost_usd,
        } => {
            // The frontend ContextUsageIndicator reads `used` (tokens consumed) and,
            // optionally, `size` (context window) + `cost` — the exact shape the ACP
            // path forwards (the claude-agent-acp SDK's UsageUpdate: {used, size,
            // cost:{amount,currency}}). Emitting the raw {input_tokens,…} shape left
            // the indicator blank (no `used` key). `size` is omitted: UsageDelta
            // carries no context-window figure (that rides the separate
            // get_context_usage control probe, not wired here), and the frontend
            // guards `if size>0` so its absence is safe. `used` = total_tokens (the
            // genuine cumulative total the adapter already computed, incl. cache).
            let mut usage = serde_json::json!({ "used": total_tokens });
            if let Some(cost) = cost_usd {
                usage["cost"] = serde_json::json!({ "amount": cost, "currency": "USD" });
            }
            // Keep the raw counters too (harmless extra keys) for any richer consumer.
            usage["input_tokens"] = serde_json::json!(input_tokens);
            usage["output_tokens"] = serde_json::json!(output_tokens);
            vec![AgentStreamEvent::AcpContextUsage(usage)]
        }
        // A confirmed mode/model switch is NOT forwarded as a stream frame. The origin
        // frontend's mode/model pickers (AgentModeSelector / AcpModelSelector) track the
        // selection in local state updated optimistically on the PUT /config-options
        // call + its REST response — they do NOT consume a config stream frame. And the
        // origin `useAcpMessage` has no `acp_config_option` case, so any such frame falls
        // into its `default:` arm and lights the turn timer bar (`setRunning(true)`) —
        // the "switching mode shows a spurious timer" regression. So emit nothing here;
        // the selection persist is handled separately by `persist_side_effects`.
        SessionEvent::ConfigChanged { .. } => Vec::new(),
        // Handled earlier in the pump (needs runtime overrides for the current-value
        // highlight; projected to an AcpConfigOption frame there). Never reaches this
        // stateless translator, but the match is total so give it an explicit no-op arm.
        SessionEvent::CatalogUpdated { .. } => Vec::new(),
        // Live plan / to-do snapshot (codex `turn/plan/updated`; claude never emits it).
        // origin has `AgentStreamEvent::Plan` + a `MessagePlan` renderer that reads
        // `entries[].content` + `entries[].status` where status is snake_case
        // (`pending`/`in_progress`/`completed`). Our `PlanStatus` serializes PascalCase,
        // so map it to the frontend contract explicitly rather than serde-dumping the
        // pub(crate) struct (a raw dump would send `Completed` and the card would never tick).
        SessionEvent::Plan { entries, .. } => {
            let entries: Vec<serde_json::Value> = entries
                .iter()
                .map(|e| {
                    let status = match e.status {
                        roseui_session::PlanStatus::Pending => "pending",
                        roseui_session::PlanStatus::InProgress => "in_progress",
                        roseui_session::PlanStatus::Completed => "completed",
                    };
                    serde_json::json!({ "content": e.content, "status": status })
                })
                .collect();
            vec![AgentStreamEvent::Plan(
                crate::protocol::events::session_updates::PlanEventData {
                    session_id: None,
                    entries,
                },
            )]
        }
        // Out-of-turn advisory (codex `warning`/`guardianWarning`/`configWarning`/
        // `deprecationNotice`; claude a rejected mode/model/effort set surfaced by
        // `sniff_set_config_reject`). Both backends emit `Notice` *specifically so a
        // failed/advisory event is VISIBLE instead of silently dropped* — re-dropping it
        // here would re-introduce exactly the silent-degradation the backends were coded
        // to avoid (e.g. a rejected effort switch would look like it succeeded). Surface
        // it as a `Tips` frame — the one advisory frame the origin frontend already
        // renders (`MessageTips`, warning/info styling). NOTE: origin's `useAcpMessage`
        // has no explicit `tips` case, so a `tips` frame lands in its `default:` arm,
        // which is benign for display (it renders via `mergeLiveMessage`) but also calls
        // `setRunning(true)`. That is acceptable here: a Notice only arrives mid/around a
        // turn that is already running (or immediately re-settled by the turn's terminal
        // Finish), so it does not manufacture a spurious idle timer the way a config frame
        // would. `NoticeLevel` has only Info/Warning (no Error tier), matching TipType.
        SessionEvent::Notice { level, message } => {
            let tip_type = match level {
                roseui_session::NoticeLevel::Info => TipType::Info,
                roseui_session::NoticeLevel::Warning => TipType::Warning,
            };
            vec![AgentStreamEvent::Tips(TipsEventData {
                content: message,
                tip_type,
                code: None,
                params: None,
            })]
        }
        // Events with no origin-side counterpart (or purely internal) are dropped.
        // Cancel folds into the Finish emitted by the resulting terminal; Heartbeat,
        // PromptAccepted, Snapshot, Lagged, item lifecycle, subagent/rewound/etc. are
        // not part of origin's AgentStreamEvent vocabulary. codex ToolOutputDelta /
        // TurnDiffUpdated / SubagentUpdate are also dropped for now — separate
        // follow-ups (each needs its own origin frame + renderer verification).
        _ => Vec::new(),
    }
}

/// Flatten a tool result's content parts into a single text string for the
/// `ToolCallEventData.output` field (origin renders that).
pub(crate) fn tool_result_text(content: &[ToolResultContent]) -> Option<String> {
    let mut buf = String::new();
    for part in content {
        if let ToolResultContent::Text(t) = part {
            if !buf.is_empty() {
                buf.push('\n');
            }
            buf.push_str(t);
        }
    }
    if buf.is_empty() { None } else { Some(buf) }
}
