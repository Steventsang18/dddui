//! Core event pump: SessionEvent → AgentStreamEvent broadcast.
use super::*;

/// Discriminant name of a `SessionEvent`, for the pump's diagnostic debug log
/// (no payload — safe at debug; used to confirm which backend events actually
/// arrive when comparing the session path against the legacy ACP path).
pub(crate) fn session_event_name(e: &SessionEvent) -> &'static str {
    match e {
        SessionEvent::TurnStarted { .. } => "TurnStarted",
        SessionEvent::MessageDelta { .. } => "MessageDelta",
        SessionEvent::ThoughtDelta { .. } => "ThoughtDelta",
        SessionEvent::ToolCall { .. } => "ToolCall",
        SessionEvent::ToolResult { .. } => "ToolResult",
        SessionEvent::TurnResult { .. } => "TurnResult",
        SessionEvent::Detached { .. } => "Detached",
        SessionEvent::Permission { .. } => "Permission",
        SessionEvent::PermissionResolved { .. } => "PermissionResolved",
        SessionEvent::UsageDelta { .. } => "UsageDelta",
        SessionEvent::ConfigChanged { .. } => "ConfigChanged",
        SessionEvent::BackendBound { .. } => "BackendBound",
        SessionEvent::PromptAccepted { .. } => "PromptAccepted",
        SessionEvent::Snapshot { .. } => "Snapshot",
        other => {
            // Fallback for the many additive variants the pump drops; a leaked
            // debug string is fine (no payload).
            let s: &'static str = match other {
                SessionEvent::Plan { .. } => "Plan",
                SessionEvent::Rewound { .. } => "Rewound",
                SessionEvent::SubagentUpdate { .. } => "SubagentUpdate",
                SessionEvent::SubagentDetail { .. } => "SubagentDetail",
                SessionEvent::Notice { .. } => "Notice",
                SessionEvent::ToolOutputDelta { .. } => "ToolOutputDelta",
                SessionEvent::TurnDiffUpdated { .. } => "TurnDiffUpdated",
                SessionEvent::Provisioning { .. } => "Provisioning",
                _ => "Other",
            };
            s
        }
    }
}

