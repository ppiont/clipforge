# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**ClipForge** is a desktop video editor built with Tauri and Svelte. The application features a compact recorder interface for capturing screen, webcam, or both simultaneously, with a full-sized editor window for trimming, arranging, and exporting video clips.

- **Primary Platform**: macOS (Apple Silicon - aarch64-apple-darwin)
- **Secondary Platform**: Windows (planned)
- **Tech Stack**: Tauri 2.0 (Rust) + Svelte 5.0 + FFmpeg 8.x (bundled)
- **Status**: Core editor features complete, recorder module pending

## Key Features

### Recording Module
- Separate floating recorder window (400x300px, always-on-top)
- Three recording modes:
  - Screen capture (with audio)
  - Webcam recording (with audio)
  - Simultaneous screen + webcam with picture-in-picture overlay
- Duration timer, red dot indicator, record/stop/save controls
- Auto-saves and auto-imports recordings to timeline

### Editing Features ✅ (Implemented)
- **Two-track timeline** for main video + overlay
- **Resizable panels**: Preview, media library, and timeline are all resizable via drag handles
- **Filmstrip visualization**: Timeline clips show 20-frame vertical filmstrips using CSS background-position for GPU-accelerated performance
- **Professional trim UX**:
  - Drag clip edges to trim with static filmstrip and sliding grey overlay
  - Real-time playhead sync - see exact frame while dragging trim handles
  - Smooth 200ms ease-out animations on release
  - Frame-accurate trimming with visual confirmation in preview
- **Dynamic time ruler**: Markers adapt to zoom level (0.1s to 10min intervals)
- **Drag-and-drop**: Clips from media library to timeline tracks
- **Timeline controls**: Zoom in/out (20-300px/sec), playhead dragging, click to seek
- **HTML5 video preview**: Plays timeline clips with trim support
- **Playback controls**: Play/pause (spacebar), stop, keyboard shortcuts (J/K/L, arrows)
- **Undo/Redo**: Full undo/redo support with Ctrl+Z/Ctrl+Shift+Z
- **Clip selection**: Visual selection indicator (4px primary ring, brightness boost)

### Import & Export ✅ (Implemented)
- **Import**: File picker (MP4, MOV, WebM, MKV, AVI) or drag-and-drop to media library
- **Metadata extraction**: Duration, resolution, codec via ffmpeg-next bindings
- **Thumbnail generation**: Auto-generated at 1s timestamp (160x90, base64 data URL)
- **Filmstrip generation**: 20-frame vertical strips cached in temp directory
- **Export formats**: MP4, MOV, WebM with multiple resolutions:
  - Source, 720p (1280x720), 1080p (1920x1080), 1440p (2560x1440), 4K (3840x2160)
- **Export dialog**: shadcn-svelte based with format/resolution selection
- **FFmpeg bundling**: FFmpeg binary bundled as Tauri sidecar (no system dependency required)
- **Codec support**:
  - MP4/MOV: H.264 video (libx264, CRF 23) + AAC audio (192k)
  - WebM: VP9 video (CRF 30) + Opus audio (128k)

## UI Layout

