//! Build helpers: SessionBuildInputs, build_session_instance, spec helpers.
use super::*;

/// Open a claude/codex `SessionBackend` via the clean-slate connection and wrap it
/// as an `AgentInstance::Session`. Called from the ACP factory when the resolved
/// backend is claude/codex and a spawner is available. `backend_label` is the
/// authoritative vendor ("claude"/"codex"); other labels return `None` so the caller
/// falls back to the ACP manager path.
/// Everything the caller (`factory::acp::build`) already resolved and that the
/// session assembly needs. Bundled so `build_session_instance` is the SINGLE
/// place that maps an ACP build request → the clean-slate `SessionSpec`/
/// `SessionConfig`, mirroring clean-slate's `build_runtime` (spec_and_config +
/// resolve_session_init + the per-backend spawn_env/sandbox/approval seams). Every
/// field here has a 1:1 counterpart in that path.
pub struct SessionBuildInputs<'a> {
    /// The conversation this session belongs to (the clean-slate `session_id`).
    pub conversation_id: String,
    /// Owner Core user. Scopes every acp_session persistence write, the MCP
    /// server resolution, and the catalog writeback (multi-account boundary).
    pub user_id: String,
    /// The resolved workspace path (`SessionConfig.cwd`).
    pub workspace: String,
    /// The conversation's persisted build `extra` (mode/model/mcp/preset/skills).
    pub config: &'a AcpBuildExtra,
    /// The resolved catalog row. Used to normalize the persisted/requested mode
    /// alias (`yolo`/`yoloNoSandbox` → the row's `yolo_id`; codex `default`/`autoEdit`
    /// → `auto`) into the backend-native mode id, exactly as the ACP path does via
    /// `initial_mode_from_params`. Without this a conversation persisted with a
    /// generic alias resumes by handing the raw alias to the backend (claude rejects
    /// an unknown permission-mode id; codex gets a non-native mode → wrong policy).
    pub metadata: &'a roseui_api_types::AgentMetadata,
    /// The persisted runtime snapshot, when present. Its `current_mode_id` /
    /// `current_model_id` are the interactive-switch-persisted selections and take
    /// precedence over the create-time `config` values — the same precedence
    /// clean-slate's `spec_and_config` applies (`current_mode_id` ⟶ `session_mode`).
    pub session_snapshot: Option<&'a PersistedSessionState>,
    /// The CLI-assigned backend session id anchor. `Some` ⇒ `SessionSpec::Resume`
    /// (the same signal clean-slate's `spec_and_config` uses); `None` ⇒ `Fresh`.
    pub backend_session_id: Option<String>,
    /// User-configured MCP server repository (feature ELECTRON-1JG). `None` on
    /// paths that never inject MCP (tests) ⇒ no injection.
    pub mcp_server_repo: Option<&'a Arc<dyn IMcpServerRepository>>,
    /// The conversation runtime context env (`ROSEUI_USER_ID` /
    /// `ROSEUI_CONVERSATION_ID` / `ROSEUI_HELPER_BIN` / `ROSEUI_BASE_URL` /
    /// `ROSEUI_RUNTIME_TOKEN`, filled by `apply_conversation_runtime_context`).
    /// The legacy ACP path injects these into every agent spawn via
    /// `apply_acp_launch_policy`; the direct-CLI path forwards them through
    /// `SessionConfig.spawn_env` so team/helper tooling inside the agent process
    /// keeps working. Empty ⇒ nothing injected.
    pub runtime_env: &'a [(String, String)],
    /// Broadcaster forwarded to the MCP resolver for runtime-resolution reporting
    /// parity with the legacy ACP path.
    pub broadcaster: Arc<dyn EventBroadcaster>,
    /// The resolved catalog row id + the registry's catalog sender, used to write
    /// the backend's discovered modes/models/commands back into `agent_metadata`
    /// (GAP #7 / G5) so the `/api/agents` picker stays fresh. `None` on paths that
    /// have no catalog row to refresh.
    pub catalog_writeback: Option<(String, crate::registry::CatalogSender)>,
    /// The `acp_session` persistence sink. The event pump writes the resume anchor
    /// (`BackendBound` → `session_id`) + observed mode/model (`ConfigChanged`) here —
    /// the writes the legacy ACP path performed via `AcpSessionSyncService`, which
    /// this direct-CLI path bypasses. `None` (tests) = no persistence.
    pub acp_session_repo: Option<Arc<dyn IAcpSessionRepository>>,
    /// DEV (`--dump-prompts`): the already-resolved `<data_dir>/prompt-dumps`
    /// dir, or `None` when off. `build_session_instance` uses it for the
    /// spawn-time `session-cli-config` dump AND threads it (with the vendor
    /// label) into the `SessionAgentTask` for the send-time dump.
    pub prompt_dump_dir: Option<std::path::PathBuf>,
}

