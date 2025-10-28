use ffmpeg_next as ffmpeg;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoMetadata {
    pub filename: String,
    pub path: String,
    pub duration: f64,
    pub resolution: String,
    pub codec: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimelineClip {
    pub id: String,
    pub clip_id: String,
    pub track: u32,
    pub start_time: f64,
    pub trim_start: f64,
    pub trim_end: f64,
    pub duration: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportRequest {
    pub clips: Vec<TimelineClip>,
    pub output_path: String,
    pub resolution: String, // "Source", "720p", "1080p", "1440p", or "4K"
    pub format: String, // "mp4", "webm", or "mov"
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

/// Generate a filmstrip (vertical series of thumbnails) from a video file
/// Returns the file path to the generated filmstrip PNG
#[tauri::command]
fn generate_filmstrip(
    video_path: String,
    clip_id: String,
    frame_count: u32,
) -> Result<String, String> {
    println!(
        "Generating filmstrip for: {} (clip_id: {}, frames: {})",
        video_path, clip_id, frame_count
    );
    use std::fs;
    use std::env;

    // Create persistent cache directory for filmstrips
    let cache_dir = env::temp_dir().join("clipforge_cache").join("filmstrips");
    fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create cache directory: {}", e))?;

    // Generate filename for filmstrip
    let filmstrip_filename = format!("{}_filmstrip.png", clip_id);
    let filmstrip_path = cache_dir.join(&filmstrip_filename);

    // Check if filmstrip already exists (caching)
    if filmstrip_path.exists() {
        println!("Filmstrip already exists, returning cached version");
        return Ok(filmstrip_path.to_string_lossy().to_string());
    }

    // Get video metadata to calculate frame selection interval
    let metadata = extract_video_metadata(&video_path)?;

    // Build FFmpeg command for filmstrip generation
    // Strategy: Extract frames at regular intervals, scale, and tile vertically
    // For a 60fps 10s video (600 total frames) with 20 desired frames:
    // We select every Nth frame to sample evenly across the video
    let select_filter = format!(
        "select='not(mod(n,{}))',scale=120:-2,tile=1x{}",
        // Select every Nth frame (approximate, assuming 30fps baseline)
        ((metadata.duration * 30.0) / frame_count as f64).max(1.0) as i32,
        frame_count
    );

    let output = Command::new("ffmpeg")
        .arg("-y") // Overwrite existing file
        .arg("-i")
        .arg(&video_path) // Input file
        .arg("-vf")
        .arg(&select_filter) // Filter: select frames, scale, tile vertically
        .arg("-frames")
        .arg("1") // Output 1 image (the tiled result)
        .arg(filmstrip_path.to_string_lossy().to_string())
        .output()
        .map_err(|e| format!("Failed to run FFmpeg: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("FFmpeg filmstrip generation failed: {}", stderr));
    }

    println!("Filmstrip generated successfully at: {:?}", filmstrip_path);
    Ok(filmstrip_path.to_string_lossy().to_string())
}

/// Generate a thumbnail image from a video file at a specific timestamp
/// Returns the base64-encoded PNG image data URL
#[tauri::command]
fn generate_thumbnail(video_path: String, timestamp: f64) -> Result<String, String> {
    println!("Generating thumbnail for: {} at {}s", video_path, timestamp);
    use std::fs;
    use std::env;

    // Create temp directory for thumbnails if it doesn't exist
    let temp_dir = env::temp_dir().join("clipforge_thumbnails");
    fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;

    // Generate unique filename for thumbnail
    let thumbnail_filename = format!("thumb_{}_{}.png",
        PathBuf::from(&video_path)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy(),
        timestamp.round() as i64
    );
    let thumbnail_path = temp_dir.join(thumbnail_filename);

    // Use FFmpeg to extract frame at timestamp
    let output = Command::new("ffmpeg")
        .arg("-y") // Overwrite existing file
        .arg("-ss").arg(timestamp.to_string()) // Seek to timestamp
        .arg("-i").arg(&video_path) // Input file
        .arg("-vframes").arg("1") // Extract 1 frame
        .arg("-vf").arg("scale=160:90") // Scale to thumbnail size (16:9 aspect ratio)
        .arg("-q:v").arg("2") // High quality
        .arg(thumbnail_path.to_string_lossy().to_string())
        .output()
        .map_err(|e| format!("Failed to run FFmpeg: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("FFmpeg thumbnail generation failed: {}", stderr));
    }

    // Read the thumbnail file and convert to base64 data URL
    let thumbnail_data = fs::read(&thumbnail_path)
        .map_err(|e| format!("Failed to read thumbnail file: {}", e))?;

    // Convert to base64
    let base64_data = base64_encode(&thumbnail_data);
    let data_url = format!("data:image/png;base64,{}", base64_data);

    // Clean up thumbnail file (optional, could keep for caching)
    let _ = fs::remove_file(&thumbnail_path);

    Ok(data_url)
}

/// Simple base64 encoding function
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();

    for chunk in data.chunks(3) {
        let mut buf = [0u8; 3];
        for (i, &byte) in chunk.iter().enumerate() {
            buf[i] = byte;
        }

        let b1 = (buf[0] >> 2) as usize;
        let b2 = (((buf[0] & 0x03) << 4) | (buf[1] >> 4)) as usize;
        let b3 = (((buf[1] & 0x0f) << 2) | (buf[2] >> 6)) as usize;
        let b4 = (buf[2] & 0x3f) as usize;

        result.push(CHARS[b1] as char);
        result.push(CHARS[b2] as char);
        result.push(if chunk.len() > 1 { CHARS[b3] as char } else { '=' });
        result.push(if chunk.len() > 2 { CHARS[b4] as char } else { '=' });
    }

    result
}

/// Export video timeline to MP4 using FFmpeg
/// For MVP: Simple implementation that handles single clips
/// TODO: Add multi-clip concatenation and overlay support
#[tauri::command]
fn export_video(request: ExportRequest, clips_data: Vec<VideoMetadata>) -> Result<String, String> {
    if request.clips.is_empty() {
        return Err("No clips to export".to_string());
    }

    // For MVP, export first clip only (simplified)
    let timeline_clip = &request.clips[0];

    println!("Exporting clip with path: {}", timeline_clip.clip_id);

    // clip_id now contains the actual file path
    let source_clip = clips_data
        .iter()
        .find(|c| c.path == timeline_clip.clip_id)
        .ok_or_else(|| format!("Source clip not found for path: {}", timeline_clip.clip_id))?;

    // Determine output resolution
    let scale_filter = match request.resolution.as_str() {
        "720p" => Some("scale=1280:720"),
        "1080p" => Some("scale=1920:1080"),
        "1440p" => Some("scale=2560:1440"),
        "4K" => Some("scale=3840:2160"),
        _ => None, // Source resolution
    };

    // Build FFmpeg command
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y") // Overwrite output file
        .arg("-i")
        .arg(&source_clip.path);

    // Add trim if needed
    if timeline_clip.trim_start > 0.0 || timeline_clip.trim_end < source_clip.duration {
        cmd.arg("-ss").arg(timeline_clip.trim_start.to_string());
        let trim_duration = timeline_clip.trim_end - timeline_clip.trim_start;
        cmd.arg("-t").arg(trim_duration.to_string());
    }

    // Add scale filter if needed
    if let Some(scale) = scale_filter {
        cmd.arg("-vf").arg(scale);
    }

    // Output settings based on format
    match request.format.as_str() {
        "webm" => {
            // WebM: VP9 video + Opus audio
            cmd.arg("-c:v")
                .arg("libvpx-vp9")
                .arg("-crf")
                .arg("30") // Quality (0-63, lower = better)
                .arg("-b:v")
                .arg("0") // Use CRF mode
                .arg("-c:a")
                .arg("libopus")
                .arg("-b:a")
                .arg("128k");
        }
        "mov" => {
            // MOV: H.264 video + AAC audio (QuickTime compatible)
            cmd.arg("-c:v")
                .arg("libx264")
                .arg("-preset")
                .arg("medium")
                .arg("-crf")
                .arg("23")
                .arg("-c:a")
                .arg("aac")
                .arg("-b:a")
                .arg("192k");
        }
        _ => {
            // MP4 (default): H.264 video + AAC audio
            cmd.arg("-c:v")
                .arg("libx264")
                .arg("-preset")
                .arg("medium")
                .arg("-crf")
                .arg("23")
                .arg("-c:a")
                .arg("aac")
                .arg("-b:a")
                .arg("192k");
        }
    }

    cmd.arg(&request.output_path);

    println!("Running FFmpeg command: {:?}", cmd);

    // Execute FFmpeg
    let output = cmd.output().map_err(|e| format!("Failed to run FFmpeg: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("FFmpeg export failed: {}", stderr));
    }

    Ok(request.output_path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize FFmpeg
    ffmpeg::init().expect("Failed to initialize FFmpeg");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            pick_video_file,
            pick_video_file_by_path,
            generate_thumbnail,
            generate_filmstrip,
            export_video
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
