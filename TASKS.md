# ClipForge Implementation Tasks

These tasks are organized for MVP (Tuesday) while maintaining architectural compatibility with full features (Wednesday and beyond).

## Phase 1: Foundation & Architecture (Day 1 Morning)

### Task 1.1: Main Editor Window Layout
**Objective:** Create the 1200x800px main editor window with layout structure

**Requirements:**
- Window dimensions: 1200x800px (resizable)
- Layout: Top bar (50px) + main area (600px) + timeline (150px) + control bar (50px)
- Responsive flex layout that maintains proportions
- No content yet, just structure and styling

**Interfaces to Design:**
- EditorStore (for editor state, playback state)
- TimelineStore (for clips on timeline)
- ClipsStore (for media library)

**Acceptance Criteria:**
- Window opens at correct size
- Layout sections visible and properly proportioned
- Ready for component insertion

---

### Task 1.2: Component Structure & State Management
**Objective:** Set up Svelte components and stores for entire application

**Components to Create:**
```
src/
├── App.svelte (router, main window selection)
├── components/
│   ├── TopBar.svelte (import, record, export buttons - record/export empty for now)
│   ├── Preview.svelte (video player, empty initially)
│   ├── MediaLibrary.svelte (clips list, empty initially)
│   ├── Timeline.svelte (timeline UI, empty initially)
│   └── Controls.svelte (playback buttons, empty initially)
├── views/
│   ├── Editor.svelte (main editor layout - will be route later)
│   └── Recorder.svelte (stub for future recording window)
└── stores/
    ├── clips.js (media library clips)
    ├── timeline.js (timeline clips data, playhead position)
    ├── playback.js (play/pause, current time)
    └── recording.js (stub for future recording state)
```

**Store Interfaces:**
```javascript
// clips.js
export const clipsStore = writable([
  // { id, filename, path, duration, resolution }
]);

// timeline.js
export const timelineStore = writable({
  clips: [], // { id, clipId, track, startTime, trimStart, trimEnd, duration }
  playhead: 0,
  duration: 0
});

// playback.js
export const playbackStore = writable({
  isPlaying: false,
  currentTime: 0,
  selectedClipId: null
});

// recording.js (stub)
export const recordingStore = writable({
  isRecording: false,
  source: null // 'screen' | 'webcam' | 'both'
});
```

**Acceptance Criteria:**
- All components render without errors
- Stores initialized and accessible from components
- App.svelte selects which view to display (Editor vs Recorder)
- Layout structure visible

---

## Phase 2: File Import System (Day 1 Afternoon - Morning)

### Task 2.1: File Picker Integration
**Objective:** Implement file picker dialog via Tauri

**Requirements:**
- Add `tauri_dialog` integration to Tauri backend
- Create `pick_video_file` Tauri command
- Show file picker for MP4, MOV (WebM for future recordings)
- Return selected file path

**Rust Backend** (src-tauri/src/lib.rs or new commands.rs):
```rust
#[tauri::command]
async fn pick_video_file() -> Result<Option<String>, String> {
    let file = tauri::api::dialog::FileDialogBuilder::new()
        .add_filter("Video", &["mp4", "mov", "webm"])
        .pick_file()
        .await
        .map_err(|e| e.to_string())?;

    Ok(file.map(|p| p.to_string_lossy().to_string()))
}
```

**Frontend Integration:**
```svelte
<!-- TopBar.svelte -->
<button on:click={openFilePicker}>Import</button>

<script>
  import { invoke } from "@tauri-apps/api/core";

  async function openFilePicker() {
    const filePath = await invoke("pick_video_file");
    if (filePath) {
      await importVideo(filePath);
    }
  }
</script>
```

**Acceptance Criteria:**
- File picker opens on button click
- Only video files shown
- Returns valid file path or null
- Ready for video import handler

---

### Task 2.2: Video Metadata Extraction & Clip Store
**Objective:** Extract video metadata and populate media library

