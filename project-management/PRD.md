# ClipForge PRD - Technical Specification

## Product Overview
Desktop video editor built with Tauri and Svelte. Compact recorder interface, full-sized editor window.

**Platform:** macOS (primary), Windows (secondary)  
**Tech Stack:** Tauri (Rust) + Svelte + FFmpeg

---

## MVP Requirements (Tuesday Deadline)

### Core Functionality
- Desktop app launches (Tauri window)
- Video import via file picker (MP4/MOV)
- Timeline showing imported clips
- Video preview player
- Trim functionality (set in/out points on single clip)
- Export to MP4
- Packaged as native app (.dmg or .msi)

**Success Criteria:** Import one video, trim it, export it.

---

## Full Submission Requirements (Wednesday Deadline)

### 1. Recording Features

**Recording Window**
- Separate floating window (400x300px)
- Always-on-top while recording
- Opens when user clicks "Record" in main window

**Screen Recording**
- Full screen or window selection capture
- Microphone audio capture
- Record/Stop/Save controls
- Duration timer display
- Red dot recording indicator

**Webcam Recording**
- System camera access
- Video with audio recording
- Live preview of webcam feed

**Simultaneous Screen + Webcam**
- Picture-in-picture layout (webcam overlay on screen)
- Live preview showing both feeds
- Single recording session captures both
- Combined output as one file

**Recording Flow**
1. User clicks "Record" in main window
2. Compact recorder window opens
3. User selects source (screen/webcam/both)
4. Recording starts
5. File auto-saves and auto-imports to timeline on stop
6. Recorder window closes

### 2. Import & Media Management

**Import Methods**
- File picker button (MP4, MOV, WebM)
- Drag-and-drop onto app window

**Media Library Panel**
- List view of imported clips
- Display filename and duration
- Click to add to timeline

### 3. Timeline Editor

**Core Timeline**
- Horizontal timeline with time ruler
- Playhead (vertical line at current position)
- Drag clips from media library to timeline
- Clips displayed as rectangles with duration

**Editing Operations**
- Arrange: Drag clips left/right to reorder
- Trim: Drag clip edges to adjust start/end points
- Split: Split clip at playhead position
- Delete: Remove clip from timeline

**Two Tracks**
- Track 1: Main video
- Track 2: Overlay/PiP (for webcam on screen recording)

**Timeline Controls**
- Zoom in/out (show more/less time)
- Snap to clip edges

### 4. Preview & Playback

**Preview Window**
- Video player showing composition at playhead
- Displays main track plus overlay track composited
- Size: 640x360 or 854x480 (16:9 aspect ratio)

**Playback Controls**
- Play/Pause (spacebar)
- Stop
- Scrubbing: Click or drag on timeline to jump
- Current time display (MM:SS / Total MM:SS)

**Audio**
- Audio synchronized with video
- System volume control

### 5. Export

**Export Dialog**
- Export button opens modal
- Select output location via file picker
- Resolution dropdown: 720p, 1080p, Source
- Start Export button

**Export Process**
- Progress bar with percentage
- Cancel button
- Completion notification
- FFmpeg rendering to MP4

**Output Specification**
- Format: MP4
- Video codec: H.264
- Audio codec: AAC
- Bitrate: FFmpeg defaults

---

## User Interface Layout

### Main Editor Window (1200x800px)
```
Top Bar (50px):
- App title
- Import button
- Record button
- Export button

Main Area (600px height):
- Left (70%): Video preview player
- Right (30%): Media library (scrollable list)

Timeline Section (150px height):
- Two-track timeline with playhead
- Time ruler

Control Bar (50px):
- Playback controls
- Split button
- Delete button
- Zoom controls
```

### Recording Window (400x300px)
```
Status Bar (40px):
- Recording indicator
- Duration timer

Preview Area (180px):
- Webcam feed preview (when applicable)

Source Selector (40px):
- Radio buttons: Screen / Webcam / Both

Action Buttons (40px):
- Stop button
- Cancel button
```

---

## Technical Architecture

### Frontend Structure (Svelte)
```
src/
├── App.svelte
├── views/
│   ├── Editor.svelte
│   └── Recorder.svelte
├── components/
│   ├── TopBar.svelte
│   ├── Preview.svelte
│   ├── MediaLibrary.svelte
│   ├── Timeline.svelte
│   ├── Controls.svelte
│   └── ExportModal.svelte
└── stores/
    ├── clips.js
    ├── timeline.js
    ├── playback.js
    └── recording.js
```

### Backend Structure (Tauri/Rust)
```
src-tauri/
├── src/
│   ├── main.rs
│   └── commands.rs
├── binaries/
│   └── ffmpeg-*
└── Cargo.toml
```

