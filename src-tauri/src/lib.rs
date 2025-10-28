use ffmpeg_next as ffmpeg;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoMetadata {
    pub filename: String,
    pub path: String,
    pub duration: f64,
    pub resolution: String,
    pub codec: String,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Extracts video metadata using FFmpeg Rust bindings
/// Returns duration (in seconds), resolution (WxH), and codec info
fn extract_video_metadata(file_path: &str) -> Result<VideoMetadata, String> {
    // Open the file with FFmpeg
    let input = ffmpeg::format::input(&file_path)
        .map_err(|e| format!("Failed to open video file: {}", e))?;

    // Get duration
    let duration = input.duration() as f64 / ffmpeg::ffi::AV_TIME_BASE as f64;

    // Find the video stream
    let stream = input
        .streams()
        .best(ffmpeg::media::Type::Video)
        .ok_or_else(|| "No video stream found in file".to_string())?;

    // Get codec context from stream parameters
    let codec = ffmpeg::codec::context::Context::from_parameters(stream.parameters())
        .map_err(|e| format!("Failed to get codec context: {}", e))?;

    // Get codec name before consuming codec
    let codec_name = codec
        .codec()
        .map(|c| c.name().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Get video decoder to access video properties (this consumes codec)
    let video = codec
        .decoder()
        .video()
        .map_err(|e| format!("Failed to get video decoder: {}", e))?;

    // Get resolution
    let width = video.width();
    let height = video.height();

    let file_name = PathBuf::from(file_path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    Ok(VideoMetadata {
        filename: file_name,
        path: file_path.to_string(),
        duration,
        resolution: format!("{}x{}", width, height),
        codec: codec_name,
    })
}

/// Opens a file picker dialog for video files
/// Returns VideoMetadata with file info and duration/resolution
#[tauri::command]
fn pick_video_file() -> Result<Option<VideoMetadata>, String> {
    // Use rfd file dialog (blocking is fine for this operation)
    let file_path = rfd::FileDialog::new()
        .add_filter("Video Files", &["mp4", "mov", "webm", "mkv", "avi"])
        .pick_file();

    if let Some(path) = file_path {
        let path_str = path.to_string_lossy().to_string();
        let metadata = extract_video_metadata(&path_str)?;
        Ok(Some(metadata))
    } else {
        Ok(None)
    }
}

/// Extracts metadata from a video file at the given path
/// Used for drag-and-drop file imports
#[tauri::command]
fn pick_video_file_by_path(path: String) -> Result<VideoMetadata, String> {
    extract_video_metadata(&path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize FFmpeg
    ffmpeg::init().expect("Failed to initialize FFmpeg");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, pick_video_file, pick_video_file_by_path])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