**Requirements:**
- Extract duration and resolution from video file
- Create clip object in clipsStore
- Display in media library panel

**Rust Backend** (new VideoMetadata struct):
```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct VideoMetadata {
    pub duration: f64, // seconds
    pub width: u32,
    pub height: u32,
    pub fps: f64
}

#[tauri::command]
async fn get_video_metadata(path: String) -> Result<VideoMetadata, String> {
    // Use ffprobe or ffmpeg to extract metadata
    // For MVP, can hardcode test values or use simple heuristic
}

#[tauri::command]
async fn import_video(path: String) -> Result<String, String> {
    let metadata = get_video_metadata(path.clone()).await?;
    // Store metadata, generate unique ID
    // Return clip ID to frontend
}
```

**Frontend** (in TopBar.svelte or separate service):
```svelte
<script>
  import { clipsStore } from '../stores/clips.js';

  async function importVideo(filePath) {
    const clipId = await invoke("import_video", { path: filePath });
    const filename = filePath.split('/').pop();
    const metadata = await invoke("get_video_metadata", { path: filePath });

    clipsStore.update(clips => [...clips, {
      id: clipId,
      filename,
      path: filePath,
      duration: metadata.duration,
      resolution: `${metadata.width}x${metadata.height}`
    }]);
  }
</script>
```

**Acceptance Criteria:**
- Video metadata extracted successfully
- Clip added to clipsStore with all fields
- Ready for MediaLibrary display

---

### Task 2.3: Media Library UI
**Objective:** Display imported clips in list view

**Requirements:**
- Show filename, duration formatted as MM:SS, resolution
- Click clip to select it
- Update Preview on selection
- Prepare for drag-to-timeline in next phase

**MediaLibrary.svelte:**
```svelte
<script>
  import { clipsStore } from '../stores/clips.js';
  import { playbackStore } from '../stores/playback.js';

  function selectClip(clipId) {
    playbackStore.update(state => ({
      ...state,
      selectedClipId: clipId
    }));
  }

  function formatTime(seconds) {
    const m = Math.floor(seconds / 60);
    const s = Math.floor(seconds % 60);
    return `${m}:${s.toString().padStart(2, '0')}`;
  }
</script>

<div class="media-library">
  {#each $clipsStore as clip (clip.id)}
    <div
      class="clip-item"
      on:click={() => selectClip(clip.id)}
    >
      <div class="filename">{clip.filename}</div>
      <div class="metadata">
        {formatTime(clip.duration)} | {clip.resolution}
      </div>
    </div>
  {/each}
</div>

<style>
  .media-library {
    overflow-y: auto;
    padding: 10px;
  }

  .clip-item {
    padding: 10px;
    margin: 5px 0;
    background: #f0f0f0;
    border-radius: 4px;
    cursor: pointer;
  }

  .clip-item:hover {
    background: #e0e0e0;
  }
</style>
```

**Acceptance Criteria:**
- Imported clips display in list
- Click selects clip
- Duration formatted correctly
- selectedClipId updated in store

---

## Phase 3: Preview Player (Day 1 Afternoon)

### Task 3.1: HTML5 Video Preview Component
**Objective:** Create video preview player connected to media library

**Requirements:**
- HTML5 `<video>` element
- Display selected clip from media library
- Prepare for playback controls in next task
- Show current time / total duration

