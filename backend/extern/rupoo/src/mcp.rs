//! MCP tool executor — bridges plan-level ToolCall steps with
//! rig_tools implementations.
//!
//! Architecture: McpToolExecutor directly holds `Box<dyn rig::tool::Tool>`,
//! eliminating the intermediate ToolKind enum and manual serialization.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::RwLock;

use crate::agent::ToolExecutor;
use crate::approval::ApprovalGate;
use crate::error::{AgentError, AgentResult};
use crate::safety::SafetyContext;
use crate::task::McpToolResult;

use rig::tool::Tool;

// -----------------------------------------------------------------------------
// Helper function to extract content from tool output
// -----------------------------------------------------------------------------

/// Extract content from tool output JSON and convert to McpToolResult.
/// Handles different output structures:
/// - EchoOutput: { result: String }
/// - FileReadOutput: { content: String, success: bool, error: Option<String> }
/// - FileWriteOutput: { bytes_written: usize, success: bool, error: Option<String> }
/// - ListDirOutput: { entries: Vec<DirEntry>, success: bool, error: Option<String> }
/// - WebSearchOutput: { results: String, success: bool, error: Option<String> }
/// - ShellExecOutput: { stdout: String, exit_code: Option<i32>, success: bool, error: Option<String> }
fn extract_mcp_result(value: &serde_json::Value) -> McpToolResult {
    // Check if it's already a McpToolResult
    if let Ok(result) = serde_json::from_value::<McpToolResult>(value.clone()) {
        return result;
    }

    // Check for success field
    let success = value
        .get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    if !success {
        // Error case
        let error = value
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("tool execution failed")
            .to_string();
        return McpToolResult::Error { message: error };
    }

    // Success case - extract content from different output types
    let content = if let Some(result) = value.get("result").and_then(|v| v.as_str()) {
        // EchoOutput
        result.to_string()
    } else if let Some(content) = value.get("content").and_then(|v| v.as_str()) {
        // FileReadOutput
        content.to_string()
    } else if let Some(bytes_written) = value.get("bytes_written").and_then(|v| v.as_u64()) {
        // FileWriteOutput
        format!("{} bytes written", bytes_written)
    } else if let Some(results) = value.get("results").and_then(|v| v.as_str()) {
        // WebSearchOutput
        results.to_string()
    } else if let Some(stdout) = value.get("stdout").and_then(|v| v.as_str()) {
        // ShellExecOutput
        stdout.to_string()
    } else if let Some(entries) = value.get("entries").and_then(|v| v.as_array()) {
        // ListDirOutput - format entries
        entries
            .iter()
            .filter_map(|e| {
                let name = e.get("name")?.as_str()?;
                let kind = e.get("kind")?.as_str()?;
                Some(format!("{} ({})", name, kind))
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        // Fallback: serialize the entire value
        serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
    };

    McpToolResult::Success { content }
}

// -----------------------------------------------------------------------------
// BoxedTool trait - wrapper for type-erased tool execution
// -----------------------------------------------------------------------------

/// Trait for type-erased tool execution from JSON parameters.
#[async_trait]
trait BoxedTool: Send + Sync {
    async fn execute_json(&self, params: serde_json::Value) -> Result<serde_json::Value, String>;
}

/// Wrapper that adapts rig::tool::Tool to BoxedTool.
struct ToolWrapper<T>(T);

#[async_trait]
impl<T> BoxedTool for ToolWrapper<T>
where
    T: Tool + Send + Sync,
    T::Args: DeserializeOwned + Send,
    T::Output: Serialize + Send,
{
    async fn execute_json(&self, params: serde_json::Value) -> Result<serde_json::Value, String> {
        let args: T::Args = serde_json::from_value(params)
            .map_err(|e| format!("failed to deserialize args: {}", e))?;

        let output = self
            .0
            .call(args)
            .await
            .map_err(|e| format!("tool execution failed: {}", e))?;

        serde_json::to_value(output).map_err(|e| format!("failed to serialize output: {}", e))
    }
}

// -----------------------------------------------------------------------------
// External stdio MCP client bridge — forwards agent tool calls to a
// subprocess MCP server through a shared `McpClientManager`.
// -----------------------------------------------------------------------------

/// Type-erased bridge that forwards a JSON tool call to an external stdio MCP
/// server via a shared [`crate::mcp_client::McpClientManager`]. Tools are
/// registered under the standard `mcp__<server>__<tool>` wire naming so the
/// agent sees exactly the contract its MCP server declares.
struct McpClientBridge {
    client: Arc<crate::mcp_client::McpClientManager>,
    server: String,
    tool: String,
}

#[async_trait]
impl BoxedTool for McpClientBridge {
    async fn execute_json(&self, params: serde_json::Value) -> Result<serde_json::Value, String> {
        let result = self
            .client
            .call_tool(&self.server, &self.tool, params)
            .await
            .map_err(|e| {
                format!(
                    "mcp server '{}' tool '{}' failed: {}",
                    self.server, self.tool, e
                )
            })?;

        // MCP `tools/call` returns a CallToolResult: { content: [...], isError: bool }.
        let is_error = result
            .get("isError")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let text = flatten_call_tool_content(&result);
        if is_error {
            Ok(serde_json::json!({ "success": false, "error": text }))
        } else {
            Ok(serde_json::json!({ "content": text }))
        }
    }
}

/// Rig [`ToolDyn`] adapter for an external stdio MCP tool. Instances are what
/// the LLM actually sees in its tool list (unlike the executor-side registry
/// which only routes execution). Each instance forwards calls to the shared
/// [`crate::mcp_client::McpClientManager`] over the subprocess channel, so the
/// agent can invoke wiki/team servers through the standard rig tool loop.
#[derive(Clone)]
pub struct McpToolDyn {
    client: Arc<crate::mcp_client::McpClientManager>,
    server: String,
    tool: String,
    wire_name: String,
    description: String,
    parameters: serde_json::Value,
}

impl McpToolDyn {
    /// Build one rig tool from an MCP tool info under the standard
    /// `mcp__<server>__<tool>` wire naming.
    pub fn new(
        client: Arc<crate::mcp_client::McpClientManager>,
        server: &str,
        info: &crate::mcp_client::McpToolInfo,
    ) -> Self {
        Self {
            client,
            server: server.to_string(),
            tool: info.name.clone(),
            wire_name: format!("mcp__{server}__{}", info.name),
            description: info.description.clone().unwrap_or_default(),
            parameters: info
                .input_schema
                .clone()
                .unwrap_or_else(|| serde_json::json!({ "type": "object" })),
        }
    }
}

impl rig::tool::ToolDyn for McpToolDyn {
    fn name(&self) -> String {
        self.wire_name.clone()
    }

    fn definition(
        &self,
        _prompt: String,
    ) -> rig::wasm_compat::WasmBoxedFuture<'_, rig::completion::ToolDefinition> {
        Box::pin(std::future::ready(rig::completion::ToolDefinition {
            name: self.wire_name.clone(),
            description: self.description.clone(),
            parameters: self.parameters.clone(),
        }))
    }

    fn call(
        &self,
        args: String,
    ) -> rig::wasm_compat::WasmBoxedFuture<'_, Result<String, rig::tool::ToolError>> {
        let this = self.clone();
        Box::pin(async move {
            let params: serde_json::Value =
                serde_json::from_str(&args).map_err(rig::tool::ToolError::JsonError)?;
            let result = this.client.call_tool(&this.server, &this.tool, params).await.map_err(|e| {
                rig::tool::ToolError::ToolCallError(Box::new(std::io::Error::other(format!(
                    "mcp server '{}' tool '{}' failed: {}",
                    this.server, this.tool, e
                ))))
            })?;

            // MCP `tools/call` returns a CallToolResult: { content: [...], isError: bool }.
            let is_error = result
                .get("isError")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let text = flatten_call_tool_content(&result);
            let output = if is_error {
                serde_json::json!({ "success": false, "error": text })
            } else {
                serde_json::json!({ "content": text })
            };
            serde_json::to_string(&output).map_err(rig::tool::ToolError::JsonError)
        })
    }
}