### Main Editor Window (1200x800px) - Resizable Layout
```
Top Bar (50px - fixed):
- App title, Import button, Record button, Export button

Main Area (resizable vertical split):
├── Top Pane (60% default, 30-70% range):
│   ├── Preview (75% default, 40-85% range) - resizable horizontal
│   ├── Resize Handle (draggable)
│   └── Media Library (25% default, 15-60% range)
│
├── Resize Handle (draggable)
│
└── Bottom Pane (25% default, 18-28% range):
    ├── Time Ruler (24px - fixed): Zoom controls + time markers
    ├── Track 1 (Main) - 45-90px height (resizable via parent)
    └── Track 2 (Overlay) - 45-90px height (glued to controls)

Control Bar (50px - fixed at bottom):
- Play/Pause, Stop buttons (Delete via keyboard)
- Zoom controls integrated into time ruler (saves space)
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
- **UI Components**: shadcn-svelte (always use these components)
- **Icons**: @lucide/svelte (always use Lucide icons, never emojis)

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
└── lib.rs (all Tauri commands + implementation)

Implemented Tauri Commands:
- pick_video_file() → Result<Option<VideoMetadata>, String>
  - Opens file picker dialog, returns video metadata
- pick_video_file_by_path(path: String) → Result<VideoMetadata, String>
  - Used for drag-and-drop imports
- generate_thumbnail(app: AppHandle, video_path, timestamp) → Result<String, String>
  - Returns base64 data URL of thumbnail image
  - Uses bundled FFmpeg sidecar
- generate_filmstrip(app: AppHandle, video_path, clip_id, frame_count) → Result<String, String>
  - Returns file path to cached filmstrip PNG
  - Uses bundled FFmpeg sidecar, 20 frames by default
- export_video(app: AppHandle, request: ExportRequest, clips_data) → Result<String, String>
  - Exports timeline to video file with specified format/resolution
  - Uses bundled FFmpeg sidecar

Tauri Plugins:
- tauri-plugin-opener (file opening)
- tauri-plugin-dialog (file picker, save dialog)
- tauri-plugin-shell (sidecar execution for FFmpeg)
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

### Hybrid Approach: Rust Bindings + Bundled Sidecar

**Architecture Decision**: Using both `ffmpeg-next` crate AND bundled FFmpeg binary
- **Metadata Extraction**: `ffmpeg-next` 8.0.0 Rust bindings for fast, type-safe parsing
  - Used in `extract_video_metadata()` for duration, resolution, codec info
  - Native performance, no subprocess overhead
- **Video Processing**: Bundled FFmpeg binary as Tauri sidecar
  - Used for thumbnail generation, filmstrip creation, video export
  - Eliminates system dependency - app works on clean macOS installs
  - Binaries stored in `src-tauri/binaries/` with target triple suffix

**Bundled Binaries** (macOS aarch64):
- `ffmpeg-aarch64-apple-darwin` (490KB)
- `ffprobe-aarch64-apple-darwin` (286KB)
- Configured as `externalBin` in `tauri.conf.json`
- Executed via `tauri-plugin-shell` with `app.shell().sidecar("ffmpeg")`

**Implementation**:
- Initialize ffmpeg-next at startup: `ffmpeg::init()`
- Metadata extraction uses Rust bindings (synchronous)
- Video processing uses bundled sidecar (async with `tauri::async_runtime::block_on`)
- Shell permissions configured in `src-tauri/capabilities/default.json`

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

### Filmstrip Implementation

**Why CSS background-position over Canvas?**
- **GPU Acceleration**: Browser-optimized for 20+ years, uses compositor thread
- **Zero JavaScript Overhead**: No canvas redrawing on every render
- **Automatic Caching**: Browser handles image caching and memory management
- **Smooth Performance**: No blocking of main thread during scrolling/dragging

**Technical Implementation**:
1. **Generation** (`generate_filmstrip` command):
   - FFmpeg extracts 20 frames evenly distributed across video duration
   - Frames scaled to 120px width, maintaining aspect ratio
   - Tiled vertically into single PNG file using `-vf tile=1x20`
   - Cached in `$TMPDIR/clipforge_cache/filmstrips/{clip_id}_filmstrip.png`

2. **Display** (`getFilmstripStyle` function in Timeline.svelte):
   - Calculate frame dimensions: `frameHeight = trackHeight - 8px`, `frameWidth = frameHeight * (16/9)`
   - Determine visible frame count based on clip width
   - For each visible frame:
     - Map clip time to source video time (accounting for trim)
     - Calculate frame index: `floor((timeInSource / duration) * 20)`
     - Position background: `url(filmstrip) {xPos}px {-frameIndex * frameHeight}px`
   - Multiple backgrounds composited for seamless tiling

3. **Trim Interaction**:
   - Filmstrip remains static during drag (no recalculation)
   - Grey overlay (`bg-black/60`) slides to show trimmed area
   - On release: smooth 200ms transition as clip adjusts
   - Preview updates live to show exact frame at trim point

### Export Flow
1. Build FFmpeg arguments array (format, resolution, codec settings)
2. Execute via bundled sidecar: `app.shell().sidecar("ffmpeg").args(&args)`
3. Wait for completion (synchronous via `block_on`)
4. Check exit status and stderr for errors
5. Return output file path or error message

**Note**: Currently single-clip export. Multi-clip concatenation and overlay planned.

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

**IMPORTANT: Always use `bun`/`bunx` instead of `npm`/`npx`**
- This project uses Bun as the package manager
- Use `bunx` for running packages (e.g., `bunx shadcn-svelte@latest add button`)
- Never use `npm`, `npx`, `yarn`, or `pnpm` commands

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

# Installing packages
bun add <package>           # Add dependency
bun add -d <package>        # Add dev dependency
bunx <command>              # Run package binary (like npx)
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

## MVP Status

### Core Editor Features ✅ (Complete)
- [x] App launches (packaged .dmg with bundled FFmpeg)
- [x] Import MP4/MOV/WebM files via file picker or drag-and-drop
- [x] Video appears in media library with thumbnails
- [x] Click clip loads in preview with playback controls
- [x] Add clip to timeline (drag-and-drop or button)
- [x] Trim clip by dragging edges with real-time preview
- [x] Export to MP4/MOV/WebM with multiple resolutions
- [x] Exported video plays correctly
- [x] Resizable panels for customizable layout
- [x] Filmstrip visualization on timeline clips
- [x] Undo/redo support
- [x] Professional trim UX with smooth animations

### Recording Module ⏸️ (Not Started)
- [ ] Recorder window implementation
- [ ] Screen capture functionality
- [ ] Webcam recording
- [ ] Picture-in-picture combined recording
- [ ] Auto-import recordings to timeline

### Future Enhancements 🔮 (Planned)
- [ ] Multi-clip export with concatenation
- [ ] Track 2 overlay rendering in export
- [ ] Split clip at playhead
- [ ] Ripple delete
- [ ] Snap to clip edges
- [ ] Audio waveform visualization
- [ ] Windows build support

## Current Implementation Notes

### Important Technical Decisions

1. **Trim UX Architecture** (Timeline.svelte:521-646):
   - Store updates only on `mouseup`, not during drag
   - Grey overlay width controlled by `trimPreviewOffset` state
   - Transitions disabled during drag (`isCurrentlyTrimming` flag)
   - Playhead syncs live via `playbackStore.update()` in mousemove handler
   - Video element seeks directly to trim position for frame-accurate preview

2. **Preview Playback Logic** (Preview.svelte:43-61):
   - **Critical**: Only searches for clip at playhead when `isPlaying === true`
   - Falls back to `selectedTimelineClipId` then `selectedClipId`
   - Prevents issues where `currentTime >= 0` (always true) caused blank previews

3. **Resizable Panels** (Editor.svelte:91-123):
   - Uses shadcn-svelte Resizable component with nested PaneGroups
   - Timeline maxSize: 28% (prevents blank space at max expansion)
   - Track heights scale together (45-90px range, 2x maximum)
   - Controls bar fixed outside resizable area

4. **FFmpeg Sidecar Pattern**:
   ```rust
   let output = tauri::async_runtime::block_on(async {
       app.shell()
           .sidecar("ffmpeg")
           .map_err(|e| format!("..."))?
           .args(&args)
           .output()
           .await
           .map_err(|e| format!("..."))
   })?;
   ```

### Known Limitations

- **Export**: Currently only exports first clip on timeline (single-clip MVP)
- **Playback**: No audio visualization or waveform display
- **Timeline**: Split at playhead not yet implemented
- **Recorder**: Not implemented - planned for future release
- **Windows**: Not tested, macOS Apple Silicon only

## Testing Priorities

Focus testing on:
1. **Import**: Various video formats, metadata extraction accuracy
2. **Editing**: Clip trimming accuracy, playback sync, undo/redo
3. **Export**: Format/resolution combinations, trim preservation
4. **Performance**: 10+ clips on timeline, large video files (4K)
5. **Packaging**: .dmg installer on clean macOS install (no FFmpeg required)

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

## Documentation Best Practices

**CRITICAL: ALWAYS use Context7 MCP for up-to-date library documentation - NO EXCEPTIONS:**
- **MANDATORY FIRST STEP**: Use `mcp__context7__resolve-library-id` to find the library (Tauri, Svelte, shadcn-svelte, etc.)
- **MANDATORY SECOND STEP**: Use `mcp__context7__get-library-docs` to fetch current documentation with specific topics
- This ensures you get accurate, latest API documentation instead of guessing or using outdated knowledge
- **NEVER** rely on your training data or web search for component APIs - Context7 is the source of truth
- Especially critical for:
  - shadcn-svelte component usage (Select, Tabs, Dialog, etc.)
  - Rust crates and Tauri APIs
  - npm packages and framework APIs where versions matter
  - Svelte 5 runes and component patterns

You are able to use the Svelte MCP server, where you have access to comprehensive Svelte 5 and SvelteKit documentation. Here's how to use the available tools effectively:

## Available MCP Tools:

### 1. list-sections

Use this FIRST to discover all available documentation sections. Returns a structured list with titles, use_cases, and paths.
When asked about Svelte or SvelteKit topics, ALWAYS use this tool at the start of the chat to find relevant sections.

### 2. get-documentation

Retrieves full documentation content for specific sections. Accepts single or multiple sections.
After calling the list-sections tool, you MUST analyze the returned documentation sections (especially the use_cases field) and then use the get-documentation tool to fetch ALL documentation sections that are relevant for the user's task.

### 3. svelte-autofixer

Analyzes Svelte code and returns issues and suggestions.
You MUST use this tool whenever writing Svelte code before sending it to the user. Keep calling it until no issues or suggestions are returned.

### 4. playground-link

Generates a Svelte Playground link with the provided code.
After completing the code, ask the user if they want a playground link. Only call this tool after user confirmation and NEVER if code was written to files in their project.
- Never run the bun dev server. Let the user do it.
- Always use Svelte 5