**Preview.svelte:**
```svelte
<script>
  import { clipsStore } from '../stores/clips.js';
  import { playbackStore } from '../stores/playback.js';

  let videoElement;
  let currentTime = 0;
  let duration = 0;

  $: selectedClip = $clipsStore.find(c => c.id === $playbackStore.selectedClipId);

  function formatTime(seconds) {
    const m = Math.floor(seconds / 60);
    const s = Math.floor(seconds % 60);
    return `${m}:${s.toString().padStart(2, '0')}`;
  }

  function onTimeUpdate() {
    currentTime = videoElement?.currentTime || 0;
    playbackStore.update(state => ({
      ...state,
      currentTime
    }));
  }

  function onLoadedMetadata() {
    duration = videoElement?.duration || 0;
  }
</script>

<div class="preview-container">
  <video
    bind:this={videoElement}
    src={selectedClip?.path}
    on:timeupdate={onTimeUpdate}
    on:loadedmetadata={onLoadedMetadata}
    width="640"
    height="360"
  />

  <div class="time-display">
    {formatTime(currentTime)} / {formatTime(duration)}
  </div>
</div>

<style>
  .preview-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    background: #000;
    aspect-ratio: 16 / 9;
  }

  video {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  .time-display {
    color: #fff;
    position: absolute;
    bottom: 10px;
    font-size: 12px;
  }
</style>
```

**Acceptance Criteria:**
- Video displays when clip selected
- Time updates as video plays
- Duration shows correctly
- Ready for playback controls

---

## Phase 4: Timeline Foundation (Day 1 Evening)

### Task 4.1: Timeline UI Structure
**Objective:** Create two-track timeline with playhead and time ruler

**Requirements:**
- Horizontal timeline showing time from 0 to max duration
- Time ruler (marks at 5s, 10s, 15s, etc.)
- Playhead (vertical line at current playback position)
- Two tracks (Track 1 main, Track 2 overlay - empty for MVP)
- Calculate total timeline duration from playhead

**Timeline.svelte:**
```svelte
<script>
  import { timelineStore } from '../stores/timeline.js';
  import { playbackStore } from '../stores/playback.js';

  let timelineElement;
  let zoom = 1; // pixels per second (can adjust later)

  $: timelineWidth = ($timelineStore.duration * zoom);
  $: playheadPosition = ($playbackStore.currentTime * zoom);

  function getTimeMarkers(duration) {
    const markers = [];
    for (let i = 0; i <= duration; i += 5) {
      markers.push(i);
    }
    return markers;
  }

  function formatTime(seconds) {
    const m = Math.floor(seconds / 60);
    const s = Math.floor(seconds % 60);
    return `${m}:${s.toString().padStart(2, '0')}`;
  }

  function handleTimelineClick(e) {
    const rect = timelineElement.getBoundingClientRect();
    const clickX = e.clientX - rect.left;
    const newTime = clickX / zoom;

    playbackStore.update(state => ({
      ...state,
      currentTime: newTime
    }));

    if (videoElement) {
      videoElement.currentTime = newTime;
    }
  }
</script>

<div class="timeline-container">
  <!-- Time Ruler -->
  <div class="time-ruler">
    {#each getTimeMarkers($timelineStore.duration) as time}
      <div
        class="marker"
        style="left: {time * zoom}px"
      >
        {formatTime(time)}
      </div>
    {/each}
  </div>

  <!-- Timeline Tracks -->
  <div
    bind:this={timelineElement}
    class="timeline"
    on:click={handleTimelineClick}
    style="width: {Math.max(timelineWidth, 100)}px"
  >
    <!-- Playhead -->
    <div
      class="playhead"
      style="left: {playheadPosition}px"
    />

    <!-- Track 1 (Main) -->
    <div class="track">
      <!-- Clips will be inserted here by Task 4.2 -->
    </div>

    <!-- Track 2 (Overlay) -->
    <div class="track">
      <!-- For future PiP overlay -->
    </div>
  </div>
</div>

<style>
  .timeline-container {
    display: flex;
    flex-direction: column;
    height: 150px;
    border-top: 1px solid #ccc;
    background: #fafafa;
    overflow-x: auto;
  }

  .time-ruler {
    position: relative;
    height: 30px;
    border-bottom: 1px solid #ccc;
    background: #f0f0f0;
  }

  .marker {
    position: absolute;
    font-size: 10px;
    color: #666;
    top: 5px;
  }

  .timeline {
    position: relative;
    display: flex;
    flex: 1;
    min-height: 120px;
    cursor: pointer;
  }

  .playhead {
    position: absolute;
    width: 2px;
    height: 100%;
    background: red;
    z-index: 10;
  }

  .track {
    flex: 1;
    position: relative;
    border-bottom: 1px solid #ddd;
  }
</style>
```

