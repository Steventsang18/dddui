//! `roseui-webhost` — 单二进制 SPA 静态托管层。
//!
//! 把前端 `vite build` 产物（dist/）在编译期经 `rust-embed` 内嵌进 `dodiddoneui`
//! 二进制，运行期同源回话给浏览器。支持：
//! - 内嵌静态资源服务（带 content-type / ETag / Cache-Control）
//! - SPA history fallback：未匹配静态文件的非 API 路径统一回 `index.html`
//! - `--static <dir>` 覆盖：开发/调试期跳过内嵌，直接伺服指定目录的磁盘文件
//!
//! 安全：本路由只处理前端资源与 history fallback，**绝不**截获 `/api`、`/ws`、
//! `/health`——这些路由在 `routes.rs` 中先于本路由注册，axum 精确匹配优先于
//! `.fallback()`，因此 API/WS 不受影响。

use std::path::PathBuf;

use axum::Router;
use axum::body::Body;
use axum::extract::Request;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use rust_embed::RustEmbed;
use sha2::{Digest, Sha256};

/// 编译期内嵌的前端构建产物目录（由构建脚本从前端 dist 拷入）。
///
/// `rust-embed` 的 `compression` feature 已在 crate 依赖中启用，编译期会把
/// dist 资源 gzip 后存入二进制，运行期 `get()` 自动解压返回原始 bytes（对调用
/// 方透明）。前端 22MB+ 资源经 gzip 内嵌可将二进制体积再压掉约 15MB，且不改变
/// 分发形态（仍是单二进制）。
#[derive(RustEmbed)]
#[folder = "assets/"]
struct FrontendAssets;

const CACHE_CONTROL_IMMUTABLE: &str = "public, max-age=31536000, immutable";
const CACHE_CONTROL_NO_CACHE: &str = "no-cache";

/// 构建前端托管路由。
///
/// - `static_dir = Some(path)`：直接伺服该目录的磁盘文件（覆盖内嵌，便于开发）。
/// - `static_dir = None`：使用编译期内嵌的前端产物。
pub fn webhost_routes(static_dir: Option<PathBuf>) -> Router {
    match static_dir {
        Some(dir) => Router::new().fallback(move |req: Request| serve_disk_spa(dir.clone(), req)),
        None => Router::new().fallback(serve_embedded_spa),
    }
}

/// 未注册的 `/api/*` 请求统一返回 404，避免被 SPA fallback 吞成 index.html。
/// API 客户端依赖 404 判断"路由不存在"；SPA 前端路由（非 /api）不受影响。
fn is_api_path(path: &str) -> bool {
    path == "/api" || path.starts_with("/api/")
}

fn content_type_for_path(path: &str) -> HeaderValue {
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    HeaderValue::from_str(mime.as_ref()).unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"))
}

fn build_etag(bytes: &[u8]) -> HeaderValue {
    let digest = Sha256::digest(bytes);
    HeaderValue::from_str(&format!("\"{digest:x}\"")).unwrap_or_else(|_| HeaderValue::from_static("\"\""))
}

async fn serve_embedded_spa(req: Request) -> Response {
    let path = req.uri().path();
    if is_api_path(path) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let rel = if path.is_empty() {
        "index.html"
    } else {
        path.trim_start_matches('/')
    };
    match FrontendAssets::get(rel) {
        Some(file) => {
            let bytes = file.data.into_owned();
            let etag = build_etag(&bytes);
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, content_type_for_path(rel))
                .header(header::CACHE_CONTROL, CACHE_CONTROL_IMMUTABLE)
                .header(header::ETAG, etag)
                .body(Body::from(bytes))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
        None => serve_index_html().await,
    }
}

/// SPA history fallback：把未匹配静态资源的前端路由统一回 index.html。
async fn serve_index_html() -> Response {
    match FrontendAssets::get("index.html") {
        Some(file) => {
            let bytes = file.data.into_owned();
            Response::builder()
                .status(StatusCode::OK)
                .header(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("text/html; charset=utf-8"),
                )
                .header(header::CACHE_CONTROL, CACHE_CONTROL_NO_CACHE)
                .body(Body::from(bytes))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn serve_disk_spa(dir: PathBuf, req: Request) -> Response {
    let path = req.uri().path();
    if is_api_path(path) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let rel = if path.is_empty() {
        "index.html"
    } else {
        path.trim_start_matches('/')
    };
    let full = dir.join(sanitize(rel));
    match tokio::fs::read(&full).await {
        Ok(bytes) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, content_type_for_path(rel))
            .header(header::CACHE_CONTROL, CACHE_CONTROL_IMMUTABLE)
            .body(Body::from(bytes))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response()),
        Err(_) => serve_disk_index(dir).await,
    }
}

async fn serve_disk_index(dir: PathBuf) -> Response {
    let full = dir.join("index.html");
    match tokio::fs::read(&full).await {
        Ok(bytes) => Response::builder()
            .status(StatusCode::OK)
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            )
            .header(header::CACHE_CONTROL, CACHE_CONTROL_NO_CACHE)
            .body(Body::from(bytes))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response()),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

/// 阻止 `--static` 目录遍历：只允许相对子路径，拒绝 `..` 与绝对分量。
fn sanitize(rel: &str) -> PathBuf {
    let mut out = PathBuf::new();
    for component in std::path::Path::new(rel).components() {
        match component {
            std::path::Component::Normal(value) => out.push(value),
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir | std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                // 拒绝：落到 dir 之外
                return PathBuf::new();
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    // (unused import removed)
    use tower::ServiceExt;

    #[tokio::test]
    async fn serves_index_html_at_root_when_embedded() {
        let router = webhost_routes(None);
        let res = router
            .clone()
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert!(
            res.headers()[header::CONTENT_TYPE]
                .to_str()
                .unwrap()
                .contains("text/html")
        );
    }

    #[tokio::test]
    async fn spa_history_fallback_returns_index_html() {
        let router = webhost_routes(None);
        let res = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/guid/some-deep-route")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert!(
            res.headers()[header::CONTENT_TYPE]
                .to_str()
                .unwrap()
                .contains("text/html")
        );
    }

    #[test]
    fn sanitize_rejects_traversal() {
        assert!(sanitize("../etc/passwd").as_os_str().is_empty());
        assert!(sanitize("/abs/path").as_os_str().is_empty());
        assert_eq!(sanitize("assets/app.png"), PathBuf::from("assets/app.png"));
    }
}
