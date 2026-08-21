//! DoDidDoneUi desktop shell.
//!
//! Startup order: single-instance guard → spawn backend sidecar → wait for
//! HTTP readiness → create the main window with `window.__backendPort`
//! injected (the frontend's httpBridge picks desktop mode from that flag).

mod port;
mod sidecar;
mod tray;

use sidecar::Backend;
use tauri::{Manager, RunEvent, TitleBarStyle, WebviewUrl, WebviewWindowBuilder};

/// Frontend-triggered app quit (the “exit DoDidDoneUi” menu item).
/// `app.exit` fires `RunEvent::Exit`, which shuts the sidecar down cleanly —
/// unlike POSTing /api/system/shutdown, which the watchdog would restart.
#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

pub fn run() {
    let app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![quit_app])
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let backend = Backend::new();
            let port = backend.start(app.handle())?;
            app.manage(backend);

            tray::setup(app.handle())?;

            // Overlay keeps the macOS traffic lights while letting the
            // frontend-drawn titlebar sit under them. Title must stay empty:
            // with Overlay macOS renders the window title natively next to the
            // traffic lights, crowding the frontend's titlebar buttons.
            let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
                .title("")
                .inner_size(1280.0, 832.0)
                .min_inner_size(900.0, 600.0)
                .title_bar_style(TitleBarStyle::Overlay)
                .initialization_script(&format!("window.__backendPort = {port};"))
                .build()?;
            let _ = window.set_focus();
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("failed to build DoDidDoneUi desktop app");

    app.run(|app_handle, event| {
        if let RunEvent::Exit = event {
            if let Some(backend) = app_handle.try_state::<Backend>() {
                backend.shutdown();
            }
        }
    });
}
