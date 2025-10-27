use std::path::PathBuf;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Opens a file picker dialog for video files
/// Returns the selected file path or None if cancelled
///
/// Note: Tauri dialog API requires feature flags. For MVP, this is a stub
/// that will be implemented with proper dialog plugin in next phase.
#[tauri::command]
async fn pick_video_file() -> Result<Option<PathBuf>, String> {
    // TODO: Implement with tauri-plugin-dialog in phase 2
    // For now, return None to indicate no file selected
    Ok(None)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, pick_video_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
