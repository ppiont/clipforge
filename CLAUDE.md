# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**ClipForge** is a desktop video editor built with Tauri and Svelte. The application features a compact recorder interface for capturing screen, webcam, or both simultaneously, with a full-sized editor window for trimming, arranging, and exporting video clips.

- **Primary Platform**: macOS
- **Secondary Platform**: Windows
- **Tech Stack**: Tauri (Rust) + Svelte + FFmpeg
- **Status**: MVP deadline Tuesday, full submission Wednesday

## Key Features

### Recording Module
- Separate floating recorder window (400x300px, always-on-top)
- Three recording modes:
  - Screen capture (with audio)
  - Webcam recording (with audio)
  - Simultaneous screen + webcam with picture-in-picture overlay
- Duration timer, red dot indicator, record/stop/save controls
- Auto-saves and auto-imports recordings to timeline

### Editing Features
- Two-track timeline for main video + overlay
- Drag clips from media library to timeline
- Trim (drag clip edges), split (at playhead), delete, and arrange clips
- Horizontal timeline with time ruler and playhead
- Zoom in/out, snap to clip edges
- HTML5 video preview (640x360 or 854x480, 16:9 aspect ratio)
- Playback controls: play/pause (spacebar), scrubbing, stop

### Import & Export
- Import via file picker (MP4, MOV, WebM) or drag-and-drop
- FFmpeg-based export to MP4 (H.264 video, AAC audio)
- Export dialog with resolution selection (720p, 1080p, Source)
- Progress bar with cancel button during export
- Multi-clip composition and overlay support

## UI Layout

### Main Editor Window (1200x800px)
```
Top Bar (50px):
- App title, Import button, Record button, Export button

Main Area (600px):
- Left (70%): Video preview player
- Right (30%): Media library (scrollable list)

Timeline Section (150px):
- Two-track timeline with playhead and time ruler

Control Bar (50px):
- Playback controls, split/delete buttons, zoom controls
```

### Recording Window (400x300px)
```
Status Bar (40px):
- Recording indicator, duration timer

Preview Area (180px):
- Webcam feed preview (when applicable)

Source Selector (40px):
- Radio buttons: Screen / Webcam / Both

Action Buttons (40px):
- Stop and Cancel buttons
```

## Technology Stack

- **Frontend**: Svelte 5.0.0 + Vite 6.0.3 (with SvelteKit adapter-static)
- **Backend**: Rust with Tauri 2.0
- **Build Tool**: FFmpeg for video processing
- **Package Manager**: Bun
- **TypeScript**: Strict mode enabled

## Frontend Architecture

```
src/
├── App.svelte (main entry point)
├── views/
│   ├── Editor.svelte (main editor)
│   └── Recorder.svelte (recording window)
├── components/
│   ├── TopBar.svelte
│   ├── Preview.svelte
│   ├── MediaLibrary.svelte
│   ├── Timeline.svelte
│   ├── Controls.svelte
│   └── ExportModal.svelte
└── stores/
    ├── clips.js (media library state)
    ├── timeline.js (timeline clips)
    ├── playback.js (playhead, play/pause)
    └── recording.js (recording state)
```

## Backend Architecture (Rust/Tauri)

```
src-tauri/src/
├── main.rs (entry point)
├── lib.rs (Tauri commands)
└── commands.rs (command implementations)

Key Tauri Commands:
- import_video(path) → VideoMetadata
- save_recording(blob, filename) → filepath
- export_video(clips, output) → Result
- open_recorder_window(app)
- close_recorder_window(app)
- get_ffmpeg_path() → String
```

### Window Management
Create recorder window via Tauri:
```rust
#[tauri::command]
fn open_recorder_window(app: tauri::AppHandle) {
    tauri::WindowBuilder::new(&app, "recorder",
        tauri::WindowUrl::App("/recorder".into()))
        .title("Record")
        .inner_size(400.0, 300.0)
        .always_on_top(true)
        .resizable(false)
        .build()
        .unwrap();
}
```

## Data Models

```javascript
// Clip in media library
{
  id: string,
  filename: string,
  path: string,
  duration: number,
  resolution: string
}

// Clip on timeline
{
  id: string,
  clipId: string,
  track: number (0 or 1),
  startTime: number,
  trimStart: number,
  trimEnd: number,
  duration: number
}

// Recording state
{
  isRecording: boolean,
  source: 'screen' | 'webcam' | 'both',
  startTime: number,
  stream: MediaStream
}
```

## FFmpeg Integration

### Common Export Commands

**Single clip with trim:**
```bash
ffmpeg -i input.mp4 -ss [trimStart] -t [duration] -c copy output.mp4
```

**Multiple clips (concatenation):**
```bash
ffmpeg -f concat -safe 0 -i filelist.txt -c copy output.mp4
```

