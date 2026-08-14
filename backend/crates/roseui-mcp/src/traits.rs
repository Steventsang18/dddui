use std::sync::Arc;

use roseui_api_types::{
    BatchImportMcpServersRequest, CreateMcpServerRequest, DetectedMcpServerResponse, McpConnectionTestResult,
    McpServerResponse, UpdateMcpServerRequest,
};

use crate::error::McpError;

/// Abstraction over MCP server configuration CRUD.
///
/// Mirrors the seam pattern from `roseui-file` (`IFileService` + `FileServiceRef`)
/// and `roseui-shell` (`IShellService` + `ShellServiceRef`): callers depend on
/// `Arc<dyn IMcpConfigService>` instead of the concrete `McpConfigService`, so a
/// future alternate implementation (e.g. config backed by a remote store) can be
/// swapped in at the app assembly point without touching route handlers.
#[async_trait::async_trait]
pub trait IMcpConfigService: Send + Sync {
    async fn list_servers(&self, user_id: &str) -> Result<Vec<McpServerResponse>, McpError>;
    async fn get_server(&self, user_id: &str, id: &str) -> Result<McpServerResponse, McpError>;
    async fn add_server(
        &self,
        user_id: &str,
        req: CreateMcpServerRequest,
    ) -> Result<McpServerResponse, McpError>;
    async fn edit_server(
        &self,
        user_id: &str,
        id: &str,
        req: UpdateMcpServerRequest,
    ) -> Result<McpServerResponse, McpError>;
    async fn delete_server(&self, user_id: &str, id: &str) -> Result<bool, McpError>;
    async fn toggle_server(&self, user_id: &str, id: &str) -> Result<McpServerResponse, McpError>;
    async fn batch_import(
        &self,
        user_id: &str,
        req: BatchImportMcpServersRequest,
    ) -> Result<Vec<McpServerResponse>, McpError>;
    async fn persist_test_result(
        &self,
        user_id: &str,
        id: &str,
        result: &McpConnectionTestResult,
    ) -> Result<(), McpError>;
}

/// Abstraction over multi-agent MCP sync.
#[async_trait::async_trait]
pub trait IMcpSyncService: Send + Sync {
    async fn get_agent_configs(
        &self,
        user_id: &str,
    ) -> Result<Vec<DetectedMcpServerResponse>, McpError>;
}

/// Shared, reference-counted handles to MCP service capabilities.
pub type McpConfigServiceRef = Arc<dyn IMcpConfigService>;
pub type McpSyncServiceRef = Arc<dyn IMcpSyncService>;