/// Flatten an MCP `CallToolResult.content` array (plus textual fallbacks) into
/// a single string the engine's `extract_mcp_result` can surface.
fn flatten_call_tool_content(result: &serde_json::Value) -> String {
    if let Some(parts) = result.get("content").and_then(|v| v.as_array()) {
        let texts: Vec<&str> = parts
            .iter()
            .filter_map(|part| part.get("text").and_then(|t| t.as_str()))
            .collect();
        if !texts.is_empty() {
            return texts.join("\n");
        }
    } else if let Some(text) = result.get("text").and_then(|v| v.as_str()) {
        return text.to_string();
    }
    serde_json::to_string_pretty(result).unwrap_or_else(|_| result.to_string())
}

// -----------------------------------------------------------------------------
// McpToolExecutor — holds Box<dyn Tool> instances
// -----------------------------------------------------------------------------

/// A dispatcher that holds tool instances implementing rig::tool::Tool.
/// Used by the Agent for explicit ToolCall steps and by the MCP server.
///
/// Cloning shares the same registry (Arc<RwLock<...>>), so both copies
/// always see the same registered tools.
#[derive(Clone)]
pub struct McpToolExecutor {
    registry: Arc<RwLock<HashMap<String, Arc<dyn BoxedTool>>>>,
    /// Keep tool definitions for schema listing
    definitions: Arc<RwLock<HashMap<String, rig::completion::ToolDefinition>>>,
    /// Retained safety context so `execute_tool` can consult `needs_approval`.
    safety: SafetyContext,
    /// Optional interactive approval gate. When set and a tool requires
    /// approval, execution parks until the host resolves it.
    approval: Option<Arc<ApprovalGate>>,
}

