//! Build script for `roseui-webhost`.
//!
//! rust-embed 在编译期把 `assets/` 目录内嵌进 `dodiddoneui`。本脚本在前端
//! `vite build` 产物（dist/）存在时，将其同步进 `assets/`，确保每次编译
//! 嵌入的都是最新前端。这样 `cargo build -p roseui-app` 产出的单二进制
//! 始终携带当时最新的前端，无需手动拷贝。
//!
//! 行为：
//! - `ROSEUI_FRONTEND_DIST` 环境变量指向的目录优先；
//! - 否则回退到仓库相对路径
//!   `frontend/packages/desktop/src/renderer/dist`
//!   （vite `root` 设为 `packages/desktop/src/renderer`，默认 outDir 即其下 `dist`）。
//! - dist 存在 → 清旧 assets 并拷贝 dist 全部内容。
//! - dist 不存在但 assets 已有内容 → 保留旧前端，仅告警（兼容手动/CI 已拷状态）。
//! - 两者皆空 → 构建失败并提示先 `vite build`，避免产出"嵌入空前端"的二进制。

use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set by cargo"),
    );
    let assets_dir = manifest_dir.join("assets");

    // 1) 解析前端 dist 来源
    let dist_dir: Option<PathBuf> = if let Ok(env_dist) = std::env::var("ROSEUI_FRONTEND_DIST") {
        let p = PathBuf::from(env_dist);
        if p.exists() {
            Some(p)
        } else {
            println!("cargo:warning=ROSEUI_FRONTEND_DIST={} does not exist, ignoring", p.display());
            None
        }
    } else {
        // 从本 crate 绝对目录向上定位仓库根：crates/roseui-webhost -> crates ->
        // upstream-AionCore -> 仓库根(rust-aion-ui)。用绝对祖先避免 CWD 歧义。
        // vite root = packages/desktop/src/renderer，故默认 outDir 为
        // packages/desktop/src/renderer/dist（非 packages/desktop/dist）。
        let repo_root = manifest_dir
            .ancestors()
            .nth(3)
            .expect("failed to resolve repo root from CARGO_MANIFEST_DIR")
            .to_path_buf();
        let default_dist = repo_root.join("frontend/packages/desktop/src/renderer/dist");
        if default_dist.exists() {
            Some(default_dist)
        } else {
            None
        }
    };

    match dist_dir {
        Some(dist) => {
            sync_dist_into_assets(&dist, &assets_dir);
            println!("cargo:rerun-if-changed={}", dist.join("index.html").display());
        }
        None => {
            let assets_has_content =
                assets_dir.exists() && fs::read_dir(&assets_dir).map(|mut it| it.next().is_some()).unwrap_or(false);
            if assets_has_content {
                println!(
                    "cargo:warning=frontend dist not found; keeping existing assets/ (stale frontend may be embedded)"
                );
            } else {
                panic!(
                    "No frontend dist found and assets/ is empty. Run `cd frontend && npm run build` first, \
                     or set ROSEUI_FRONTEND_DIST to the built dist directory."
                );
            }
        }
    }

    // 监控本脚本与 assets 目录变化，确保 cargo 重编
    println!("cargo:rerun-if-changed=build.rs");
    if assets_dir.exists() {
        println!("cargo:rerun-if-changed={}", assets_dir.display());
    }
}

/// 将 dist 内容同步进 assets：先清空 assets（保留目录本身），再拷贝。
fn sync_dist_into_assets(dist: &Path, assets: &Path) {
    if assets.exists()
        && let Err(e) = fs::remove_dir_all(assets)
    {
        println!("cargo:warning=failed to clear assets/: {e}");
    }
    if let Err(e) = fs::create_dir_all(assets) {
        panic!("failed to create assets/ dir: {e}");
    }

    copy_dir_recursive(dist, assets);
    println!("cargo:warning=synced frontend dist ({}) into assets/", dist.display());
}

fn copy_dir_recursive(from: &Path, to: &Path) {
    for entry in fs::read_dir(from).expect("read_dir failed") {
        let entry = entry.expect("dir entry failed");
        let path = entry.path();
        let name = entry.file_name();
        let target = to.join(name);
        if path.is_dir() {
            fs::create_dir_all(&target).expect("create_dir failed");
            copy_dir_recursive(&path, &target);
        } else {
            fs::copy(&path, &target).expect("copy failed");
        }
    }
}
