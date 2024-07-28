use std::path::PathBuf;
use tauri::AppHandle;

pub fn get_error_icon(app: &AppHandle) -> PathBuf {
    app.path_resolver()
        .resolve_resource("icons/tray/glucmon_icon_NOT-WORKING.png")
        .unwrap()
}
