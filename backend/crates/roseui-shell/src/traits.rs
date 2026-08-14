use std::sync::Arc;

use roseui_api_types::ToolType;

use crate::error::ShellError;

/// Abstraction over the shell / OS-integration service.
///
/// Defining the public surface as a trait lets the router state and any future
/// caller depend on `Arc<dyn IShellService>` instead of the concrete
/// `ShellService`. This is the same seam pattern already used by `roseui-file`
/// (`IFileService` + `FileServiceRef`): it keeps the concrete `new(...)` at the
/// app assembly point while the rest of the code only sees the capability.
///
/// A future `SandboxedShellService` (e.g. for containerized command execution,
/// see the harness comparison report §4.4) can implement this trait without
/// touching any caller.
#[async_trait::async_trait]
pub trait IShellService: Send + Sync {
    /// Open a file with the OS-registered handler.
    async fn open_file(&self, file_path: &str) -> Result<(), ShellError>;

    /// Reveal a file/dir in the system file manager (highlighting it).
    async fn show_item_in_folder(&self, file_path: &str) -> Result<(), ShellError>;

    /// Open an external URL (http/https/mailto only).
    async fn open_external(&self, url: &str) -> Result<(), ShellError>;

    /// Check whether a known external tool (Terminal / Explorer / VSCode) is
    /// available on this host.
    async fn check_tool_installed(&self, tool: ToolType) -> bool;

    /// Open a folder with a specific tool (VSCode / Terminal / Explorer).
    async fn open_folder_with(&self, folder_path: &str, tool: ToolType) -> Result<(), ShellError>;
}

/// Shared, reference-counted handle to a shell service capability.
pub type ShellServiceRef = Arc<dyn IShellService>;
