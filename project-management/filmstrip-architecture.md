# Filmstrip Architecture for ClipForge

**Target:** Add video thumbnail filmstrips to timeline clips in ClipForge (Tauri + Svelte 5)

## TL;DR: Recommended Approach

**Hybrid Strategy**: FFmpeg backend generation + Canvas display with progressive loading

- **Why**: Best balance of performance, visual quality, and development simplicity for MVP
- **Complexity**: Medium (simpler than pure canvas extraction, more performant than CSS sprites)
- **Timeline**: 1-2 days implementation

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    VIDEO IMPORT FLOW                         │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  User Imports Video                                          │
│         ↓                                                    │
│  Tauri Command: generate_thumbnail()                        │
│         ↓                                                    │
│  FFmpeg: Extract single frame (instant feedback)            │
│         ↓                                                    │
│  Store in App Cache: /thumbnails/{clipId}_preview.jpg       │
│         ↓                                                    │
│  Return to Frontend → Display immediately                   │
│         ↓                                                    │
│  Background Task: generate_filmstrip()                      │
│         ↓                                                    │
│  FFmpeg: Create 20-frame vertical strip                     │
│         ↓                                                    │
│  Store: /thumbnails/{clipId}_filmstrip.png                  │
│         ↓                                                    │
│  Emit Event → Frontend replaces preview with filmstrip      │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## Data Structure

```typescript
// Extend existing Clip type in src/lib/stores/clips.js
{
  id: string,
  filename: string,
  path: string,
  duration: number,
  resolution: string,
  codec: string,
  thumbnail: string,        // EXISTING: Single frame for library preview
  filmstrip?: string,       // NEW: Path to vertical filmstrip image
  filmstripFrameCount?: number  // NEW: Number of frames in filmstrip (default: 20)
}
```

---

## Implementation Plan

### Phase 1: Backend (Rust/Tauri)

**File: `src-tauri/src/lib.rs`**

```rust
use std::process::Command;
use tauri::AppHandle;

#[tauri::command]
async fn generate_filmstrip(
    app: AppHandle,
    video_path: String,
    clip_id: String,
    frame_count: u32
) -> Result<String, String> {
    // Get app cache directory
    let cache_dir = app.path_resolver()
        .app_cache_dir()
        .ok_or("Failed to get cache dir")?;

    let thumbnails_dir = cache_dir.join("thumbnails");
    std::fs::create_dir_all(&thumbnails_dir)
        .map_err(|e| e.to_string())?;

    let output_path = thumbnails_dir
        .join(format!("{}_filmstrip.png", clip_id))
        .to_string_lossy()
        .to_string();

    // Calculate frame selection interval
    // For a 60fps 10s video: 600 frames total / 20 desired frames = select every 30 frames
    let select_interval = format!("select=not(mod(n\\,30))");

    // FFmpeg command: Extract frames, scale to 120px wide, arrange vertically
    let status = Command::new("ffmpeg")
        .args(&[
            "-i", &video_path,
            "-frames", "1",
            "-vf", &format!(
                "{},scale=120:-2,tile=1x{}",
                select_interval,
                frame_count
            ),
            &output_path,
            "-y"  // Overwrite if exists
        ])
        .status()
        .map_err(|e| e.to_string())?;

    if status.success() {
        Ok(output_path)
    } else {
        Err("FFmpeg filmstrip generation failed".into())
    }
}
```

**Why vertical strip?**
- CSS background-position works more intuitively with vertical layouts
- Single image file = single HTTP request
- Easier to calculate which frame to show based on position

---

### Phase 2: Frontend State Management

**File: `src/lib/stores/clips.js`**

```javascript
import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export const clipsStore = writable([]);

// NEW: Function to request filmstrip generation
export async function generateFilmstripForClip(clipId) {
  const clip = get(clipsStore).find(c => c.id === clipId);
  if (!clip || clip.filmstrip) return; // Already has filmstrip

  try {
    const filmstripPath = await invoke('generate_filmstrip', {
      videoPath: clip.path,
      clipId: clip.id,
      frameCount: 20
    });

    // Update clip with filmstrip path
    clipsStore.update(clips =>
      clips.map(c =>
        c.id === clipId
          ? { ...c, filmstrip: filmstripPath, filmstripFrameCount: 20 }
          : c
      )
    );
  } catch (err) {
    console.error('Filmstrip generation failed:', err);
  }
}
```

---

### Phase 3: Timeline Clip Component

**File: `src/lib/components/TimelineClip.svelte`**