**Acceptance Criteria:**
- Time ruler displays correctly
- Playhead visible and positioned at current time
- Two tracks visible
- Click timeline updates playback position
- Timeline width expands with duration

---

### Task 4.2: Drag Clips to Timeline
**Objective:** Enable dragging clips from media library to timeline

**Requirements:**
- Implement HTML5 drag-and-drop from MediaLibrary to Timeline
- Add clip to timeline tracks on drop
- Display clip as rectangle with duration
- Update timelineStore with new timeline clip
- Support both tracks (for future PiP)

**MediaLibrary.svelte (updated):**
```svelte
<div
  class="clip-item"
  draggable="true"
  on:dragstart={(e) => {
    e.dataTransfer.effectAllowed = 'copy';
    e.dataTransfer.setData('application/json', JSON.stringify({
      clipId: clip.id,
      duration: clip.duration
    }));
  }}
>
  <!-- ... -->
</div>
```

**Timeline.svelte (updated):**
```svelte
<div class="track"
  on:drop={handleDrop}
  on:dragover={(e) => e.preventDefault()}
>
  {#each getClipsForTrack(trackIndex) as timelineClip}
    <div
      class="timeline-clip"
      style="
        left: {timelineClip.startTime * zoom}px;
        width: {timelineClip.duration * zoom}px;
      "
      on:click={() => selectTimelineClip(timelineClip.id)}
    >
      {getClipFilename(timelineClip.clipId)}
    </div>
  {/each}
</div>

<script>
  function handleDrop(e, trackIndex) {
    e.preventDefault();
    const data = JSON.parse(e.dataTransfer.getData('application/json'));

    const rect = e.currentTarget.getBoundingClientRect();
    const dropX = e.clientX - rect.left;
    const startTime = dropX / zoom;

    const timelineClip = {
      id: generateId(),
      clipId: data.clipId,
      track: trackIndex,
      startTime,
      trimStart: 0,
      trimEnd: data.duration,
      duration: data.duration
    };

    timelineStore.update(state => ({
      ...state,
      clips: [...state.clips, timelineClip],
      duration: Math.max(state.duration, startTime + data.duration)
    }));
  }
</script>
```

**Acceptance Criteria:**
- Can drag clips from media library to timeline
- Clips display as rectangles on tracks
- Correct duration shown
- timelineStore updated with timeline clips
- Supports both Track 1 and Track 2

---

## Phase 5: Playback Controls (Day 2 Morning)

### Task 5.1: Play/Pause Controls & Playhead Scrubbing
**Objective:** Implement basic playback control and timeline scrubbing

**Requirements:**
- Play/pause button in control bar
- Spacebar for play/pause
- Click on timeline to seek (already done in Task 4.1, refine here)
- Drag playhead to scrub
- Update currentTime in playbackStore

**Controls.svelte:**
```svelte
<script>
  import { playbackStore } from '../stores/playback.js';

  let videoElement; // passed from Preview

  function togglePlayPause() {
    playbackStore.update(state => ({
      ...state,
      isPlaying: !state.isPlaying
    }));

    if ($playbackStore.isPlaying) {
      videoElement?.play();
    } else {
      videoElement?.pause();
    }
  }

  function handleKeyPress(e) {
    if (e.code === 'Space') {
      e.preventDefault();
      togglePlayPause();
    }
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeyPress);
    return () => window.removeEventListener('keydown', handleKeyPress);
  });
</script>

<div class="controls">
  <button on:click={togglePlayPause}>
    {$playbackStore.isPlaying ? '⏸ Pause' : '▶ Play'}
  </button>

  <button on:click={() => videoElement?.pause()}>
    ⏹ Stop
  </button>
</div>

<style>
  .controls {
    display: flex;
    gap: 10px;
    padding: 10px;
  }

  button {
    padding: 8px 16px;
    cursor: pointer;
  }
</style>
```

