use std::path::PathBuf;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Opens a file picker dialog for video files
/// Returns the selected file path or None if cancelled
#[tauri::command]
async fn pick_video_file() -> Result<Option<PathBuf>, String> {
    use tauri::api::dialog;

    let file_path = dialog::FileDialogBuilder::new()
        .add_filter("Video Files", &["mp4", "mov", "webm"])
        .add_filter("MP4", &["mp4"])
        .add_filter("MOV", &["mov"])
        .add_filter("WebM", &["webm"])
        .add_filter("All Files", &["*"])
        .pick_file()
        .await
        .map_err(|e| e.to_string())?;

    Ok(file_path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, pick_video_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