### Window Management
```rust
#[tauri::command]
fn open_recorder_window(app: tauri::AppHandle) {
    tauri::WindowBuilder::new(
        &app,
        "recorder",
        tauri::WindowUrl::App("/recorder".into())
    )
    .title("Record")
    .inner_size(400.0, 300.0)
    .always_on_top(true)
    .resizable(false)
    .build()
    .unwrap();
}
```

### Tauri Commands
```rust
#[tauri::command]
fn import_video(path: String) -> Result<VideoMetadata>

#[tauri::command]
fn save_recording(blob: Vec<u8>, filename: String) -> Result<String>

#[tauri::command]
fn export_video(clips: Vec<Clip>, output: String) -> Result<()>

#[tauri::command]
fn get_ffmpeg_path() -> String

#[tauri::command]
fn open_recorder_window(app: tauri::AppHandle)

#[tauri::command]
fn close_recorder_window(app: tauri::AppHandle)
```

### Data Models
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
  track: number,
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

---

## Implementation Tasks

### Day 1 - Foundation
**Morning**
- Initialize Tauri + Svelte project
- Main editor window layout (1200x800)
- File picker integration (Tauri dialog API)

**Afternoon**
- Video import to media library
- Display imported clips in library panel
- Preview player with HTML5 video
- Click clip in library loads in preview

**Evening**
- Basic timeline UI (two tracks)
- Drag clip from library to timeline
- Display clip as rectangle on timeline
- Click clip on timeline updates preview

### Day 2 - MVP Deadline
**Morning**
- Playhead scrubbing (click timeline to seek)
- Play/pause controls
- Trim functionality (drag clip edges on timeline)
- Preview updates with trimmed range

**Afternoon**
- FFmpeg integration for export
- Export single clip to MP4
- Progress bar during export
- Test export pipeline

**Evening**
- Package app as .dmg/.msi
- Test packaged app
- Bug fixes

### Day 3 - Full Features
**Morning**
- Create recorder window (separate Tauri window)
- Screen recording (getDisplayMedia + MediaRecorder)
- Save recording to file
- Auto-import recording to timeline

**Afternoon**
- Webcam recording in recorder window
- Source selector (screen/webcam/both)
- Simultaneous screen + webcam (PiP composition)
- Two-track timeline with overlay support

**Evening**
- Split clip functionality
- Delete clip from timeline
- Timeline arrangement (reorder clips by dragging)
- Multi-clip export with FFmpeg concatenation

---

## FFmpeg Usage

### Export Commands

**Single clip with trim**
```bash
ffmpeg -i input.mp4 -ss [trimStart] -t [duration] -c copy output.mp4
```

**Multiple clips (concatenation)**
```bash
ffmpeg -f concat -safe 0 -i filelist.txt -c copy output.mp4
```

**With PiP overlay (Track 2 over Track 1)**
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

---

## Recording Implementation

### Screen Recording
```javascript
async function startScreenRecording() {
  const stream = await navigator.mediaDevices.getDisplayMedia({
    video: { 
      cursor: "always",
      displaySurface: "monitor"
    },
    audio: {
      echoCancellation: true,
      noiseSuppression: true
    }
  });

  const mediaRecorder = new MediaRecorder(stream, {
    mimeType: 'video/webm;codecs=vp9',
    videoBitsPerSecond: 2500000
  });

  const chunks = [];
  mediaRecorder.ondataavailable = e => chunks.push(e.data);
  mediaRecorder.onstop = async () => {
    const blob = new Blob(chunks, { type: 'video/webm' });
    await saveRecording(blob);
  };

  mediaRecorder.start();
}
```

### Webcam Recording
```javascript
async function startWebcamRecording() {
  const stream = await navigator.mediaDevices.getUserMedia({
    video: {
      width: { ideal: 1280 },
      height: { ideal: 720 }
    },
    audio: true
  });

  videoPreview.srcObject = stream;
  // MediaRecorder setup same as screen recording
}
```

### Simultaneous Recording (PiP)
```javascript
async function startCombinedRecording() {
  const screenStream = await navigator.mediaDevices.getDisplayMedia({
    video: true,
    audio: true
  });
  
  const webcamStream = await navigator.mediaDevices.getUserMedia({
    video: true,
    audio: false
  });

  const canvas = document.createElement('canvas');
  canvas.width = 1920;
  canvas.height = 1080;
  const ctx = canvas.getContext('2d');

  const screenVideo = document.createElement('video');
  const webcamVideo = document.createElement('video');
  
  screenVideo.srcObject = screenStream;
  webcamVideo.srcObject = webcamStream;
  
  await screenVideo.play();
  await webcamVideo.play();

  function drawFrame() {
    ctx.drawImage(screenVideo, 0, 0, canvas.width, canvas.height);
    
    const pipWidth = 320;
    const pipHeight = 240;
    const pipX = canvas.width - pipWidth - 20;
    const pipY = canvas.height - pipHeight - 20;
    ctx.drawImage(webcamVideo, pipX, pipY, pipWidth, pipHeight);
    
    requestAnimationFrame(drawFrame);
  }
  drawFrame();

  const compositeStream = canvas.captureStream(30);
  
  const audioTrack = screenStream.getAudioTracks()[0];
  if (audioTrack) {
    compositeStream.addTrack(audioTrack);
  }

  const mediaRecorder = new MediaRecorder(compositeStream);
  // Continue with recording
}
```