**Preview.svelte (updated):**
- Pass videoElement ref to Controls
- Keep time sync between video element and playbackStore

**Timeline.svelte (updated):**
```svelte
<div
  class="playhead"
  draggable="true"
  style="left: {playheadPosition}px"
  on:drag={(e) => {
    const rect = timelineElement.getBoundingClientRect();
    const newPos = Math.max(0, e.clientX - rect.left);
    const newTime = newPos / zoom;
    playbackStore.update(state => ({...state, currentTime: newTime}));
  }}
/>
```

**Acceptance Criteria:**
- Play/pause button works and updates UI
- Spacebar toggles play/pause
- Timeline click seeks to position
- Playhead dragging scrubs video
- currentTime kept in sync with video element

---

### Task 5.2: Trim Functionality
**Objective:** Enable trimming clips by dragging edges on timeline

**Requirements:**
- Show drag handles on clip edges (left/right)
- Drag left edge to adjust trimStart
- Drag right edge to adjust trimEnd
- Update preview to show trimmed range
- Maintain trimmed range in timelineStore

**Timeline.svelte (updated):**
```svelte
<div
  class="timeline-clip"
  style="left: {timelineClip.startTime * zoom}px; width: {(timelineClip.trimEnd - timelineClip.trimStart) * zoom}px"
  on:click={() => selectTimelineClip(timelineClip.id)}
>
  <!-- Left handle -->
  <div
    class="trim-handle left"
    on:mousedown={(e) => startTrimDrag(e, timelineClip.id, 'start')}
  />

  <span class="clip-label">{getClipFilename(timelineClip.clipId)}</span>

  <!-- Right handle -->
  <div
    class="trim-handle right"
    on:mousedown={(e) => startTrimDrag(e, timelineClip.id, 'end')}
  />
</div>

<script>
  function startTrimDrag(e, clipId, side) {
    e.preventDefault();
    e.stopPropagation();

    const startX = e.clientX;
    const startTime = $timelineStore.clips.find(c => c.id === clipId)[
      side === 'start' ? 'trimStart' : 'trimEnd'
    ];

    function handleMouseMove(moveE) {
      const deltaX = moveE.clientX - startX;
      const deltaTime = deltaX / zoom;
      const newTime = Math.max(0, startTime + deltaTime);

      timelineStore.update(state => ({
        ...state,
        clips: state.clips.map(c =>
          c.id === clipId
            ? {
                ...c,
                [side === 'start' ? 'trimStart' : 'trimEnd']: newTime,
                duration: Math.abs(
                  side === 'start'
                    ? c.trimEnd - newTime
                    : newTime - c.trimStart
                )
              }
            : c
        )
      }));
    }

    function handleMouseUp() {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    }

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  }
</script>

<style>
  .timeline-clip {
    position: relative;
    background: #4CAF50;
    color: white;
    padding: 5px;
    display: flex;
    align-items: center;
    user-select: none;
    cursor: pointer;
  }

  .trim-handle {
    position: absolute;
    width: 8px;
    height: 100%;
    cursor: col-resize;
  }

  .trim-handle.left {
    left: 0;
  }

  .trim-handle.right {
    right: 0;
  }
</style>
```

**Preview.svelte (updated):**
- Show only trimmed portion of selected clip
- Update video currentTime range when trim changes

