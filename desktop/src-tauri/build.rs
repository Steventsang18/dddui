fn main() {
    // 声明应用级命令，生成 allow-quit-app 权限供 capabilities 引用
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(&["quit_app"])),
    )
    .expect("failed to run tauri-build");
}
