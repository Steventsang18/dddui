//! Translation layer between the Rupoo engine's `AgentEvent` stream and
//! DoDidDoneUi's `AgentStreamEvent` protocol.
//!
//! The engine crate (`rupoo`) emits a small, flat event set
//! (`TextDelta` / `ToolCall` / `ToolResult`). DoDidDoneUi's wire protocol
//! (`AgentStreamEvent`) is richer and is what the web frontend renders.
//! This module is the single place that maps one onto the other so the
//! manager stays free of wire-format concerns.
//!
//! ## Approval-tool handling
//!
//! When an interactive approval gate is installed, high-risk tools pause for
//! user confirmation *before* execution. In that mode the host surfaces an
//! `AcpPermission` confirmation card (driven by the gate's notice channel),
//! so the translator must NOT also emit a `ToolCall`/`Running` bubble for those
//! tools — that would double-represent the same call. Likewise, when the tool
//! finally runs and `ToolResult` arrives, the translator emits `Completed`
//! keyed by the approval `call_id` (set via [`EventTranslator::on_approval_request`])
//! rather than the synthetic per-turn sequence id.

use std::collections::HashSet;
use std::sync::Mutex;

use rupoo::llm::{AgentEvent, TokenUsage};

use crate::protocol::events::{
    AgentStreamEvent, FinishEventData, StartEventData, TextEventData, ToolCallEventData, ToolCallStatus,
};

/// State carried across a single turn so we can synthesize stable `call_id`s
/// for tool calls (the engine only names the tool, it does not assign ids).
#[derive(Default)]
pub struct EventTranslator {
    tool_seq: u64,
    /// Tools that require interactive approval (suppress their `ToolCall`
    ////`ToolResult` bubbles; the `AcpPermission` card represents them).
    approval_tool_names: HashSet<String>,
    /// The call id of the in-flight approval request, if any. Set when the host
    /// drains an approval notice; consumed by the next `ToolResult` for an
    /// approval tool.
    approval_call_id: Mutex<Option<String>>,
}

impl EventTranslator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure the set of tool names that require interactive approval.
    pub fn with_approval_tools(mut self, names: HashSet<String>) -> Self {
        self.approval_tool_names = names;
        self
    }

    /// Record the call id of a pending approval request. The next `ToolResult`
    /// for an approval tool will be emitted with this id.
    pub fn on_approval_request(&self, call_id: String) {
        if let Ok(mut slot) = self.approval_call_id.lock() {
            *slot = Some(call_id);
        }
    }

    fn is_approval_tool(&self, tool_name: &str) -> bool {
        let lower = tool_name.to_lowercase();
        let base = lower.split_whitespace().next().unwrap_or(&lower);
        self.approval_tool_names.iter().any(|s| s == base)
            || base.starts_with("/bin/sh")
            || base.starts_with("/bin/bash")
            || base.starts_with("/usr/bin/env")
    }

    /// Translate one engine event into zero or more wire events.
    ///
    /// `TextDelta` → `Text` (appended to the current assistant bubble).
    /// `ToolCall`   → `ToolCall` with `status = Pending` (announces intent).
    /// `ToolResult`→ `ToolCall` with `status = Completed` (carries output).
    pub fn translate(&mut self, event: AgentEvent) -> Vec<AgentStreamEvent> {
        match event {
            AgentEvent::TextDelta(text) => {
                vec![AgentStreamEvent::Text(TextEventData { content: text })]
            }
            AgentEvent::ToolCall { tool_name, args } => {
                // Approval tools are represented by the AcpPermission card, not
                // a running tool bubble.
                if self.is_approval_tool(&tool_name) {
                    return Vec::new();
                }
                self.tool_seq += 1;
                let call_id = format!("rupoo-tool-{}", self.tool_seq);
                vec![AgentStreamEvent::ToolCall(ToolCallEventData {
                    call_id,
                    name: tool_name,
                    args: parse_args(&args),
                    status: ToolCallStatus::Running,
                    input: None,
                    output: None,
                    description: None,
                })]
            }
            AgentEvent::ToolResult { tool_name, result } => {
                if self.is_approval_tool(&tool_name) {
                    // Emit completion under the approval call id so the UI
                    // updates the confirmation card's tool bubble.
                    let call_id = self
                        .approval_call_id
                        .lock()
                        .ok()
                        .and_then(|mut s| s.take())
                        .unwrap_or_else(|| {
                            self.tool_seq += 1;
                            format!("rupoo-tool-{}", self.tool_seq)
                        });
                    return vec![AgentStreamEvent::ToolCall(ToolCallEventData {
                        call_id,
                        name: tool_name,
                        args: serde_json::Value::Null,
                        status: ToolCallStatus::Completed,
                        input: None,
                        output: Some(result),
                        description: None,
                    })];
                }
                // Pair with the most recent pending call of the same tool.
                let call_id = format!("rupoo-tool-{}", self.tool_seq.max(1));
                vec![AgentStreamEvent::ToolCall(ToolCallEventData {
                    call_id,
                    name: tool_name,
                    args: serde_json::Value::Null,
                    status: ToolCallStatus::Completed,
                    input: None,
                    output: Some(result),
                    description: None,
                })]
            }
        }
    }

    /// Emitted once when a turn begins.
    pub fn start_event(session_id: Option<String>) -> AgentStreamEvent {
        AgentStreamEvent::Start(StartEventData { session_id })
    }

    /// Emitted once when a turn converges. `usage` carries the turn's prompt /
    /// completion token counts when the engine reports them (the per-reply
    /// input/output data sizes shown in the frontend).
    pub fn finish_event(session_id: Option<String>, usage: Option<TokenUsage>) -> AgentStreamEvent {
        AgentStreamEvent::Finish(FinishEventData {
            session_id,
            input_tokens: usage.map(|u| u.prompt_tokens),
            output_tokens: usage.map(|u| u.completion_tokens),
            elapsed_ms: None,
        })
    }
}

/// Best-effort parse of a tool argument string into JSON. Tool-call args from
/// the engine are opaque strings; we keep them verbatim when they are not
/// valid JSON so the frontend still receives the raw payload.
fn parse_args(args: &str) -> serde_json::Value {
    if args.trim().is_empty() {
        return serde_json::Value::Null;
    }
    serde_json::from_str(args).unwrap_or_else(|_| serde_json::Value::String(args.to_string()))
}