```svelte
<script>
  let trimStart = 0;
  let trimEnd = 0;

  $: if (selectedTimelineClip) {
    trimStart = selectedTimelineClip.trimStart;
    trimEnd = selectedTimelineClip.trimEnd;
  }

  function onTimeUpdate() {
    if (videoElement.currentTime > trimEnd) {
      videoElement.currentTime = trimStart;
      videoElement.play(); // loop
    }
  }
</script>

<video
  on:play={() => videoElement.currentTime = trimStart}
/>
```

**Acceptance Criteria:**
- Drag handles visible on clip edges
- Trimming updates timelineStore
- Preview shows only trimmed portion
- Trim handles responsive and intuitive

---

## Phase 6: FFmpeg Integration & Export (Day 2 Afternoon)

### Task 6.1: FFmpeg Backend Setup & Export Command
**Objective:** Integrate FFmpeg for video export

**Requirements:**
- Add FFmpeg binding to Tauri (tauri-plugin-ffmpeg or shell command)
- Create `export_video` Tauri command
- Take timeline clips data and export to MP4
- For MVP: single track, single clip with trimming

**Rust Backend** (src-tauri/src/lib.rs or commands.rs):
```rust
use std::process::Command;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TimelineClip {
    pub id: String,
    pub clipId: String,
    pub track: i32,
    pub startTime: f64,
    pub trimStart: f64,
    pub trimEnd: f64,
    pub duration: f64,
}

#[tauri::command]
async fn get_ffmpeg_path() -> Result<String, String> {
    // Return path to ffmpeg binary
    // On macOS, can use `which ffmpeg` or bundle with app
    Ok("ffmpeg".to_string())
}

#[tauri::command]
async fn export_video(
    clips: Vec<(String, TimelineClip)>, // (path, clip data)
    output_path: String,
    resolution: String,
) -> Result<String, String> {
    // For MVP: single clip with trim
    // Generate FFmpeg command:
    // ffmpeg -i input.mp4 -ss [trimStart] -t [duration] -c copy output.mp4

    let clip = &clips[0];
    let input_path = &clip.0;
    let trim_data = &clip.1;

    let ffmpeg_cmd = format!(
        "ffmpeg -i \"{}\" -ss {} -t {} -c copy \"{}\"",
        input_path,
        trim_data.trimStart,
        trim_data.trimEnd - trim_data.trimStart,
        output_path
    );

    // Execute command
    let output = Command::new("sh")
        .arg("-c")
        .arg(&ffmpeg_cmd)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(format!("Export complete: {}", output_path))
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
```

**Acceptance Criteria:**
- FFmpeg command generated correctly
- Single clip with trim exports successfully
- Output is valid MP4
- Ready for multi-clip export in full product

---

### Task 6.2: Export Dialog & Progress Tracking
**Objective:** Create export UI and progress tracking

**Requirements:**
- Export button in TopBar
- Modal dialog with file picker and resolution selection
- Progress bar during export
- Cancel button
- Completion notification