```svelte
<script>
  import { convertFileSrc } from '@tauri-apps/api/core';

  let {
    clip,
    startTime,
    duration,
    trimStart,
    trimEnd,
    pixelsPerSecond
  } = $props();

  // Calculate clip width in pixels
  const clipWidth = $derived(duration * pixelsPerSecond);

  // Get source clip data
  const sourceClip = $derived($clipsStore.find(c => c.id === clip.clipId));

  // Convert file path to Tauri asset URL
  const thumbnailUrl = $derived(
    sourceClip?.thumbnail
      ? convertFileSrc(sourceClip.thumbnail)
      : null
  );

  const filmstripUrl = $derived(
    sourceClip?.filmstrip
      ? convertFileSrc(sourceClip.filmstrip)
      : null
  );

  // Calculate filmstrip display parameters
  const frameHeight = 60; // Timeline clip height
  const frameWidth = 120; // Individual thumbnail width from FFmpeg
  const totalFrames = sourceClip?.filmstripFrameCount || 20;

  // How many frames can fit in visible clip width?
  const visibleFrameCount = $derived(Math.ceil(clipWidth / frameWidth));

  // Calculate which frame indices to show based on trim
  const framePositions = $derived.by(() => {
    if (!sourceClip) return [];

    const positions = [];
    const clipDuration = duration;
    const sourceDuration = sourceClip.duration;

    // Calculate frame interval in source video time
    const frameInterval = clipDuration / visibleFrameCount;

    for (let i = 0; i < visibleFrameCount; i++) {
      // Time in the trimmed clip
      const timeInClip = i * frameInterval;
      // Corresponding time in source video
      const timeInSource = trimStart + timeInClip;

      // Which frame index in our 20-frame filmstrip?
      const frameIndex = Math.floor((timeInSource / sourceDuration) * totalFrames);
      const clampedIndex = Math.max(0, Math.min(frameIndex, totalFrames - 1));

      positions.push({
        frameIndex: clampedIndex,
        xPosition: i * frameWidth
      });
    }

    return positions;
  });
</script>

<div
  class="timeline-clip"
  style:width="{clipWidth}px"
  style:height="{frameHeight}px"
>
  {#if filmstripUrl}
    <!-- Show filmstrip frames -->
    {#each framePositions as frame (frame.xPosition)}
      <div
        class="frame"
        style:left="{frame.xPosition}px"
        style:width="{frameWidth}px"
        style:height="{frameHeight}px"
        style:background-image="url({filmstripUrl})"
        style:background-size="{frameWidth}px {frameHeight * totalFrames}px"
        style:background-position="0 -{frame.frameIndex * frameHeight}px"
      />
    {/each}
  {:else if thumbnailUrl}
    <!-- Fallback: Show single thumbnail stretched -->
    <div
      class="preview"
      style:background-image="url({thumbnailUrl})"
      style:background-size="cover"
      style:background-position="center"
    />
  {:else}
    <!-- Loading state -->
    <div class="placeholder">
      <span>Loading...</span>
    </div>
  {/if}

  <!-- Clip overlay with name/border -->
  <div class="clip-overlay">
    <span class="clip-name">{sourceClip?.filename || 'Untitled'}</span>
  </div>
</div>

<style>
  .timeline-clip {
    position: relative;
    overflow: hidden;
    border: 1px solid #444;
    border-radius: 4px;
    background: #1a1a1a;
  }

  .frame {
    position: absolute;
    top: 0;
    background-repeat: no-repeat;
    pointer-events: none;
  }

  .preview {
    width: 100%;
    height: 100%;
    opacity: 0.8;
  }

  .placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    background: #2a2a2a;
    color: #999;
    font-size: 12px;
  }

  .clip-overlay {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    padding: 4px 8px;
    background: linear-gradient(transparent, rgba(0,0,0,0.7));
    pointer-events: none;
  }

  .clip-name {
    font-size: 11px;
    color: white;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
```

---

## Progressive Loading Strategy

### On Video Import
```javascript
async function handleVideoImport(filePath) {
  // 1. Generate single thumbnail immediately (< 100ms)
  const thumbnail = await invoke('generate_thumbnail', {
    videoPath: filePath,
    clipId: newClip.id
  });

  // 2. Add to timeline with thumbnail
  clipsStore.update(clips => [...clips, {
    ...newClip,
    thumbnail
  }]);

  // 3. Generate filmstrip in background (1-2s)
  generateFilmstripForClip(newClip.id);
}
```

### On Timeline Render
```javascript
// In Timeline.svelte onMount
$effect(() => {
  $timelineStore.clips.forEach(timelineClip => {
    const sourceClip = $clipsStore.find(c => c.id === timelineClip.clipId);

    // Auto-generate filmstrip if missing
    if (sourceClip && !sourceClip.filmstrip) {
      generateFilmstripForClip(sourceClip.id);
    }
  });
});
```

---

## Performance Optimizations