/// The pure spec + mode/model mapping — the sibling of clean-slate's
/// `spec_and_config`. Extracted from `build_session_instance` so it is unit-testable
/// without spawning a backend.
///
/// - Resume when the row carries a `backend_session_id` anchor, else Fresh (both key
///   on the conversation id).
/// - `mode`: the interactive-switch-persisted `snapshot.current_mode_id` wins over the
///   create-time `config.session_mode`; empty-filtered; NO default minted (each backend
///   safe-defaults).
/// - `model`: symmetric — `snapshot.current_model_id` wins over `config.current_model_id`.
///   A BARE runtime model id (never the JSON `ProviderWithModel` blob — clean-slate #7).
pub(crate) fn spec_mode_model(
    conversation_id: &str,
    backend_session_id: Option<String>,
    config: &AcpBuildExtra,
    session_snapshot: Option<&PersistedSessionState>,
    metadata: &roseui_api_types::AgentMetadata,
) -> (roseui_session::SessionSpec, Option<String>, Option<String>) {
    use roseui_session::SessionSpec;
    let spec = match &backend_session_id {
        Some(_) => SessionSpec::Resume {
            session_id: conversation_id.to_owned(),
            backend_session_id,
        },
        None => SessionSpec::Fresh {
            session_id: conversation_id.to_owned(),
        },
    };
    // Normalize the resolved mode alias into the backend-native id — the SAME
    // transform the ACP path applies in `initial_mode_from_params`. RoseUi persists
    // generic aliases (`yolo`/`yoloNoSandbox`; codex `default`/`autoEdit`); handing
    // those raw to the backend on resume rejects (claude unknown permission-mode) or
    // mis-policies (codex non-native mode). `normalize_requested_mode` maps them via
    // the catalog row's `yolo_id` / backend label; a mode without an alias passes
    // through unchanged. Runs BEFORE the codex sandbox/approval derivation downstream
    // (which matches both the alias and the native id, so ordering is safe).
    let mode = session_snapshot
        .and_then(|s| s.current_mode_id.as_ref().map(|m| m.as_str().to_owned()))
        .or_else(|| config.session_mode.clone())
        .map(|m| crate::manager::acp::mode_normalize::normalize_requested_mode(metadata, &m))
        .filter(|s| !s.is_empty());
    let model = session_snapshot
        .and_then(|s| s.current_model_id.as_ref().map(|m| m.as_str().to_owned()))
        .or_else(|| config.current_model_id.clone())
        .filter(|s| !s.is_empty());
    (spec, mode, model)
}

