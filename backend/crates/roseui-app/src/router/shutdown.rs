//! Graceful shutdown endpoint for the RoseUi backend.
//!
//! `POST /api/system/shutdown` stops the `dodiddoneui` process so the user can
//! fully exit RoseUi from the web UI. In a production build the frontend is
//! embedded inside `dodiddoneui`, so stopping the process tears down the entire
//! app. In a dev build the separate `vite` dev server keeps running and must
//! be stopped independently.
//!
//! Safety: the handler only fires after a lightweight same-origin guard
//! (`X-Requested-With: roseui`). Local single-operator deployments bind to
//! 127.0.0.1, so this is not reachable from the network.

use axum::http::StatusCode;
use axum::response::IntoResponse;

/// Body-less response type for the shutdown acknowledgement.
#[derive(serde::Serialize)]
pub(super) struct ShutdownResponse {
    pub status: &'static str,
}

/// Request guard: only allow same-origin calls carrying the RoseUi marker
/// header. This blocks accidental cross-site triggers while staying usable
/// from the web frontend's fetch helper.
fn is_authorized_request(headers: &axum::http::HeaderMap) -> bool {
    headers
        .get("x-requested-with")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.eq_ignore_ascii_case("roseui"))
}

/// `POST /api/system/shutdown`
///
/// Acknowledges with `200 OK` first, then exits the process on a short delay
/// so the HTTP response is flushed before the runtime tears down.
pub(super) async fn shutdown(headers: axum::http::HeaderMap) -> impl IntoResponse {
    if !is_authorized_request(&headers) {
        return (
            StatusCode::FORBIDDEN,
            axum::Json(ShutdownResponse { status: "forbidden" }),
        )
            .into_response();
    }

    // Respond first, then exit so the client receives the 200 before the
    // connection drops.
    tokio::spawn(async {
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        tracing::info!("shutdown requested via web UI; exiting dodiddoneui");
        std::process::exit(0);
    });

    (
        StatusCode::OK,
        axum::Json(ShutdownResponse {
            status: "shutting_down",
        }),
    )
        .into_response()
}