### 1. **Lazy Generation**
Only generate filmstrips for clips actually added to timeline (not entire media library)

### 2. **Caching**
```rust
// Check if filmstrip already exists before regenerating
let output_path = thumbnails_dir.join(format!("{}_filmstrip.png", clip_id));
if output_path.exists() {
    return Ok(output_path.to_string_lossy().to_string());
}
```

### 3. **Efficient Rendering**
- CSS `background-position` is GPU-accelerated
- Single image per clip = minimal DOM elements
- Virtualization: Only render clips in viewport (already implemented in Timeline)

### 4. **Frame Selection**
```bash
# For 60fps 10s video (600 frames) → 20 thumbnails
# Select every 30th frame: select=not(mod(n\,30))

# For variable framerate, use time-based selection:
-vf "fps=2,scale=120:-2,tile=1x20"  # Extract 2 frames per second
```

---

## Handling Edge Cases

### 1. **Zoom Levels**
When user zooms in, show more detail:
```javascript
// Calculate frame density based on zoom
const framesNeeded = Math.ceil(clipWidth / MIN_FRAME_WIDTH);
if (framesNeeded > sourceClip.filmstripFrameCount) {
  // Option A: Repeat frames (faster)
  // Option B: Request higher-density filmstrip (better quality)
}
```

### 2. **Trimmed Clips**
Formula to map trimmed region to filmstrip frames:
```javascript
// Time in source video
const sourceTime = trimStart + (pixelPosition / pixelsPerSecond);

// Which frame in our 20-frame filmstrip?
const frameIndex = Math.floor((sourceTime / sourceDuration) * 20);

// Y-position in vertical filmstrip
const yOffset = -frameIndex * frameHeight;
```

### 3. **Loading States**
```svelte
{#if filmstripUrl}
  <!-- Full filmstrip -->
{:else if thumbnailUrl}
  <!-- Single thumbnail fallback -->
{:else}
  <!-- Loading spinner -->
{/if}
```

---

## Alternative Approaches (Not Recommended for MVP)

### ❌ Pure Canvas Extraction
**Why not**: Slow (500ms-2s per clip), browser seeking unreliable, memory intensive
**When useful**: Real-time preview without backend, browser-only apps

### ❌ CSS Sprite Sheets
**Why not**: Complex math for 2D positioning, harder to maintain
**When useful**: Grid-based layouts, static frame counts

### ✅ **Recommended: FFmpeg + CSS Background (Chosen)**
**Why**: Fast generation (200-500ms), reliable, GPU-accelerated display, simple math

---

## Implementation Checklist

- [ ] Add `generate_filmstrip` Tauri command
- [ ] Update Clip type with `filmstrip` and `filmstripFrameCount` fields
- [ ] Create `generateFilmstripForClip()` store function
- [ ] Update TimelineClip component with filmstrip rendering
- [ ] Add background filmstrip generation on video import
- [ ] Implement fallback to single thumbnail while loading
- [ ] Add caching check to avoid regenerating existing filmstrips
- [ ] Test with various zoom levels
- [ ] Test with trimmed clips
- [ ] Verify performance with 10+ clips on timeline

---

## Expected Results

**Visual Quality**: 20 frames provides smooth visual continuity at typical zoom levels

**Performance**:
- Single thumbnail: < 100ms
- Full filmstrip: 200-500ms (background)
- Rendering: 60fps (CSS background-position is GPU-accelerated)

**User Experience**:
1. Import video → See thumbnail immediately
2. Add to timeline → Thumbnail stretched across clip
3. 1-2 seconds later → Filmstrip loads, replacing stretched thumbnail
4. Zoom in/out → Frames adjust automatically
5. Trim clip → Correct frames shown based on trim range

---

## Future Enhancements (Post-MVP)

1. **Multi-Resolution Filmstrips**: Generate low/med/high density for different zoom levels
2. **WebWorker Processing**: Move filmstrip positioning calculations off main thread
3. **Scrubbing Preview**: Hover over clip to see larger preview at that timestamp
4. **Waveform Overlay**: Add audio waveform below video filmstrip
5. **IndexedDB Caching**: Persist filmstrips in browser storage for faster reloads

---

## Why This Approach Wins

✅ **Simplicity**: FFmpeg command + CSS positioning = ~150 lines of code
✅ **Performance**: GPU-accelerated rendering, fast generation
✅ **Reliability**: FFmpeg is battle-tested, no browser seeking quirks
✅ **Progressive**: Shows thumbnail immediately, upgrades to filmstrip
✅ **Svelte 5 Native**: Uses `$derived` for reactive frame calculations
✅ **Tauri Integration**: Leverages existing FFmpeg infrastructure

Start with this, iterate based on real usage. Ship the MVP! 🚀
