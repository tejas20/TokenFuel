fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "read_config",
            "save_settings",
            "save_account",
            "remove_account",
            "set_manual",
            "open_provider",
            "refresh",
            "snap_window",
        ]),
    ))
    .expect("TokenFuel capability generation failed")
}