impl Default for McpToolExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl McpToolExecutor {
    pub fn new() -> Self {
        Self::with_safety(SafetyContext::default())
    }

    pub fn with_defaults() -> Self {
        Self::new()
    }

    /// Create with a pre-configured SafetyContext for file jail enforcement.
    /// File tools will use the SafetyContext's jail_root for path validation.
    pub fn with_safety(safety_ctx: SafetyContext) -> Self {
        let jail_root = safety_ctx.jail_root().map(|p| p.to_path_buf());
        let (tools, defs) = Self::build_tools(jail_root, &None, &std::collections::HashSet::new());
        Self {
            registry: Arc::new(RwLock::new(tools)),
            definitions: Arc::new(RwLock::new(defs)),
            safety: safety_ctx,
            approval: None,
        }
    }

    /// Like [`with_safety`](Self::with_safety) but also installs an interactive
    /// approval gate so high-risk tools pause for user confirmation.
    pub fn with_safety_and_approval(safety_ctx: SafetyContext, gate: Arc<ApprovalGate>) -> Self {
        let mut exec = Self::with_safety(safety_ctx);
        exec.approval = Some(gate);
        exec
    }

    /// Build from an industry-template [`SafetySection`].
    ///
    /// Applies the resolved `allowed_tools` / `excluded_tools` to the tool
    /// registry (allowlist when `allowed_tools` is set; `excluded_tools` is
    /// always subtracted). The `SafetyContext` is derived via
    /// [`SafetyContext::from_section`] so command jail + approval policy also apply.
    pub fn with_safety_section(
        section: &crate::config::SafetySection,
        profile: Option<&crate::config::AgentProfile>,
        gate: Option<Arc<ApprovalGate>>,
    ) -> Self {
        let safety = SafetyContext::from_section(section);
        let jail_root = safety.jail_root().map(|p| p.to_path_buf());

        let allowed = profile
            .and_then(|p| p.allowed_tools.as_ref())
            .map(|v| v.iter().cloned().collect::<HashSet<_>>());
        let excluded = profile
            .and_then(|p| p.excluded_tools.as_ref())
            .map(|v| v.iter().cloned().collect::<HashSet<_>>())
            .unwrap_or_default();

        let (tools, defs) = Self::build_tools(jail_root, &allowed, &excluded);

        Self {
            registry: Arc::new(RwLock::new(tools)),
            definitions: Arc::new(RwLock::new(defs)),
            safety,
            approval: gate,
        }
    }

