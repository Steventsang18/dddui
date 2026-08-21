//! Sidecar lifecycle for the `dodiddoneui` backend binary.
//!
//! The shell spawns the bundled backend on a random loopback port in `--local`
//! embedded mode, waits for HTTP readiness, and keeps a watchdog that
//! auto-restarts the backend on unexpected exit (bounded attempts). The
//! backend also receives `--parent-pid` so it self-terminates if this
//! process dies without a clean shutdown.

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Manager};
use tauri_plugin_shell::process::{CommandChild, CommandEvent};
use tauri_plugin_shell::ShellExt;

use crate::port;

const SIDECAR_NAME: &str = "dodiddoneui";
const READY_TIMEOUT: Duration = Duration::from_secs(30);
const READY_POLL_INTERVAL: Duration = Duration::from_millis(150);
const MAX_AUTO_RESTARTS: u8 = 3;

pub struct Backend {
    child: Mutex<Option<CommandChild>>,
    shutting_down: AtomicBool,
    auto_restarts: AtomicU8,
}

impl Backend {
    pub fn new() -> Self {
        Self {
            child: Mutex::new(None),
            shutting_down: AtomicBool::new(false),
            auto_restarts: AtomicU8::new(0),
        }
    }

    /// First launch: spawn the sidecar and block until the HTTP API answers.
    pub fn start(&self, app: &AppHandle) -> Result<u16, String> {
        let port = self.spawn(app)?;
        wait_ready(port)?;
        Ok(port)
    }

    fn spawn(&self, app: &AppHandle) -> Result<u16, String> {
        let port = port::pick_free_port().map_err(|e| format!("no free loopback port: {e}"))?;
        let data_dir = app
            .path()
            .app_data_dir()
            .map_err(|e| format!("cannot resolve app data dir: {e}"))?;
        std::fs::create_dir_all(&data_dir).map_err(|e| format!("cannot create data dir: {e}"))?;

        let args = vec![
            "--local".to_string(),
            "--no-open".to_string(),
            "--host".to_string(),
            "127.0.0.1".to_string(),
            "--port".to_string(),
            port.to_string(),
            "--parent-pid".to_string(),
            std::process::id().to_string(),
            "--data-dir".to_string(),
            data_dir.display().to_string(),
        ];

        let (mut rx, child) = app
            .shell()
            .sidecar(SIDECAR_NAME)
            .map_err(|e| format!("sidecar `{SIDECAR_NAME}` not found in bundle: {e}"))?
            .args(args)
            .spawn()
            .map_err(|e| format!("failed to launch backend: {e}"))?;

        *self.child.lock().unwrap() = Some(child);

        // Watchdog: react to unexpected termination only; a clean shutdown
        // sets `shutting_down` before killing the child.
        let app_handle = app.clone();
        std::thread::spawn(move || {
            while let Some(event) = rx.blocking_recv() {
                if let CommandEvent::Terminated(_) = event {
                    let backend = app_handle.state::<Backend>();
                    if backend.shutting_down.load(Ordering::SeqCst) {
                        break;
                    }
                    eprintln!("[desktop] backend exited unexpectedly, attempting restart");
                    backend.handle_unexpected_exit(&app_handle);
                    break;
                }
            }
        });

        Ok(port)
    }

    fn handle_unexpected_exit(&self, app: &AppHandle) {
        let attempts = self.auto_restarts.fetch_add(1, Ordering::SeqCst);
        if attempts >= MAX_AUTO_RESTARTS {
            eprintln!("[desktop] backend restart limit ({MAX_AUTO_RESTARTS}) reached, giving up");
            return;
        }
        match self.spawn(app).and_then(|port| wait_ready(port).map(|()| port)) {
            Ok(new_port) => {
                // Rebind the injected port and let the frontend reconnect.
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.eval(&format!(
                        "window.__backendPort = {new_port}; window.location.reload();"
                    ));
                }
            }
            Err(error) => eprintln!("[desktop] backend restart failed: {error}"),
        }
    }

    pub fn shutdown(&self) {
        self.shutting_down.store(true, Ordering::SeqCst);
        if let Some(child) = self.child.lock().unwrap().take() {
            let _ = child.kill();
        }
    }
}

fn wait_ready(port: u16) -> Result<(), String> {
    let url = format!("http://127.0.0.1:{port}/api/auth/status");
    let deadline = Instant::now() + READY_TIMEOUT;
    while Instant::now() < deadline {
        if ureq::get(&url).timeout(Duration::from_secs(2)).call().is_ok() {
            return Ok(());
        }
        std::thread::sleep(READY_POLL_INTERVAL);
    }
    Err(format!(
        "backend not ready within {}s ({url})",
        READY_TIMEOUT.as_secs()
    ))
}