/// Drain the backend's `events()` and re-broadcast each as an `AgentStreamEvent`.
pub(crate) fn spawn_event_pump(
    mut events: BoxStream<'static, SessionEnvelope>,
    runtime: Arc<SessionRuntime>,
    conversation_id: String,
    user_id: String,
    session_repo: Option<Arc<dyn IAcpSessionRepository>>,
) {
    use futures_util::StreamExt as _;
    // The pump owns ONLY the event stream (a broadcast `Receiver` handle — see
    // `ClaudeSessionBackend::events`), NEVER an `Arc<dyn SessionBackend>`. Holding a
    // backend Arc here would be self-referential: the backend pub(crate) struct owns the
    // `event_tx` this stream subscribes to, so a backend Arc in this task would keep
    // `event_tx` alive, the stream would never see `Closed`, this loop would never
    // exit, and the backend's `Drop` (the sole process reaper) would never run —
    // leaking the child CLI. By capturing only the stream, the sole long-lived
    // backend Arc is `SessionAgentTask.backend`; dropping the task (e.g. idle-kill
    // removing it from the manager map) drops that Arc → backend `Drop` → reader
    // abort + `kill_on_drop` → `event_tx` drops → this stream Closes → the loop ends.
    tokio::spawn(async move {
        // Per-tool accumulated live output for codex `ToolOutputDelta` (streamed
        // command stdout). The frontend merges `tool_call` frames by call_id with a
        // shallow REPLACE of `output` (hooks.ts: `{...existing, ...new}`), so we must
        // send the CUMULATIVE text each time, not the delta — otherwise each chunk
        // overwrites the last and only the final chunk shows. Keyed by item_id (==
        // the ToolCall tool_use_id). The authoritative full output still arrives on
        // the completed ToolResult, which harmlessly replaces this live view.
        let mut tool_output: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        // In-flight workflow/subagent refs, mirroring `state::background_active`
        // (any non-terminal roster entry ⇒ in-flight). claude's non-blocking
        // Workflow turn emits MULTIPLE `result` frames: the LAUNCH result arrives
        // while subagents are still running, and the TERMINAL result arrives only
        // AFTER every `task_notification{completed}` (fixture 2.1.176 invariant:
        // all completed precede all result). Forwarding the launch result's Finish
        // would terminate the relay and drop the workflow's completion message, so
        // we suppress the intermediate Finish until this set drains.
        let mut workflow_inflight: std::collections::HashSet<String> = std::collections::HashSet::new();
        // Remembered `tool_use_id` → tool name, learned from each `ToolCall` frame.
        // A tool's lifecycle emits SEVERAL frames sharing one call_id — the initial
        // ToolCall (name known), any codex `ToolOutputDelta` (name absent on the wire),
        // and the terminal `ToolResult` (the wire `tool_result` block carries only
        // tool_use_id, NOT the name). The frontend persists tool_call rows keyed by
        // call_id (stream_persistence::persist_tool_call, upsert), so a later frame with
        // an empty name would OVERWRITE the row's name to "" and the tool would render
        // nameless. Stamp the remembered name onto every follow-up frame so the name
        // survives — mirroring the reference `BackendOutputSink::emit_tool_result`,
        // which re-sends the name on completion. Cleared per turn with `tool_output`.
        let mut tool_name: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        // Tool calls of the CURRENT turn still awaiting their terminal `ToolResult`
        // (call_id → name). If the turn ends without one — user cancel, process
        // crash, or the CLI dropping the result — the persisted tool_call row would
        // stay status "work" FOREVER and the frontend View-Steps spinner
        // (`hasRunningToolMessages`) would never stop, surviving even reloads. The
        // terminal arm below closes every remaining entry with a `Canceled` frame
        // BEFORE the Finish (the relay stops forwarding a turn at Finish).
        let mut open_tools: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        // Did the CURRENT turn emit any user-visible output (text / thinking / tool /
        // plan / permission)? Mirrors the ACP path's `is_empty_turn` (agent_session_flow.rs):
        // a clean terminal with this still `false` is a "blank reply" (ELECTRON-1JG) and
        // gets a diagnostic Tip so the user isn't left staring at an empty bubble. Set as
        // events are observed, reset at the per-turn terminal (with `tool_output`/`tool_name`).
        let mut saw_visible_output = false;
        // Did the CURRENT turn already reach a terminal `TurnResult`? Mirrors the
        // reducer's `Running{terminal_result_seen}` flag. The pump consumes RAW
        // pre-reducer events, so to classify a `Detached` the way the reducer's
        // `crash_outcome` does — a crash mid-turn (no result yet) → Crashed, a
        // Detached AFTER the turn's result → absorbed (I10) — it must track this
        // itself. Without it, a Detached that trails a completed turn (idle-kill,
        // clean shutdown) would be misread as a mid-turn crash. Set on the terminal
        // TurnResult, reset on the next TurnStarted.
        let mut terminal_result_seen = false;
        while let Some(env) = events.next().await {
            runtime.touch();
            tracing::debug!(conv_id = %conversation_id, event = session_event_name(&env.event), "session-pump: backend event");

            // Empty-turn diagnostic Tip to emit for THIS terminal, if the turn was a
            // clean blank reply. Computed in the terminal match arm below (while
            // `saw_visible_output` still reflects this turn) and drained just before the
            // Finish in the translate loop — a Tips after Finish would be dropped, since
            // the relay breaks the turn on Finish. Per-iteration, so it never leaks
            // across turns.
            let mut pending_empty_turn_tip: Option<TipsEventData> = None;

            // ToolOutputDelta needs pump-local accumulation (see above), so it is
            // handled here rather than in the stateless translate_event.
            if let SessionEvent::ToolOutputDelta { item_id, text } = &env.event {
                // Streamed tool stdout is user-visible output — this turn is not blank.
                saw_visible_output = true;
                let acc = tool_output.entry(item_id.clone()).or_default();
                acc.push_str(text);
                let _ = runtime.tx.send(AgentStreamEvent::ToolCall(ToolCallEventData {
                    call_id: item_id.clone(),
                    // The wire delta carries no name; use the remembered one so this
                    // live-output frame doesn't overwrite the persisted row's name to "".
                    name: tool_name.get(item_id).cloned().unwrap_or_default(),
                    args: serde_json::Value::Null,
                    status: ToolCallStatus::Running,
                    input: None,
                    output: Some(acc.clone()),
                    description: None,
                }));
                continue;
            }

            // Async catalog discovery (claude `initialize` / codex `model/list` +
            // `collaborationMode/list` RESPONSE). Project it into an `AcpConfigOption`
            // frame — the direct-CLI analogue of the ACP path's `emit_snapshot_events`
            // catalog push. The frontend's `useAcpConfigOptions` handler REPLACES its
            // whole snapshot on this frame and re-derives the picker's `canSwitch`, so a
            // catalog that arrived ~6s after `open_session` (long after the frontend read
            // an empty `config_options`) finally lights the model/mode selector. Built
            // here, not in the stateless `translate_event`, because the current-value
            // highlight needs the runtime's optimistic overrides. Emitted whole (model +
            // mode categories together) so it never wipes a sibling category.
            if let SessionEvent::CatalogUpdated {
                models,
                modes,
                slash_commands,
            } = &env.event
            {
                let mut config_options: Vec<roseui_api_types::AcpConfigOptionDto> = Vec::new();
                if !modes.is_empty() {
                    config_options.push(roseui_api_types::AcpConfigOptionDto {
                        id: "mode".into(),
                        name: Some("Mode".into()),
                        label: None,
                        description: None,
                        category: Some("mode".into()),
                        option_type: "select".into(),
                        current_value: runtime.mode_override(),
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
                        current_value: runtime.model_override(),
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
                // Reasoning-effort axis (claude per-model `supportedEffortLevels`). The
                // frontend REPLACES its whole config-options snapshot on this frame, so we
                // MUST re-emit effort here too — otherwise a late catalog push would wipe
                // the effort option that `get_config_options` (REST) surfaced. The pump has
                // no backend Arc, so the current model is resolved from the pushed catalog
                // and the highlight comes from the runtime's optimistic effort override
                // (claude emits no effort echo). Emitted only when the current model
                // advertises efforts (union fallback when the current model is unknown).
                let efforts = resolve_current_model_efforts(models, runtime.model_override().as_deref());
                if !efforts.is_empty() {
                    config_options.push(roseui_api_types::AcpConfigOptionDto {
                        id: "reasoning_effort".into(),
                        name: Some("Thinking".into()),
                        label: None,
                        description: None,
                        category: Some("thought_level".into()),
                        option_type: "select".into(),
                        current_value: runtime.effort_override(),
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
                // No categories (both lists empty) → nothing to re-project; a spurious
                // empty-snapshot frame would only clobber the frontend's picker.
                if !config_options.is_empty()
                    && let Ok(v) = serde_json::to_value(serde_json::json!({ "config_options": config_options }))
                {
                    let _ = runtime.tx.send(AgentStreamEvent::AcpConfigOption(v));
                }
                // Slash-command catalog. claude advertises its command list in the
                // async `initialize` response — the same late-catalog timing that
                // strands the model/mode picker — and the frontend's mount-time REST
                // read (`fetchAcpSlashCommands`) returns empty before it lands. The
                // legacy ACP path recovers via a live `AvailableCommands` push
                // (translate.rs `AvailableCommandsUpdate` arm); this is its direct-CLI
                // analogue, so the `/` menu fills once discovery completes instead of
                // staying empty until a manual refetch.
                if !slash_commands.is_empty() {
                    let commands = slash_commands
                        .iter()
                        .map(|c| {
                            agent_client_protocol::schema::v1::AvailableCommand::new(
                                c.name.clone(),
                                c.description.clone().unwrap_or_default(),
                            )
                        })
                        .collect();
                    let _ = runtime
                        .tx
                        .send(AgentStreamEvent::AvailableCommands(AvailableCommandsEventData {
                            commands,
                        }));
                }
                continue;
            }

            // Track in-flight workflow/subagent refs so a non-blocking Workflow's
            // intermediate `result` frame does not prematurely terminate the turn.
            // Mirrors `state::background_active`: a ref is in-flight while its status
            // is non-terminal ({PendingInit, Running}); a terminal status
            // ({Interrupted, Completed, Errored, Shutdown}) removes it.
            if let SessionEvent::SubagentUpdate { r#ref, status, .. } = &env.event {
                use roseui_session::SubagentStatus;
                match status {
                    SubagentStatus::PendingInit | SubagentStatus::Running => {
                        workflow_inflight.insert(r#ref.clone());
                    }
                    SubagentStatus::Interrupted
                    | SubagentStatus::Completed
                    | SubagentStatus::Errored
                    | SubagentStatus::Shutdown => {
                        workflow_inflight.remove(r#ref);
                    }
                }
            }

            // Is THIS TurnResult an intermediate (workflow-launch) result whose Finish
            // must be suppressed? True only for a clean (non-error, non-cancel) result
            // that arrives while a workflow is still in flight. An error/cancel result
            // is always honoured as the terminal (the user must see it, and the
            // fixture invariant only covers clean completion ordering).
            let suppress_intermediate_finish = matches!(&env.event, SessionEvent::TurnResult { is_error, outcome, .. }
                if !workflow_inflight.is_empty()
                    && !*is_error
                    && !matches!(outcome, roseui_session::TurnOutcome::Cancelled { .. }));
            if suppress_intermediate_finish {
                tracing::info!(
                    conv_id = %conversation_id,
                    inflight = workflow_inflight.len(),
                    "session-pump: suppressing intermediate workflow-launch Finish (turn stays open until workflow completes)"
                );
            }

            // Drive the coarse status off the turn-boundary events so `status()`
            // reflects running/finished (the app gates the sidebar spinner on it).
            match &env.event {
                SessionEvent::TurnStarted { .. } => {
                    runtime.set_status(ConversationStatus::Running);
                    // New turn: the prior turn's terminal no longer applies.
                    terminal_result_seen = false;
                }
                // Track the call's open/closed lifecycle. Also remember the name for
                // `stamp_tool_name` — the map was previously never populated, so
                // ToolOutputDelta/ToolResult follow-up frames went out nameless.
                SessionEvent::ToolCall { tool_use_id, name, .. } => {
                    tool_name.insert(tool_use_id.clone(), name.clone());
                    open_tools.insert(tool_use_id.clone(), name.clone());
                }
                SessionEvent::ToolResult { tool_use_id, .. } => {
                    open_tools.remove(tool_use_id);
                }
                SessionEvent::TurnResult { .. } | SessionEvent::Detached { .. } if !suppress_intermediate_finish => {
                    runtime.set_status(ConversationStatus::Finished);
                    // Close every tool call the turn left open (cancel/crash/dropped
                    // result): emit a terminal `Canceled` frame per call so the
                    // persisted row leaves "work" and the frontend spinner stops.
                    // Must precede the Finish emitted by the translate loop below.
                    for (call_id, name) in open_tools.drain() {
                        tracing::info!(
                            conv_id = %conversation_id,
                            %call_id,
                            tool = %name,
                            "session-pump: closing tool call left open at turn end as canceled"
                        );
                        let _ = runtime.tx.send(AgentStreamEvent::ToolCall(ToolCallEventData {
                            call_id,
                            name,
                            args: serde_json::Value::Null,
                            status: ToolCallStatus::Canceled,
                            input: None,
                            output: None,
                            description: None,
                        }));
                    }
                    // A terminal TurnResult decided this turn; a later Detached is then
                    // an absorbed teardown, not a mid-turn crash (see `crash_outcome`).
                    if matches!(env.event, SessionEvent::TurnResult { .. }) {
                        terminal_result_seen = true;
                    }
                    // Empty-turn (blank-reply) diagnostic, mirroring the ACP path
                    // (agent_session_flow.rs `prompt_outcome_from_stop_reason`): a turn
                    // that reached a CLEAN terminal (`TurnResult{is_error:false}`, not
                    // cancelled) without emitting any user-visible output gets an
                    // informational/warning Tip so the user isn't left with an empty
                    // bubble. `Detached` (process crash) is excluded — that surfaces as a
                    // crash error elsewhere, not a "the model had nothing to say" tip, and
                    // ACP likewise only tips on a completed prompt. An error result is
                    // excluded because it already terminates as `AgentStreamEvent::Error`.
                    if let SessionEvent::TurnResult {
                        is_error: false,
                        outcome,
                        ..
                    } = &env.event
                        && !saw_visible_output
                    {
                        pending_empty_turn_tip = empty_turn_tip(outcome);
                    }
                    // Live tool-output accumulators are per-turn; the authoritative
                    // full output already rode each ToolResult. Drop them so a long
                    // session doesn't retain every turn's stdout.
                    tool_output.clear();
                    tool_name.clear();
                    // Reset the per-turn visibility flag for the next turn.
                    saw_visible_output = false;
                }
                // Learn the CLI-assigned session id so send_message (Start) and the
                // Finish stamping below carry it, matching the ACP path.
                SessionEvent::BackendBound {
                    backend_session_id: Some(bid),
                } => runtime.set_session_id(bid.clone()),
                _ => {}
            }
            // Persist the Tier-2 side-effects the legacy ACP path wrote via
            // AcpSessionSyncService (which this direct-CLI path bypasses). Best-effort:
            // a repo error is warn-logged, never fatal to the stream.
            if let Some(repo) = session_repo.as_ref() {
                persist_side_effects(repo.as_ref(), &user_id, &conversation_id, &env.event).await;
            }
            for mut ev in translate_event(env.event, &conversation_id, terminal_result_seen) {
                // Keep the tool name alive across a call's multi-frame lifecycle (see
                // `stamp_tool_name`): the terminal ToolResult frame leaves the name
                // empty, and the upsert-by-call_id persistence would otherwise clobber
                // the row's name to "". Runs before any routing decision below;
                // no-op on non-ToolCall frames (e.g. the suppressed Finish).
                stamp_tool_name(&mut tool_name, &mut ev);
                // Record whether this turn produced user-visible output, so a clean
                // terminal with none is detected as a blank reply (see the terminal
                // match arm above). Checked against the translated frame so the
                // definition matches the relay's own notion of visible output.
                if event_is_user_visible_output(&ev) {
                    saw_visible_output = true;
                }
                // Emit the empty-turn diagnostic Tip immediately BEFORE the Finish it
                // was computed for. It MUST precede Finish: the relay breaks the turn on
                // Finish (stream_relay.rs), so a Tips sent afterwards would never be
                // forwarded. `pending_empty_turn_tip` is only ever set on a clean
                // TurnResult, whose translation is exactly one Finish, so this fires once.
                if matches!(ev, AgentStreamEvent::Finish(_))
                    && let Some(tip) = pending_empty_turn_tip.take()
                {
                    let _ = runtime.tx.send(AgentStreamEvent::Tips(tip));
                }
                // Suppress the intermediate workflow-launch Finish: the assistant's
                // reply text already reached the frontend via MessageDelta→Text, so
                // dropping this Finish loses no output — it only keeps the relay open
                // so the workflow's later completion result can still be delivered.
                //
                // Emit a SegmentBreak in its place: the launch reply and the later
                // completion reply are two independent claude outputs, so the relay
                // must close the current text segment here. Otherwise both batches
                // accumulate under one msg_id and the frontend renders them as a
                // single bubble with no separator. SegmentBreak is consumed inside
                // the relay (never forwarded to the WS), so it changes only bubble
                // boundaries, not the wire contract.
                if suppress_intermediate_finish && matches!(ev, AgentStreamEvent::Finish(_)) {
                    let _ = runtime.tx.send(AgentStreamEvent::SegmentBreak);
                    continue;
                }
                // Stamp the CLI session id onto the Finish frame, matching the ACP path
                // which sends Finish{session_id}. The resume anchor rides it to the
                // frontend. (Start is emitted by send_message, already stamped.)
                //
                // KNOWN DIVERGENCE (accepted, additive gap): claude emits its per-turn
                // `UsageDelta` a few ms AFTER `TurnResult`, and origin's relay stops
                // forwarding a turn once it sees this Finish — so the trailing
                // AcpContextUsage frame does not reach the frontend and the context
                // indicator stays blank. The ACP path avoids this only because its SDK
                // blocks prompt() until usage is collected. Matching that needs an
                // end-of-turn "collect usage" barrier (or wiring get_context_usage) and
                // is deferred; the core turn flow is otherwise frame-equivalent.
                if let AgentStreamEvent::Finish(data) = &mut ev
                    && data.session_id.is_none()
                {
                    data.session_id = runtime.session_id();
                }
                // A send error only means no live subscribers — harmless.
                let _ = runtime.tx.send(ev);
            }
        }
    });
}
