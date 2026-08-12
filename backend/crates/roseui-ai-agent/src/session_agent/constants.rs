//! Shared constants for session-agent lifecycle.

pub(crate) const EVENT_CHANNEL_CAPACITY: usize = 512;

// Option ids for the generic tool-approval card. `confirm()` maps the incoming
// `data` string against these to pick the PermissionDecision; anything else is
// treated as an AskUserQuestion answer label (Approved + `selected`).
pub(crate) const PERM_ALLOW: &str = "allow";
pub(crate) const PERM_ALLOW_ALWAYS: &str = "allow_always";
pub(crate) const PERM_REJECT: &str = "reject";

/// The `config_selections` key under which a claude session's chosen reasoning-effort
/// level is persisted. claude emits NO `ConfigChanged` for effort (only mode/model), so
/// `set_config_option` persists it here directly and `build_session_instance` re-applies
/// it after open (there is no spawn-time effort flag; it rides a post-open
/// control_request). The three accepted incoming option ids (`effort`/`reasoning_effort`/
/// `thought_level`) all normalize to this one storage key.
pub(crate) const EFFORT_CONFIG_KEY: &str = "effort";