/// Build a claude/codex `SessionAgentTask` (the session-model port's `IAgentTask`)
/// from a resolved ACP build request, or `Ok(None)` for a non-session backend.
///
/// This is the faithful port of clean-slate `build_runtime`'s per-conversation
/// assembly (`crates/roseui-app/src/session_runtime/mod.rs`): it resolves the
/// resume spec, the mode/model precedence, the MCP + preset + skills init surface,
/// the claude cc-switch provider env, and the codex sandbox/approval policy — so a
/// claude/codex session started through the ACP factory is byte-equivalent to one
/// started through the clean-slate registry.
pub async fn build_session_instance(
    backend_label: &str,
    inputs: SessionBuildInputs<'_>,
    spawner: Arc<dyn roseui_process::Spawner>,
) -> Result<Option<crate::agent_task::AgentInstance>, AgentError> {
    use roseui_session::{
        BackendConnection, ClaudeConnection, CodexConnection, McpServerSpec, SessionConfig, SessionInit, SessionSpec,
    };

    let connection: Box<dyn BackendConnection> = match backend_label {
        "claude" => Box::new(ClaudeConnection::new(spawner)),
        "codex" => Box::new(CodexConnection::new(spawner)),
        _ => return Ok(None),
    };

    let SessionBuildInputs {
        conversation_id,
        user_id,
        workspace,
        config,
        metadata,
        session_snapshot,
        backend_session_id,
        mcp_server_repo,
        runtime_env,
        broadcaster,
        catalog_writeback,
        acp_session_repo,
        prompt_dump_dir,
    } = inputs;

    // GAP #1/#2 — the pure spec + mode/model mapping (resume anchor → Resume/Fresh,
    // snapshot-wins precedence). Extracted so it is unit-testable in isolation, the
    // exact sibling of clean-slate's `spec_and_config`.
    let (spec, mode, model) = spec_mode_model(&conversation_id, backend_session_id, config, session_snapshot, metadata);

    // GAP #3 — MCP init surface: resolve user-configured servers to the neutral
    // spec (clean-slate resolve_session_init), fold in the inline snapshot, then
    // prepend the team coordination MCP. Same order as the app boundary.
    let mut neutral = match mcp_server_repo {
        Some(repo) => {
            crate::mcp_resolve::resolve_session_mcp_servers(
                repo.as_ref(),
                &user_id,
                config.mcp_server_ids.as_deref(),
                &conversation_id,
                broadcaster,
            )
            .await
        }
        None => Vec::new(),
    };
    neutral.extend(config.session_mcp_servers.iter().cloned());
    let mut mcp_servers: Vec<McpServerSpec> = neutral.iter().map(session_server_to_spec).collect();

    // Built-in MCP servers are PREPENDED before the user's servers
    // (clean-slate + legacy acp_assembler ordering). Wiki first, then team.
    if let Some(cfg) = config.wiki_mcp_stdio_config.as_ref() {
        let mut with_wiki = vec![wiki_mcp_server_spec(cfg)];
        with_wiki.append(&mut mcp_servers);
        mcp_servers = with_wiki;
    }
    if let Some(cfg) = config.team_mcp_stdio_config.as_ref() {
        let mut with_team = vec![team_mcp_server_spec(cfg)];
        with_team.append(&mut mcp_servers);
        mcp_servers = with_team;
    }

    // GAP #4 — preset_context + skills carried into the init surface.
    let init = SessionInit {
        mcp_servers,
        skills: config.skills.clone(),
        preset_context: config.preset_context.clone(),
        // acp/codex resume via SessionSpec::Resume; no in-band snapshot needed.
        session_snapshot: None,
        resume: matches!(spec, SessionSpec::Resume { .. }),
    };

    let mut session_config = SessionConfig {
        cwd: Some(workspace.clone()),
        model,
        mode,
        init,
        // Packaged app: resolve the bundled claude/codex binary and forward its
        // absolute path so the backend spawns OUR CLI, not the user's PATH one.
        // Bundled-missing / dev falls back to a PATH lookup via
        // `resolve_command_path` (NOT the bare name): on Windows, npm installs
        // ship `claude.cmd`/`codex.cmd` shims which `CreateProcess` does not
        // find from a bare name (#299 parity; Rust std runs `.cmd` via
        // `cmd.exe` since the BatBadBut fix). `None` (nothing on PATH either)
        // keeps the bare name so the spawn error stays diagnosable. Detection
        // (cli_probe) stays PATH-only and is unaffected.
        cli_program: roseui_runtime::resolve_bundled_cli(backend_label)
            .or_else(|| roseui_runtime::resolve_command_path(backend_label)),
        ..Default::default()
    };

    // Spawn env (legacy spawn-surface parity, claude AND codex).
    session_config.spawn_env = assemble_spawn_env(&metadata.env, runtime_env);
    if !session_config.spawn_env.is_empty() {
        let keys: Vec<&str> = session_config.spawn_env.iter().map(|e| e.name.as_str()).collect();
        tracing::info!(conv_id = %conversation_id, ?keys, "session spawn env: agent overrides + runtime context");
    }

    // GAP #5 — claude cc-switch provider env: inject ANTHROPIC_BASE_URL /
    // ANTHROPIC_AUTH_TOKEN (third-party relay creds) into the spawn, mirroring the
    // legacy ACP-claude path. Empty (no cc-switch config) = byte-identical spawn.
    if backend_label == "claude" {
        let provider_env = crate::cc_switch::read_claude_provider_env();
        if !provider_env.is_empty() {
            let keys: Vec<String> = provider_env.keys().cloned().collect();
            session_config.spawn_env.extend(
                provider_env
                    .into_iter()
                    .map(|(name, value)| roseui_common::EnvVar { name, value }),
            );
            tracing::info!(conv_id = %conversation_id, ?keys, "cc-switch: provider env injected into claude spawn");
        }
    }

    // GAP #6 — codex sandbox + approval policy resolved from the requested mode
    // (clean-slate codex_sandbox_for_mode / codex_approval_for_mode). A full-access
    // / yolo mode escalates the sandbox and drops approval prompts; everything else
    // (incl. None) leaves these None so the backend safe-defaults
    // (workspace-write / on-request).
    if backend_label == "codex" {
        if let Some(sandbox) = codex_sandbox_for_mode(session_config.mode.as_deref()) {
            tracing::info!(conv_id = %conversation_id, sandbox, "codex: sandbox policy resolved from requested mode");
            session_config.sandbox_mode = Some(sandbox.to_string());
        }
        if let Some(approval) = codex_approval_for_mode(session_config.mode.as_deref()) {
            tracing::info!(conv_id = %conversation_id, approval, "codex: approval policy resolved from requested mode");
            session_config.approval_policy = Some(approval.to_string());
        }
    }

    // #4 — the persisted reasoning-effort level (claude only). There is no spawn-time
    // effort flag (effort rides a post-open control_request, NOT `--`args like
    // model/mode), so it cannot go into `SessionConfig`; instead we re-apply it AFTER
    // open. codex effort is not a standalone selection (it rides collaborationMode via
    // SetMode), so this is claude-scoped. Read from the snapshot's config_selections
    // (the map `set_config_option` persisted under EFFORT_CONFIG_KEY).
    let persisted_effort = (backend_label == "claude")
        .then(|| {
            session_snapshot.and_then(|s| {
                s.config_selections
                    .iter()
                    .find(|(k, _)| k.as_str() == EFFORT_CONFIG_KEY)
                    .map(|(_, v)| v.as_str().to_owned())
            })
        })
        .flatten()
        .filter(|s| !s.is_empty());

    // DEV (`--dump-prompts`): dump the resolved SessionConfig BEFORE it moves
    // into open_session. Best-effort — a failure only warns, never fails open.
    // Borrows `prompt_dump_dir`; the send-side `SessionPromptDump` consumes it
    // (via `.map`) later, after open_session (which only moves `session_config`).
    if let Some(dir) = prompt_dump_dir.as_ref() {
        let backend_static: &'static str = if backend_label == "claude" { "claude" } else { "codex" };
        let value = build_session_cli_config_dump_value(backend_static, &session_config);
        let input = value.get("input").cloned().unwrap_or(serde_json::Value::Null);
        let resolved_context = value
            .get("resolved_context")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        match crate::dev_prompt_dump::dump_agent_final_input(
            dir,
            crate::dev_prompt_dump::AgentFinalInputDump {
                kind: "session-cli-config",
                backend: backend_static,
                conversation_id: &conversation_id,
                session_id: None,
                msg_id: None,
                turn_id: None,
                input,
                resolved_context,
            },
        ) {
            Ok(path) => tracing::debug!(
                conversation_id = %conversation_id,
                path = %path.display(),
                "DEV session-cli config dump written"
            ),
            Err(error) => tracing::warn!(
                conversation_id = %conversation_id,
                error = %error,
                "DEV session-cli config dump failed"
            ),
        }
    }

    let backend = connection
        .open_session(spec, session_config)
        .await
        .map_err(|e| match e {
            // #410 parity: a missing/non-directory workspace keeps its dedicated
            // error class end-to-end (ProcessError::WorkspaceUnavailable →
            // BackendError::WorkspaceUnavailable → here), so the frontend gets
            // WORKSPACE_PATH_RUNTIME_UNAVAILABLE exactly like the legacy spawn
            // path — not an opaque 502.
            roseui_session::BackendError::WorkspaceUnavailable(path) => {
                AgentError::workspace_path_runtime_unavailable(path)
            }
            e => AgentError::bad_gateway(format!("open {backend_label} session: {e}")),
        })?;

    // Re-apply the persisted effort now that the session is open. The backend validates
    // it against the current model's advertised catalog (permissive until the catalog
    // is discovered) and drops it if unsupported — the same clear_invalid_desired_*
    // semantics as the codex model/mode reconcile. Best-effort: a dispatch failure must
    // not fail the open (the session is usable; only the persisted effort is lost).
    if let Some(effort) = persisted_effort {
        if let Err(e) = backend
            .dispatch(Command::SetConfigOption {
                option_id: EFFORT_CONFIG_KEY.to_owned(),
                value: effort.clone(),
            })
            .await
        {
            tracing::warn!(conv_id = %conversation_id, effort = %effort, error = %e, "session-port: re-applying persisted effort failed (session usable, effort not restored)");
        } else {
            tracing::info!(conv_id = %conversation_id, effort = %effort, "session-port: re-applied persisted reasoning effort after open");
        }
    }

    // GAP #7 (G5): project the backend's discovered catalog back into agent_metadata
    // so the cold-start picker stays fresh. Best-effort, detached, off the open path.
    if let Some((agent_id, catalog_tx)) = catalog_writeback {
        spawn_catalog_writeback(agent_id, user_id.clone(), backend.clone(), catalog_tx);
    }

    let prompt_dump = prompt_dump_dir.map(|dir| SessionPromptDump {
        dir,
        // Only "claude"/"codex" reach here (the caller guards the match; other
        // labels returned None above), so this binary choice is total.
        backend: if backend_label == "claude" { "claude" } else { "codex" },
    });

    let task = SessionAgentTask::new_with_preload(
        AgentType::Acp,
        conversation_id,
        user_id,
        workspace,
        backend,
        acp_session_repo,
        &metadata.handshake,
        prompt_dump,
    );
    Ok(Some(crate::agent_task::AgentInstance::Session(task)))
}

