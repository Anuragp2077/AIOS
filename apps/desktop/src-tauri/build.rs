fn main() {
    const COMMANDS: &[&str] = &[
        "get_runtime_status",
        "start_runtime",
        "stop_runtime",
        "create_process",
        "start_process",
        "pause_process",
        "resume_process",
        "stop_process",
        "get_projects",
        "register_project",
        "remove_project",
        "set_active_project",
        "refresh_projects",
        "get_models",
        "register_model_provider",
        "remove_model_provider",
        "register_model",
        "remove_model",
        "set_model_default",
        "clear_model_default",
    ];

    tauri_build::try_build(
        tauri_build::Attributes::new().app_manifest(tauri_build::AppManifest::new().commands(COMMANDS)),
    )
    .expect("failed to run tauri-build");
}