    /// Immutably borrow the executor's [`SafetyContext`].
    pub fn safety(&self) -> &SafetyContext {
        &self.safety
    }

    fn build_tools(
        jail_root: Option<std::path::PathBuf>,
        allowed: &Option<HashSet<String>>,
        excluded: &HashSet<String>,
    ) -> (
        HashMap<String, Arc<dyn BoxedTool>>,
        HashMap<String, rig::completion::ToolDefinition>,
    ) {
        let mut tools: HashMap<String, Arc<dyn BoxedTool>> = HashMap::new();
        let mut defs: HashMap<String, rig::completion::ToolDefinition> = HashMap::new();

        // Helper to register a tool
        macro_rules! register {
            ($name:expr, $tool:expr) => {{
                let name: String = $name.into();
                // Apply industry-template tool scoping: allowlist (if any) must
                // contain the tool, and excluded_tools always subtracts it.
                let scoped = if let Some(allow) = allowed {
                    allow.contains(&name)
                } else {
                    true
                } && !excluded.contains(&name);
                if scoped {
                    let tool = $tool;
                    let def = futures::executor::block_on(tool.definition(String::new()));
                    tools.insert(name.clone(), Arc::new(ToolWrapper(tool)));
                    defs.insert(name, def);
                }
            }};
        }

        // Register the canonical tool set (single source of truth).
        crate::rupoo_tools!(register, jail_root);

        (tools, defs)
    }

    /// Return all registered tool names.
    pub async fn list_tools(&self) -> Vec<String> {
        let reg = self.registry.read().await;
        reg.keys().cloned().collect()
    }

    /// Return tool descriptions for MCP server.
    pub async fn list_tools_with_desc(&self) -> Vec<(String, String)> {
        let defs = self.definitions.read().await;
        defs.values()
            .map(|d| (d.name.clone(), d.description.clone()))
            .collect()
    }

    /// Return tool schemas for MCP server. Returns (name, description, parameters_json) tuples.
    pub async fn list_tools_with_schema(&self) -> Vec<(String, String, serde_json::Value)> {
        let defs = self.definitions.read().await;
        defs.values()
            .map(|d| (d.name.clone(), d.description.clone(), d.parameters.clone()))
            .collect()
    }

    /// Unregister a tool at runtime.
    pub async fn unregister_tool(&self, name: &str) {
        let mut reg = self.registry.write().await;
        reg.remove(name);
        let mut defs = self.definitions.write().await;
        defs.remove(name);
    }

    /// Register all tools discovered from an external stdio MCP server
    /// (driven by a shared [`crate::mcp_client::McpClientManager`]) under the
    /// standard `mcp__<server>__<tool>` wire naming. This is how DoDidDoneUi
    /// surfaces wiki/team MCP servers to the agent without touching the engine
    /// internals — tool names and schemas come straight from the server's
    /// `tools/list` contract.
    pub async fn register_mcp_client_tools(
        &self,
        client: Arc<crate::mcp_client::McpClientManager>,
        server_name: &str,
        tools: &[crate::mcp_client::McpToolInfo],
    ) {
        for info in tools {
            let wire_name = format!("mcp__{server_name}__{}", info.name);
            let bridge = Arc::new(McpClientBridge {
                client: Arc::clone(&client),
                server: server_name.to_string(),
                tool: info.name.clone(),
            });
            let parameters = info
                .input_schema
                .clone()
                .unwrap_or_else(|| serde_json::json!({ "type": "object" }));
            let definition = rig::completion::ToolDefinition {
                name: wire_name.clone(),
                description: info.description.clone().unwrap_or_default(),
                parameters,
            };
            self.registry.write().await.insert(wire_name.clone(), bridge);
            self.definitions.write().await.insert(wire_name, definition);
        }
    }
}