/// Assemble the direct-CLI spawn env (legacy spawn-surface parity; order
/// matters — later entries win in `ManagedProcess::spawn`):
///  1. per-agent env overrides (`AgentMetadata.env`, repair panel) — the legacy
///     path injected these via `resolve_agent_command_spec`; `ROSEUI_*`/`PATH`/…
///     keys are already filtered at the registry (`is_blocked_override_env_key`),
///     so they cannot shadow the runtime context below.
///  2. the `ROSEUI_*` conversation runtime context (`ROSEUI_USER_ID` /
///     `ROSEUI_CONVERSATION_ID` / `ROSEUI_HELPER_BIN` / `ROSEUI_BASE_URL` /
///     `ROSEUI_RUNTIME_TOKEN`) — the legacy path appended these via
///     `apply_acp_launch_policy` for every agent spawn.
pub(crate) fn assemble_spawn_env(
    agent_env: &[roseui_api_types::AgentEnvEntry],
    runtime_env: &[(String, String)],
) -> Vec<roseui_common::EnvVar> {
    let mut env: Vec<roseui_common::EnvVar> = agent_env
        .iter()
        .map(|entry| roseui_common::EnvVar {
            name: entry.name.clone(),
            value: entry.value.clone(),
        })
        .collect();
    env.extend(runtime_env.iter().map(|(name, value)| roseui_common::EnvVar {
        name: name.clone(),
        value: value.clone(),
    }));
    env
}