**ExportModal.svelte:**
```svelte
<script>
  import { invoke } from "@tauri-apps/api/core";
  import { timelineStore } from '../stores/timeline.js';
  import { clipsStore } from '../stores/clips.js';

  let showModal = false;
  let isExporting = false;
  let progress = 0;
  let resolution = 'Source';

  async function handleExport() {
    isExporting = true;
    progress = 0;

    try {
      const outputPath = await invoke('save_dialog');

      // Prepare clip data for backend
      const clipsData = $timelineStore.clips.map(tc => [
        $clipsStore.find(c => c.id === tc.clipId).path,
        tc
      ]);

      const result = await invoke('export_video', {
        clips: clipsData,
        outputPath,
        resolution
      });

      // For MVP, simple export without progress updates
      progress = 100;

      setTimeout(() => {
        showModal = false;
        alert('Export completed!');
      }, 1000);
    } catch (err) {
      alert(`Export failed: ${err}`);
    } finally {
      isExporting = false;
    }
  }
</script>

{#if showModal}
  <div class="modal-overlay">
    <div class="modal">
      <h2>Export Video</h2>

      <label>
        Resolution:
        <select bind:value={resolution}>
          <option value="Source">Source</option>
          <option value="720p">720p</option>
          <option value="1080p">1080p</option>
        </select>
      </label>

      {#if isExporting}
        <div class="progress-bar">
          <div class="progress-fill" style="width: {progress}%"></div>
        </div>
        <p>{progress}%</p>
        <button on:click={() => alert('Cancel not yet implemented')}>Cancel</button>
      {:else}
        <button on:click={handleExport}>Start Export</button>
        <button on:click={() => showModal = false}>Close</button>
      {/if}
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal {
    background: white;
    padding: 30px;
    border-radius: 8px;
    min-width: 400px;
  }

  .progress-bar {
    width: 100%;
    height: 20px;
    background: #e0e0e0;
    border-radius: 4px;
    overflow: hidden;
    margin: 20px 0;
  }

  .progress-fill {
    height: 100%;
    background: #4CAF50;
    transition: width 0.3s;
  }

  button {
    margin: 10px 5px 0 0;
    padding: 8px 16px;
    cursor: pointer;
  }
</style>
```

**TopBar.svelte (updated):**
```svelte
<button on:click={() => showExportModal = true}>Export</button>

<ExportModal bind:show={showExportModal} />
```

**Acceptance Criteria:**
- Export modal opens/closes
- Resolution selection works
- Export button triggers FFmpeg command
- Progress bar displays
- Export completes with notification

---

## Phase 7: Packaging & Release (Day 2 Evening)

### Task 7.1: Tauri Build Configuration & Packaging
**Objective:** Build and package app as native .dmg

**Requirements:**
- Verify Tauri build configuration in src-tauri/tauri.conf.json
- Build for macOS
- Generate .dmg installer
- Test packaged app

**Process:**
```bash
bun run build              # Build frontend
bun run tauri build        # Build Tauri app
# Output: src-tauri/target/release/bundle/dmg/
```

**Acceptance Criteria:**
- .dmg builds successfully
- App launches from packaged .dmg
- All MVP features work in packaged app
- No console errors

---

## Future Task Stubs (for Wednesday & Beyond)

These should be implemented with interfaces designed in Phase 1-2:

### Task 8.1: Recording Window & Screen Recording
- Create separate Tauri window for recorder (400x300px)
- Implement screen recording with getDisplayMedia
- Save WebM, convert to MP4

### Task 8.2: Webcam & PiP Recording
- Implement webcam recording with getUserMedia
- Create canvas-based PiP compositing
- Combine screen + webcam feeds

### Task 8.3: Recording Auto-Import
- Save recording file to temp directory
- Auto-import to media library
- Auto-add to timeline on save

### Task 8.4: Split & Delete Operations
- Split clip at playhead position
- Delete clip from timeline
- Update timeline duration

### Task 8.5: Multi-Clip Export & Concatenation
- Support multiple clips on timeline
- Generate FFmpeg concat demuxer file
- Handle overlay track in export

### Task 8.6: Drag Clip Reordering
- Drag clips left/right to reorder on timeline
- Snap to clip edges
- Update clip positions

---

## Implementation Order (MVP Priority)

1. **Task 1.1** - Window layout
2. **Task 1.2** - Component structure & stores
3. **Task 2.1** - File picker
4. **Task 2.2** - Video metadata
5. **Task 2.3** - Media library UI
6. **Task 3.1** - Preview player
7. **Task 4.1** - Timeline UI
8. **Task 4.2** - Drag to timeline
9. **Task 5.1** - Play/pause & scrubbing
10. **Task 5.2** - Trim functionality
11. **Task 6.1** - FFmpeg export
12. **Task 6.2** - Export dialog
13. **Task 7.1** - Packaging

This order ensures early integration of components and allows parallel work on independent features.