**Picture-in-picture overlay (Track 2 over Track 1):**
```bash
ffmpeg -i track1.mp4 -i track2.mp4 -filter_complex \
  "[1:v]scale=320:240[pip];[0:v][pip]overlay=W-w-10:H-h-10" \
  output.mp4
```

### Export Flow
1. Analyze timeline state (clips, tracks, positions)
2. Generate temporary files for trimmed segments
3. Create FFmpeg command for concatenation/overlay
4. Execute via Tauri shell command
5. Monitor progress (parse FFmpeg stderr)
6. Update progress bar in UI
7. Notify user on completion
8. Clean up temporary files

## Recording Implementation

### Screen Recording (JavaScript)
- Use `navigator.mediaDevices.getDisplayMedia()` with `cursor: "always"`
- Set audio with echo cancellation and noise suppression
- MediaRecorder with VP9 codec at 2.5Mbps
- Save as WebM, convert via FFmpeg if needed

### Webcam Recording
- Use `navigator.mediaDevices.getUserMedia()` with ideal 1280x720
- Include audio track
- Live preview via `srcObject` assignment
- Same MediaRecorder setup as screen

### Combined Screen + Webcam (PiP)
- Get both streams simultaneously
- Use Canvas API to composite:
  - Draw screen to full canvas
  - Draw webcam as 320x240 in bottom-right (with 20px margin)
- Use `canvas.captureStream(30)` for composite video
- Add audio from screen stream
- Record composite stream to file

## Common Development Commands

```bash
# Development
bun run dev                 # Start Tauri dev server with hot reload

# Building
bun run build              # Build frontend assets
bun run tauri build        # Full app build (Rust + frontend)

# Type checking
bun run check              # Check types and lint
bun run check:watch        # Watch mode

# Running Tauri commands
bun run tauri dev          # Run in dev mode (alternative)
```

## Frontend-Backend Communication

Define commands in Rust and invoke from frontend:

```rust
#[tauri::command]
fn my_command(arg: String) -> String {
    format!("Result: {}", arg)
}
```

```svelte
<script>
  import { invoke } from "@tauri-apps/api/core";

  async function handleClick() {
    const result = await invoke("my_command", { arg: "value" });
  }
</script>
```

## Key Implementation Notes

### Timeline State Management
- Use stores to track playhead position, selected clip, clip arrangements
- Timeline clips reference media library clips by ID
- Support undo/redo considerations for future (not MVP)

### Media Handling
- Load video metadata (duration, resolution) on import
- Store actual files on disk, track paths in state
- Handle WebM recordings + MP4 files in same timeline

### Recording Permissions
- Screen recording requires system permissions (getDisplayMedia prompts)
- Webcam requires user media permission
- Gracefully handle permission denials

### Performance Considerations
- Preview player should be responsive at 1080p
- Timeline with 10+ clips should remain snappy
- Canvas rendering for PiP at 30fps
- Export should not block UI (run in background)

## Risk Areas & Fallback Plans

**High-Risk:**
1. FFmpeg multi-clip export - Complex command generation
2. Recording window management - Tauri multi-window coordination
3. PiP compositing - Canvas rendering performance
4. Timeline drag/drop - Complex interaction state

**Fallbacks:**
- If PiP too complex: Skip simultaneous recording, do separate tracks only
- If multi-window fails: Embed recorder in main window
- If drag/drop problematic: Use buttons for clip operations
- If Windows build fails: Submit macOS only
- If export crashes: Simplify to single-track only

## MVP Checklist (Tuesday)
- [ ] App launches (packaged .dmg)
- [ ] Import MP4/MOV file
- [ ] Video appears in media library
- [ ] Click clip loads in preview
- [ ] Add clip to timeline
- [ ] Trim clip by dragging edges
- [ ] Export to MP4
- [ ] Exported video plays correctly

## Testing Priorities

Focus testing on:
1. **Recording**: All three modes work, permissions handled
2. **Editing**: Clips arrange correctly, trim produces correct output
3. **Export**: Multi-clip timeline exports with correct sequence and audio
4. **Performance**: No crashes with 10+ clips, export doesn't hang
5. **Packaging**: .dmg installer launches on fresh macOS

## Configuration Files

- `package.json` - Dependencies and scripts
- `vite.config.js` - Vite + Tauri-specific settings
- `svelte.config.js` - Adapter-static with SPA fallback
- `jsconfig.json` - TypeScript/JavaScript configuration
- `src-tauri/Cargo.toml` - Rust dependencies
- `src-tauri/tauri.conf.json` - Tauri window/app configuration

## Tauri-Specific Notes

- SvelteKit uses adapter-static (no Node.js server in Tauri)
- IPC via `invoke()` for Rust ↔ JavaScript communication
- Vite watches exclude `src-tauri` to avoid rebuild loops
- Multi-window management requires AppHandle in commands
- Recording requires web APIs (getDisplayMedia, getUserMedia)
- Canvas rendering requires web context (can't do in Rust)