/// Build the `session-cli-config` dump payload from the resolved `SessionConfig`
/// captured just before `open_session`. Symmetric with the ACP path's
/// `build_acp_final_input_dump_value`: returns `{ "input", "resolved_context" }`.
/// `SessionConfig` has no `Serialize`, so fields are mapped by hand. Contents
/// are RAW (dev-only, `--dump-prompts`): secrets in `spawn_env` / MCP env are
/// not redacted, matching the existing acp dump.
pub(crate) fn build_session_cli_config_dump_value(backend: &str, cfg: &roseui_session::SessionConfig) -> serde_json::Value {
    use roseui_session::McpTransport;
    let mcp_servers: Vec<serde_json::Value> = cfg
        .init
        .mcp_servers
        .iter()
        .map(|s| {
            let transport = match &s.transport {
                McpTransport::Stdio { command, args, env } => serde_json::json!({
                    "type": "stdio",
                    "command": command,
                    "args": args,
                    "env": env.iter().map(|(k, v)| serde_json::json!({ "name": k, "value": v })).collect::<Vec<_>>(),
                }),
                McpTransport::Http { url, headers } => serde_json::json!({
                    "type": "http",
                    "url": url,
                    "headers": headers.iter().map(|(k, v)| serde_json::json!({ "name": k, "value": v })).collect::<Vec<_>>(),
                }),
                McpTransport::Sse { url, headers } => serde_json::json!({
                    "type": "sse",
                    "url": url,
                    "headers": headers.iter().map(|(k, v)| serde_json::json!({ "name": k, "value": v })).collect::<Vec<_>>(),
                }),
            };
            serde_json::json!({ "name": s.name, "transport": transport })
        })
        .collect();

    let spawn_env: Vec<serde_json::Value> = cfg
        .spawn_env
        .iter()
        .map(|e| serde_json::json!({ "name": e.name, "value": e.value }))
        .collect();

    serde_json::json!({
        "input": {
            "backend": backend,
            "mode": cfg.mode,
            "model": cfg.model,
            "cli_program": cfg.cli_program.as_ref().map(|p| p.to_string_lossy()),
            "sandbox_mode": cfg.sandbox_mode,
            "approval_policy": cfg.approval_policy,
            "resume": cfg.init.resume,
        },
        "resolved_context": {
            "preset_context": cfg.init.preset_context,
            "skills": cfg.init.skills,
            "mcp_servers": mcp_servers,
            "spawn_env": spawn_env,
            "extra_args": cfg.extra_args,
        }
    })
}