### Save Recording
```javascript
async function saveRecording(blob) {
  const arrayBuffer = await blob.arrayBuffer();
  const uint8Array = new Uint8Array(arrayBuffer);

  const filename = `recording_${Date.now()}.webm`;
  const filepath = await invoke('save_recording', {
    blob: Array.from(uint8Array),
    filename
  });

  await invoke('import_video', { path: filepath });
  await invoke('close_recorder_window');
}
```

---

## Testing Checklist

### MVP Tests
- Launch app (packaged)
- Import MP4 file via file picker
- Clip appears in media library
- Click clip loads in preview
- Video plays in preview
- Add clip to timeline
- Trim clip by dragging edges
- Preview shows trimmed version
- Export trimmed clip
- Open exported file and verify

### Full Feature Tests

**Recording**
- Click "Record" opens recorder window
- Record screen for 30 seconds
- Recording auto-imports to timeline
- Record webcam for 10 seconds
- Record screen + webcam with PiP visible

**Editing**
- Import 3 different clips
- All clips appear in media library
- Drag clips onto timeline in sequence
- Clips snap to each other
- Scrub playhead updates preview
- Play timeline plays clips in sequence
- Split clip at playhead position
- Delete clip from timeline
- Drag clip on Track 2 (overlay position)
- Preview shows overlay correctly

**Export**
- Export multi-clip timeline (2+ minutes)
- Progress bar shows percentage
- Export completes successfully
- Open exported video and verify quality
- Verify clips in correct sequence
- Verify audio synchronized

**Performance**
- Timeline responsive with 10+ clips
- Preview playback smooth
- Scrubbing responsive
- Export doesn't crash
- App launches under 5 seconds

---

## Excluded Features

Features explicitly not being built:
- Text overlays
- Transitions (fade, slide, dissolve)
- Audio controls (volume, fade, separate audio track)
- Filters/effects (brightness, contrast, saturation)
- Color grading
- Export presets (YouTube, Instagram, TikTok)
- Keyboard shortcuts (except spacebar)
- Auto-save / Project files
- Undo/redo
- Cloud upload or sharing
- Thumbnail generation
- Audio waveforms visualization
- Markers/annotations
- Multi-language support
- Settings/preferences panel
- Help documentation
- Templates or presets
- Batch export
- Green screen / chroma key
- Speed controls (slow-mo, time-lapse)

---

## Deliverables

### GitHub Repository
- Complete source code
- README with setup instructions
- Architecture overview
- .gitignore for build artifacts
- License file

### Packaged App
- macOS: .dmg installer
- Windows: .msi or .exe (if time)
- Host on GitHub Releases or file hosting
- Include download link in README

### README Requirements
- System requirements
- Installation instructions
- Running from source instructions
- Building from source instructions
- Usage guide
- Known limitations
- Architecture overview
- Tech stack and dependencies

---

## Success Criteria

### MVP (Tuesday)
- App launches on macOS (packaged)
- Can import video file
- Video plays in preview
- Can trim video on timeline
- Can export to MP4
- Exported video plays correctly

### Full Submission (Wednesday)
- All three recording modes work (screen, webcam, both)
- Recorder window opens/closes correctly
- Recordings auto-import to timeline
- Can arrange multiple clips on timeline
- Can trim and split clips
- Two-track timeline functions (overlay)
- Multi-clip export produces valid MP4
- Packaged installer works

### Quality Bar
- No crashes during basic workflow
- Export produces playable video with correct content
- UI is usable
- Recording permissions work
- Timeline editing is functional

---

## Risk Areas

### High-Risk Components
1. FFmpeg multi-clip export - Complex command generation
2. Recording window management - Tauri multi-window coordination
3. PiP compositing - Canvas rendering performance
4. Timeline drag/drop - Complex interaction state
5. Recording permissions - OS-level security

### Fallback Plans
- If PiP too complex: Skip simultaneous recording, do separate tracks only
- If multi-window fails: Embed recorder in main window
- If drag/drop problematic: Use buttons to add/move/delete clips
- If Windows build fails: Submit macOS only
- If export crashes: Simplify to single-track only, skip overlays

---

## Window Specifications

| Window | Size | Purpose | Style |
|--------|------|---------|-------|
| Main Editor | 1200x800px | Editing interface | Resizable, normal |
| Recorder | 400x300px | Recording controls | Fixed, always-on-top |