//! Application configuration parsed from CLI arguments + key derivation.

use std::path::PathBuf;

use sha2::{Digest, Sha256};
use tracing::{info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityMode {
    /// Single-owner local use: skip authentication, inject `system_default_user`.
    /// This is the default for the pure-Web build — the platform is a personal
    /// single-operator tool, so it opens directly with no login wall.
    Local,
    /// Alias of `Local` with explicit naming: single-owner personal use.
    /// Distinct variant so future team/multi-account modes can be added without
    /// overloading `Local`. Auth-wise equivalent to `Local`.
    Owner,
    /// Web UI mode requiring username/password login (legacy Electron path).
    WebUi,
    /// RosePro managed identity mode (requires `AIONCORE_BOOTSTRAP_SECRET`).
    RosePro,
}

impl IdentityMode {
    pub fn auth_label(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Owner => "owner",
            Self::WebUi => "webui",
            Self::RosePro => "aionpro",
        }
    }

    /// Whether authentication is skipped and `system_default_user` is injected.
    /// Both `Local` and `Owner` are unauthenticated single-operator modes.
    pub fn is_local(self) -> bool {
        matches!(self, Self::Local | Self::Owner)
    }
}

/// Application configuration parsed from CLI arguments.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub data_dir: PathBuf,
    pub work_dir: PathBuf,
    pub app_version: String,
    /// Run in local embedded mode (skip authentication, use system_default_user).
    pub local: bool,
    pub identity_mode: IdentityMode,
    pub bootstrap_secret: Option<String>,
    /// Dump prompt diagnostics under `data_dir/prompt-dumps`.
    pub dump_prompts: bool,
    /// Explicitly authorize backup and rebuild for corruption-like local databases.
    pub recover_corrupted_database: bool,
    /// Optional external static directory to serve instead of the embedded
    /// frontend build (used for local development against a live `vite build`).
    pub static_dir: Option<PathBuf>,
    /// When `true`, do not open the default browser after startup.
    pub no_open: bool,
}

impl AppConfig {
    pub fn effective_identity_mode(&self) -> IdentityMode {
        if self.local {
            IdentityMode::Local
        } else {
            self.identity_mode
        }
    }

    /// Format as `host:port` for socket binding.
    pub fn socket_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Local URL helpers should use to call this backend from the same machine.
    pub fn local_base_url(&self) -> String {
        let host = match self.host.as_str() {
            "0.0.0.0" | "::" => "127.0.0.1",
            other => other,
        };
        format!("http://{host}:{}", self.port)
    }

    /// Path to the SQLite database file.
    pub fn database_path(&self) -> PathBuf {
        self.data_dir.join("dodiddoneui-backend.db")
    }

    /// Resolve the database path, migrating the legacy `roseui-backend.db` to the
    /// current `dodiddoneui-backend.db` name if the new file is missing but the old
    /// one exists. This preserves existing local data (provider config, history)
    /// across the rename without manual intervention.
    pub fn resolve_database_path(&self) -> PathBuf {
        let new_path = self.database_path();
        if new_path.exists() {
            return new_path;
        }
        let legacy = self.data_dir.join("roseui-backend.db");
        if legacy.exists() {
            if let Err(e) = std::fs::copy(&legacy, &new_path) {
                warn!(
                    legacy = %legacy.display(),
                    target = %new_path.display(),
                    error = %e,
                    "Failed to migrate legacy database; starting with a fresh database"
                );
            } else {
                info!(
                    legacy = %legacy.display(),
                    target = %new_path.display(),
                    "Migrated legacy database to the new filename"
                );
            }
        }
        new_path
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            host: roseui_common::constants::DEFAULT_HOST.to_string(),
            port: roseui_common::constants::DEFAULT_PORT,
            data_dir: PathBuf::from("data"),
            work_dir: PathBuf::from("data"),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            local: false,
            identity_mode: IdentityMode::Owner,
            bootstrap_secret: None,
            dump_prompts: false,
            recover_corrupted_database: false,
            static_dir: None,
            no_open: false,
        }
    }
}

/// Derive a 32-byte encryption key from the JWT secret using SHA-256.
pub fn derive_encryption_key(jwt_secret: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"roseui-encryption-key:");
    hasher.update(jwt_secret.as_bytes());
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 25808);
        assert_eq!(config.data_dir, PathBuf::from("data"));
        assert_eq!(config.app_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(config.identity_mode, IdentityMode::Owner);
        assert!(config.bootstrap_secret.is_none());
        assert!(!config.dump_prompts);
        assert!(!config.recover_corrupted_database);
    }

    #[test]
    fn test_app_config_socket_addr() {
        let config = AppConfig {
            host: "0.0.0.0".to_string(),
            port: 3000,
            ..Default::default()
        };
        assert_eq!(config.socket_addr(), "0.0.0.0:3000");
    }

    #[test]
    fn local_base_url_uses_loopback_for_wildcard_host() {
        let config = AppConfig {
            host: "0.0.0.0".to_string(),
            port: 49152,
            ..Default::default()
        };
        assert_eq!(config.local_base_url(), "http://127.0.0.1:49152");
    }

    #[test]
    fn test_app_config_database_path() {
        let config = AppConfig {
            data_dir: PathBuf::from("/tmp/roseui"),
            ..Default::default()
        };
        assert_eq!(config.database_path(), PathBuf::from("/tmp/roseui/dodiddoneui-backend.db"));
    }
}