#[async_trait]
impl ToolExecutor for McpToolExecutor {
    async fn execute_tool(
        &self,
        tool_name: &str,
        params: serde_json::Value,
    ) -> AgentResult<McpToolResult> {
        // Interactive approval: park high-risk tools until the host resolves.
        if let Some(gate) = &self.approval {
            if self.safety.needs_approval(tool_name) {
                let approved = gate.request(tool_name, params.clone()).await;
                if !approved {
                    return Ok(McpToolResult::Error {
                        message: format!("tool '{tool_name}' denied by user approval"),
                    });
                }
            }
        }

        let entry = {
            let reg = self.registry.read().await;
            reg.get(tool_name)
                .map(Arc::clone)
                .ok_or_else(|| AgentError::Mcp(format!("unknown tool: '{tool_name}'")))?
        };

        // Direct async call via BoxedTool trait
        let result = entry.execute_json(params).await;

        match result {
            Ok(value) => Ok(extract_mcp_result(&value)),
            Err(e) => Ok(McpToolResult::Error { message: e }),
        }
    }

    /// Execute multiple tools in parallel using tokio's join_all.
    async fn execute_tools_parallel(
        &self,
        tool_calls: Vec<(String, serde_json::Value)>,
    ) -> Vec<AgentResult<McpToolResult>> {
        let executor = Arc::new(self.clone());
        let futures: Vec<_> = tool_calls
            .into_iter()
            .map(move |(name, params)| {
                let executor_clone = Arc::clone(&executor);
                async move { executor_clone.execute_tool(&name, params).await }
            })
            .collect();
        futures::future::join_all(futures).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_echo_tool() {
        let executor = McpToolExecutor::new();
        let result = executor
            .execute_tool("echo", serde_json::json!({"message": "hello world"}))
            .await
            .unwrap();
        assert!(result.is_success());
        assert_eq!(result.content(), "echo: hello world");
    }

    #[tokio::test]
    async fn test_echo() {
        let executor = McpToolExecutor::new();
        let result = executor
            .execute_tool("echo", serde_json::json!({"message": "hello world"}))
            .await
            .unwrap();
        assert!(result.is_success());
        assert_eq!(result.content(), "echo: hello world");
    }

    #[tokio::test]
    async fn test_file_read_nonexistent() {
        let executor = McpToolExecutor::new();
        let result = executor
            .execute_tool(
                "file_read",
                serde_json::json!({"path": "target/_nonexistent_xyz_test_file"}),
            )
            .await
            .unwrap();
        assert!(!result.is_success());
        assert!(result.error_message().is_some());
    }

    #[tokio::test]
    async fn test_unknown_tool() {
        let executor = McpToolExecutor::new();
        let result = executor
            .execute_tool("nonexistent", serde_json::json!({}))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_directory() {
        let executor = McpToolExecutor::new();
        let result = executor
            .execute_tool("list_directory", serde_json::json!({"path": "."}))
            .await
            .unwrap();
        assert!(
            result.is_success(),
            "list_directory failed: {:?}",
            result.error_message()
        );
        assert!(!result.content().is_empty());
    }

    #[tokio::test]
    async fn test_file_write_and_read() {
        use std::path::Path;
        let test_dir = Path::new("target/_rupoo_mcp_test");
        let _ = std::fs::create_dir_all(test_dir);
        let test_path = test_dir.join("test_write.txt");
        let test_path_str = test_path.to_string_lossy().to_string();

        let executor = McpToolExecutor::new();
        let write_result = executor
            .execute_tool(
                "file_write",
                serde_json::json!({
                    "path": test_path_str,
                    "content": "hello from mcp test"
                }),
            )
            .await
            .unwrap();
        assert!(write_result.is_success());

        let read_result = executor
            .execute_tool(
                "file_read",
                serde_json::json!({"path": test_path.to_string_lossy()}),
            )
            .await
            .unwrap();
        assert!(read_result.is_success());
        assert!(read_result.content().contains("hello from mcp test"));

        // cleanup
        let _ = std::fs::remove_file(&test_path);
        let _ = std::fs::remove_dir(test_dir);
    }

    #[tokio::test]
    async fn test_list_tools() {
        let executor = McpToolExecutor::new();
        let tools = executor.list_tools().await;
        assert_eq!(tools.len(), 11);
        assert!(tools.contains(&"echo".into()));
        assert!(tools.contains(&"file_read".into()));
        assert!(tools.contains(&"file_write".into()));
        assert!(tools.contains(&"file_edit".into()));
        assert!(tools.contains(&"list_directory".into()));
        assert!(tools.contains(&"code_search".into()));
        assert!(tools.contains(&"web_search".into()));
        assert!(tools.contains(&"run_tests".into()));
        assert!(tools.contains(&"check_output".into()));
        assert!(tools.contains(&"diff_check".into()));
    }

    // ── M3 integration: an approval-required tool (shell_exec) parks on the gate
    // and only executes once the host resolves it. Mirrors the roseui confirm
    // flow without needing a live LLM. ──
    #[tokio::test]
    async fn approval_gate_parks_shell_exec_when_denied() {
        let (gate, mut rx) = ApprovalGate::new();
        let gate = Arc::new(gate);
        let exec = McpToolExecutor::with_safety_and_approval(SafetyContext::default(), gate.clone());

        let handle = {
            let exec = exec.clone();
            tokio::spawn(async move {
                exec.execute_tool("shell_exec", serde_json::json!({"command": "echo hi"}))
                    .await
            })
        };
        let req = rx.recv().await.expect("approval notice delivered");
        assert_eq!(req.tool_name, "shell_exec");
        gate.resolve(&req.call_id, false); // user denies
        let res = handle.await.unwrap().unwrap();
        assert!(!res.is_success());
        assert!(
            res.error_message().unwrap().contains("denied by user approval"),
            "denied tool must surface the approval-denied error"
        );
    }

    #[tokio::test]
    async fn approval_gate_runs_shell_exec_when_approved() {
        let (gate, mut rx) = ApprovalGate::new();
        let gate = Arc::new(gate);
        let exec = McpToolExecutor::with_safety_and_approval(SafetyContext::default(), gate.clone());

        let handle = {
            let exec = exec.clone();
            tokio::spawn(async move {
                exec.execute_tool("shell_exec", serde_json::json!({"command": "echo hi"}))
                    .await
            })
        };
        let req = rx.recv().await.expect("approval notice delivered");
        gate.resolve(&req.call_id, true); // user approves
        let res = handle.await.unwrap().unwrap();
        assert!(res.is_success());
        assert!(res.content().contains("hi"));
    }

    #[tokio::test]
    async fn auto_approved_tool_skips_gate() {
        let (gate, mut rx) = ApprovalGate::new();
        gate.set_auto_approved("shell_exec");
        let gate = Arc::new(gate);
        let exec = McpToolExecutor::with_safety_and_approval(SafetyContext::default(), gate.clone());
        // No approval notice should be sent; tool runs immediately.
        let res = exec
            .execute_tool("shell_exec", serde_json::json!({"command": "echo auto"}))
            .await
            .unwrap();
        assert!(res.is_success());
        assert!(res.content().contains("auto"));
        // The host receiver must still be empty (no pause happened).
        assert!(rx.try_recv().is_err());
    }
}
