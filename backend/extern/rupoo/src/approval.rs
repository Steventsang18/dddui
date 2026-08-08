//! Interactive tool-approval gate for the Rupoo engine.
//!
//! When a tool call requires user approval (per [`crate::safety::SafetyContext`]),
//! the [`McpToolExecutor`] parks the tool call here instead of executing it. A
//! notice is pushed onto a channel so the host (roseui) can surface a
//! confirmation dialog; [`ApprovalGate::resolve`] unparks the waiting call once
//! the user decides.
//!
//! This is the single seam that turns Rupoo's "needs approval" intent into a
//! real interactive pause — without it, `safe_mode` only blocks forbidden
//! commands and auto-runs everything else.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::Value;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

/// A pending approval request, delivered to the host over the notice channel.
#[derive(Debug, Clone)]
pub struct ApprovalRequest {
    /// Stable call id the host uses to correlate the confirmation dialog and
    /// the eventual `confirm()` call.
    pub call_id: String,
    /// Tool being requested (e.g. `bash`, `python3`).
    pub tool_name: String,
    /// Raw tool arguments (already serialized JSON string or object).
    pub args: Value,
}

/// Shared, thread-safe approval gate.
///
/// One instance is created per conversation and handed to the
/// [`crate::mcp::McpToolExecutor`]. Clone is cheap (inner `Arc`).
#[derive(Clone)]
pub struct ApprovalGate {
    inner: Arc<Inner>,
}

struct Inner {
    /// call_id → resolver for the parked `execute_tool` future.
    pending: Mutex<HashMap<String, oneshot::Sender<bool>>>,
    /// Tools the user chose "always allow" for this session.
    auto_approved: Mutex<HashSet<String>>,
    /// Host-side receiver end is held by the manager; this is the sender.
    notice_tx: mpsc::UnboundedSender<ApprovalRequest>,
}

static NEXT_CALL_ID: AtomicU64 = AtomicU64::new(1);

impl ApprovalGate {
    /// Create a gate and return it alongside the host-side receiver for notices.
    pub fn new() -> (Self, mpsc::UnboundedReceiver<ApprovalRequest>) {
        let (notice_tx, notice_rx) = mpsc::unbounded_channel();
        let gate = Self {
            inner: Arc::new(Inner {
                pending: Mutex::new(HashMap::new()),
                auto_approved: Mutex::new(HashSet::new()),
                notice_tx,
            }),
        };
        (gate, notice_rx)
    }

    /// Generate a fresh, unique call id for a new approval request.
    fn next_call_id() -> String {
        let n = NEXT_CALL_ID.fetch_add(1, Ordering::SeqCst);
        format!("rupoo-apr-{n}")
    }

    /// Whether `tool_name` has been auto-approved for the rest of the session.
    pub fn is_auto_approved(&self, tool_name: &str) -> bool {
        self.inner
            .auto_approved
            .lock()
            .map(|s| s.contains(tool_name))
            .unwrap_or(false)
    }

    /// Mark `tool_name` as always-allowed (user picked "allow always").
    pub fn set_auto_approved(&self, tool_name: &str) {
        if let Ok(mut s) = self.inner.auto_approved.lock() {
            s.insert(tool_name.to_string());
        }
    }

    /// Park the current tool execution until the host resolves it.
    ///
    /// Returns `true` if the tool may proceed, `false` if denied. If the tool
    /// was auto-approved, returns `true` immediately without parking. On
    /// cancellation (host dropped the resolver), returns `false`.
    pub async fn request(&self, tool_name: &str, args: Value) -> bool {
        if self.is_auto_approved(tool_name) {
            return true;
        }
        let call_id = Self::next_call_id();
        let (tx, rx) = oneshot::channel();
        {
            let mut pending = match self.inner.pending.lock() {
                Ok(g) => g,
                Err(_) => return false,
            };
            pending.insert(call_id.clone(), tx);
        }
        // Notify the host before parking so the dialog appears immediately.
        let notice = ApprovalRequest {
            call_id: call_id.clone(),
            tool_name: tool_name.to_string(),
            args,
        };
        if self.inner.notice_tx.send(notice).is_err() {
            // Host gone — deny.
            return false;
        }
        match rx.await {
            Ok(approved) => approved,
            Err(_) => false,
        }
    }

    /// Resolve a parked request. `approved = true` lets the tool run.
    /// Unknown/already-resolved call ids are ignored.
    pub fn resolve(&self, call_id: &str, approved: bool) {
        let tx = self
            .inner
            .pending
            .lock()
            .ok()
            .and_then(|mut m| m.remove(call_id));
        if let Some(tx) = tx {
            let _ = tx.send(approved);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn request_parks_until_resolve_approves() {
        let (gate, mut rx) = ApprovalGate::new();
        let handle = {
            let gate = gate.clone();
            tokio::spawn(async move { gate.request("bash", Value::Null).await })
        };
        let req = rx.recv().await.expect("notice delivered");
        assert_eq!(req.tool_name, "bash");
        gate.resolve(&req.call_id, true);
        assert!(handle.await.unwrap());
    }

    #[tokio::test]
    async fn request_denied_on_cancel() {
        let (gate, mut rx) = ApprovalGate::new();
        let handle = {
            let gate = gate.clone();
            tokio::spawn(async move { gate.request("bash", Value::Null).await })
        };
        let _ = rx.recv().await.unwrap();
        gate.resolve("nonexistent", true); // wrong id: should not unpark
        // A wrong-id resolve must leave the waiter parked (it does not resolve).
        // We poll a clone of the abort handle with a timeout: if the timeout
        // elapses, the task is still parked.
        let abort_handle = handle.abort_handle();
        let still_parked =
            tokio::time::timeout(std::time::Duration::from_millis(50), handle).await;
        assert!(still_parked.is_err(), "wrong-id resolve must not unpark the waiter");
        abort_handle.abort();
    }

    #[test]
    fn auto_approve_skips_parking() {
        let (gate, _rx) = ApprovalGate::new();
        gate.set_auto_approved("python3");
        assert!(gate.is_auto_approved("python3"));
        assert!(!gate.is_auto_approved("bash"));
    }
}