/// Convert a neutral `SessionMcpServer` (already stdio-launch-resolved by
/// `mcp_resolve`) into the crate-local `McpServerSpec`. Verbatim port of
/// clean-slate `session_runtime::session_server_to_spec`.
pub(crate) fn session_server_to_spec(server: &roseui_api_types::SessionMcpServer) -> roseui_session::McpServerSpec {
    use roseui_api_types::SessionMcpTransport as T;
    use roseui_session::{McpServerSpec, McpTransport};
    let sorted = |m: &std::collections::HashMap<String, String>| -> Vec<(String, String)> {
        let mut v: Vec<(String, String)> = m.iter().map(|(k, val)| (k.clone(), val.clone())).collect();
        v.sort_by(|a, b| a.0.cmp(&b.0));
        v
    };
    let transport = match &server.transport {
        T::Stdio { command, args, env } => McpTransport::Stdio {
            command: command.clone(),
            args: args.clone(),
            env: sorted(env),
        },
        T::Http { url, headers } | T::StreamableHttp { url, headers } => McpTransport::Http {
            url: url.clone(),
            headers: sorted(headers),
        },
        T::Sse { url, headers } => McpTransport::Sse {
            url: url.clone(),
            headers: sorted(headers),
        },
    };
    McpServerSpec {
        name: server.name.clone(),
        transport,
    }
}

/// The team coordination MCP server as a neutral stdio spec. Verbatim port of
/// clean-slate `session_runtime::team_mcp_server_spec` (name = TEAM_MCP_SERVER_NAME,
/// arg `mcp-team-stdio`, env PORT/TOKEN/SLOT_ID) so a session-model teammate joins
/// the SAME per-team TCP bridge the ACP path used.
pub(crate) fn team_mcp_server_spec(cfg: &roseui_api_types::TeamMcpStdioConfig) -> roseui_session::McpServerSpec {
    use roseui_api_types::TeamMcpStdioConfig as C;
    roseui_session::McpServerSpec {
        name: roseui_api_types::TEAM_MCP_SERVER_NAME.to_owned(),
        transport: roseui_session::McpTransport::Stdio {
            command: cfg.binary_path.clone(),
            args: vec!["mcp-team-stdio".to_owned()],
            env: vec![
                (C::ENV_PORT.to_owned(), cfg.port.to_string()),
                (C::ENV_TOKEN.to_owned(), cfg.token.clone()),
                (C::ENV_SLOT_ID.to_owned(), cfg.slot_id.clone()),
            ],
        },
    }
}

/// The wiki MCP server as a neutral stdio spec (name = WIKI_MCP_SERVER_NAME,
/// arg `mcp-wiki-stdio`, env ROSEUI_WIKI_DB_PATH). The wiki stdio bridge opens
/// the SQLite database directly — no TCP forwarding.
pub(crate) fn wiki_mcp_server_spec(cfg: &roseui_api_types::WikiMcpStdioConfig) -> roseui_session::McpServerSpec {
    roseui_session::McpServerSpec {
        name: roseui_api_types::WIKI_MCP_SERVER_NAME.to_owned(),
        transport: roseui_session::McpTransport::Stdio {
            command: cfg.binary_path.clone(),
            args: vec!["mcp-wiki-stdio".to_owned()],
            env: vec![("ROSEUI_WIKI_DB_PATH".to_owned(), cfg.db_path.clone())],
        },
    }
}

