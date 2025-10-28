<script>
  import { timelineStore } from '../stores/timeline.js';
  import { playbackStore } from '../stores/playback.js';
  import { clipsStore, generateFilmstripForClip } from '../stores/clips.js';
  import { undoManager, updateUndoRedoState } from '../stores/undo.js';
  import { Button } from "$lib/components/ui/button";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { ZoomIn, ZoomOut, Maximize2, X } from "@lucide/svelte";
  import { convertFileSrc } from '@tauri-apps/api/core';

  /**
   * Timeline Component
   * Displays two-track timeline with playhead
   * Supports drag-and-drop clip placement and trimming
   */

  let { videoElement = $bindable(null) } = $props();

  /** @type {HTMLElement | null} */
  let timelineElement = null;
  /** @type {HTMLElement | null} */
  let timelineContainer = null;
  /** @type {HTMLElement | null} */
  let track1Element = null;
  let zoom = $state(100); // pixels per second
  const MIN_ZOOM = 20;
  const MAX_ZOOM = 300;

  // Track height configuration
  const BASE_TRACK_HEIGHT = 45; // Original/minimum height
  const DEFAULT_TRACK_HEIGHT = 90; // New default (2x base)
  const MAX_TRACK_HEIGHT = 90; // Maximum (2x minimum)

  // Measure actual track height from rendered element
  let trackHeight = $state(DEFAULT_TRACK_HEIGHT);

  let isDraggingOverTrack1 = $state(false);
  let isDraggingOverTrack2 = $state(false);
  let isDraggingPlayhead = $state(false);

  // Calculate timeline duration: max of timeline clips duration OR currently selected video duration
  let effectiveTimelineDuration = $derived.by(() => {
    let maxDuration = $timelineStore.duration;

    // If we have a selected clip playing (not on timeline), use its duration
    if ($playbackStore.selectedClipId && !$playbackStore.selectedTimelineClipId && videoElement) {
      const selectedClip = $clipsStore.find(c => c.id === $playbackStore.selectedClipId);
      if (selectedClip) {
        maxDuration = Math.max(maxDuration, selectedClip.duration);
      }
    }

    return Math.max(maxDuration, 10); // Minimum 10 seconds for empty timeline
  });

  let timelineWidth = $derived(effectiveTimelineDuration * zoom);
  let playheadPosition = $derived($playbackStore.currentTime * zoom);

  /** @param {number} duration */
  function getTimeMarkers(duration) {
    const markers = [];
    // If timeline is empty (only default duration), just show 0:00
    if ($timelineStore.clips.length === 0) {
      return [0];
    }
    const step = duration > 60 ? 10 : 5;
    for (let i = 0; i <= Math.ceil(duration); i += step) {
      markers.push(i);
    }
    return markers;
  }

  /** @param {number} seconds */
  function formatTime(seconds) {
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = Math.floor(seconds % 60);
    if (h > 0) {
      return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
    }
    return `${m}:${s.toString().padStart(2, '0')}`;
  }

  /** @param {MouseEvent} e
   *  @param {number} trackIndex */
  function handleTimelineClick(e, trackIndex) {
    if (!timelineElement) return;
    const rect = timelineElement.getBoundingClientRect();
    const clickX = e.clientX - rect.left;
    const newTime = clickX / zoom;

    playbackStore.update(state => ({
      ...state,
      currentTime: newTime,
      selectedTimelineClipId: null // Deselect clip when clicking empty timeline
    }));

    // Seek video to clicked position
    if (videoElement) {
      videoElement.currentTime = newTime;
    }
  }

  /** @param {DragEvent} e
   *  @param {number} trackIndex */
  function handleDrop(e, trackIndex) {
    console.log("Drop event triggered on track", trackIndex);

    // Clear drag state
    isDraggingOverTrack1 = false;
    isDraggingOverTrack2 = false;

    // Check if this is an internal clip drag
    const types = e.dataTransfer?.types || [];
    const isInternalDrag = types.includes('text/x-clipforge-clip') || types.includes('text/plain');

    if (!isInternalDrag) {
      console.log("Not an internal drag, ignoring");
      return;
    }

    e.preventDefault();
    e.stopPropagation();

    if (!e.dataTransfer || !e.currentTarget) return;

    // Try to get data
    let data;
    const jsonStr = e.dataTransfer.getData('application/json');
    if (jsonStr) {
      data = JSON.parse(jsonStr);
    } else {
      // Fallback: get clipId from text/plain
      const clipId = e.dataTransfer.getData('text/plain');
      if (!clipId) {
        console.error('No clip data in drop event');
        return;
      }

      // Find clip in store
      const clip = $clipsStore.find(c => c.id === clipId);
      if (!clip) {
        console.error('Clip not found:', clipId);
        return;
      }

      data = {
        clipId: clip.id,
        duration: clip.duration,
        path: clip.path
      };
    }

    console.log('Drop data:', data);

    // Calculate drop position relative to timeline element
    if (!timelineElement) return;
    const timelineRect = timelineElement.getBoundingClientRect();
    const dropX = e.clientX - timelineRect.left;
    const startTime = Math.max(0, dropX / zoom);

    const timelineClip = {
      id: `timeline-clip-${Date.now()}-${Math.random()}`,
      clipId: data.clipId,
      track: trackIndex,
      startTime,
      trimStart: 0,
      trimEnd: data.duration,
      duration: data.duration
    };

    console.log('Dropping clip on track', trackIndex, 'at time', startTime, timelineClip);

    // Save state for undo
    undoManager.saveState();

    timelineStore.update(state => ({
      ...state,
      clips: [...state.clips, timelineClip],
      duration: Math.max(state.duration, startTime + data.duration)
    }));

    updateUndoRedoState();
  }

  /** @param {DragEvent} e
   *  @param {number} trackIndex */
  function handleDragOver(e, trackIndex) {
    // Check if this is an internal clip drag
    const types = e.dataTransfer?.types || [];
    const isInternalDrag = types.includes('text/x-clipforge-clip') || types.includes('text/plain');

    if (isInternalDrag) {
      e.preventDefault();
      e.stopPropagation();

      if (e.dataTransfer) {
        e.dataTransfer.dropEffect = 'copy';
      }

      if (trackIndex === 0) {
        isDraggingOverTrack1 = true;
      } else {
        isDraggingOverTrack2 = true;
      }

      console.log("Dragging over track", trackIndex);
    }
  }

  /** @param {number} trackIndex */
  function handleDragLeave(trackIndex) {
    if (trackIndex === 0) {
      isDraggingOverTrack1 = false;
    } else {
      isDraggingOverTrack2 = false;
    }
  }

  /** @param {string} clipId */
  function selectTimelineClip(clipId) {
    console.log("Selected timeline clip:", clipId);
    playbackStore.update(state => ({
      ...state,
      selectedTimelineClipId: clipId,
      selectedClipId: null
    }));
  }

  /** Delete selected timeline clip */
  function deleteSelectedTimelineClip() {
    const selectedId = $playbackStore.selectedTimelineClipId;
    if (!selectedId) return;

    console.log("Deleting timeline clip:", selectedId);

    // Save state for undo
    undoManager.saveState();

    timelineStore.update(state => {
      const remainingClips = state.clips.filter(c => c.id !== selectedId);

      // Recalculate timeline duration
      let maxDuration = 0;
      for (const clip of remainingClips) {
        const clipEnd = clip.startTime + clip.duration;
        if (clipEnd > maxDuration) {
          maxDuration = clipEnd;
        }
      }

      return {
        ...state,
        clips: remainingClips,
        duration: maxDuration
      };
    });

    // Clear selection
    playbackStore.update(state => ({
      ...state,
      selectedTimelineClipId: null
    }));

    updateUndoRedoState();
  }

  /** Handle keyboard shortcuts */
  function handleKeyDown(e) {
    // Don't process shortcuts if user is typing in an input
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
      return;
    }

    // Undo: Ctrl+Z (Windows/Linux) or Cmd+Z (Mac)
    if ((e.ctrlKey || e.metaKey) && e.key === 'z' && !e.shiftKey) {
      e.preventDefault();
      if (undoManager.undo()) {
        updateUndoRedoState();
        console.log('Undo performed');
      }
      return;
    }

    // Redo: Ctrl+Shift+Z or Ctrl+Y (Windows/Linux) or Cmd+Shift+Z (Mac)
    if ((e.ctrlKey || e.metaKey) && (e.shiftKey && e.key === 'z' || e.key === 'y')) {
      e.preventDefault();
      if (undoManager.redo()) {
        updateUndoRedoState();
        console.log('Redo performed');
      }
      return;
    }

    // Delete or Backspace key
    if (e.key === 'Delete' || e.key === 'Backspace') {
      e.preventDefault();
      deleteSelectedTimelineClip();
    }
  }

  /** @param {number} trackIndex */
  function getClipsForTrack(trackIndex) {
    return $timelineStore.clips.filter(c => c.track === trackIndex);
  }

  /** @param {string} clipId */
  function getClipFilename(clipId) {
    return $clipsStore.find(c => c.id === clipId)?.filename || 'Unknown';
  }

  /**
   * Get filmstrip background style for a timeline clip
   * @param {string} clipId - Source clip ID
   * @param {number} trimStart - Trim start time in seconds
   * @param {number} trimEnd - Trim end time in seconds
   * @param {number} clipWidth - Width of clip in pixels
   * @param {number} currentTrackHeight - Current track height in pixels
   * @returns {string} CSS background style
   */
  function getFilmstripStyle(clipId, trimStart, trimEnd, clipWidth, currentTrackHeight) {
    const sourceClip = $clipsStore.find(c => c.id === clipId);
    if (!sourceClip) return '';

    // Use thumbnail as fallback if filmstrip not available
    if (!sourceClip.filmstrip && sourceClip.thumbnail) {
      return `background-image: url(${sourceClip.thumbnail}); background-size: cover; background-position: center;`;
    }

    if (!sourceClip.filmstrip) return '';

    const filmstripUrl = convertFileSrc(sourceClip.filmstrip);
    const frameCount = sourceClip.filmstripFrameCount || 20;
    const frameHeight = currentTrackHeight - 8; // Track height minus padding/margins
    const frameWidth = frameHeight * (16 / 9); // Maintain 16:9 aspect ratio

    // How many frames can fit in the visible clip width?
    const visibleFrameCount = Math.ceil(clipWidth / frameWidth);

    // Calculate which frames to show based on trim
    const clipDuration = trimEnd - trimStart;
    const sourceDuration = sourceClip.duration;

    // Build background style with multiple positioned backgrounds
    const backgrounds = [];
    for (let i = 0; i < visibleFrameCount; i++) {
      // Time in the trimmed clip
      const timeInClip = (i * clipDuration) / visibleFrameCount;
      // Corresponding time in source video
      const timeInSource = trimStart + timeInClip;

      // Which frame index in our filmstrip?
      const frameIndex = Math.floor((timeInSource / sourceDuration) * frameCount);
      const clampedIndex = Math.max(0, Math.min(frameIndex, frameCount - 1));

      // Position this frame
      const xPos = i * frameWidth;
      const yPos = -clampedIndex * frameHeight;

      backgrounds.push(`url(${filmstripUrl}) ${xPos}px ${yPos}px / ${frameWidth}px ${frameHeight * frameCount}px no-repeat`);
    }

    return `background: ${backgrounds.join(', ')};`;
  }

  // Auto-generate filmstrips for clips on timeline
  $effect(() => {
    const timelineClipIds = new Set($timelineStore.clips.map(tc => tc.clipId));

    timelineClipIds.forEach(clipId => {
      const sourceClip = $clipsStore.find(c => c.id === clipId);
      if (sourceClip && !sourceClip.filmstrip) {
        console.log(`Auto-generating filmstrip for clip: ${clipId}`);
        generateFilmstripForClip(clipId);
      }
    });
  });

  // Measure actual track height from rendered track element
  $effect(() => {
    if (!track1Element) return;

    const resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        // Use the actual rendered height of the track
        trackHeight = entry.contentRect.height;
      }
    });

    resizeObserver.observe(track1Element);

    return () => {
      resizeObserver.disconnect();
    };
  });

  function zoom_in() {
    zoom = Math.min(zoom + 10, MAX_ZOOM);
  }

  function zoom_out() {
    zoom = Math.max(zoom - 10, MIN_ZOOM);
  }

  function zoom_reset() {
    zoom = 100;
  }

  /**
   * Start dragging the playhead
   * @param {MouseEvent} e
   */
  function startPlayheadDrag(e) {
    e.preventDefault();
    e.stopPropagation();
    isDraggingPlayhead = true;

    // Pause video while scrubbing
    const wasPlaying = $playbackStore.isPlaying;
    if (wasPlaying && videoElement) {
      videoElement.pause();
      playbackStore.update(state => ({
        ...state,
        isPlaying: false
      }));
    }

    /** @param {MouseEvent} moveEvent */
    function handleMouseMove(moveEvent) {
      if (!timelineElement || !isDraggingPlayhead) return;

      const rect = timelineElement.getBoundingClientRect();
      const x = Math.max(0, moveEvent.clientX - rect.left);
      const newTime = Math.max(0, Math.min(x / zoom, effectiveTimelineDuration));

      // Update playback position
      playbackStore.update(state => ({
        ...state,
        currentTime: newTime
      }));

      // Update video element
      if (videoElement) {
        // Find clip at new position
        const clipAtPosition = $timelineStore.clips
          .filter(c => c.track === 0)
          .find(c => newTime >= c.startTime && newTime < c.startTime + c.duration);

        if (clipAtPosition) {
          // Calculate offset within clip
          const offset = newTime - clipAtPosition.startTime;
          videoElement.currentTime = clipAtPosition.trimStart + offset;
        }
      }
    }

    function handleMouseUp() {
      isDraggingPlayhead = false;
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);

      // Resume playback if it was playing before
      if (wasPlaying && videoElement) {
        videoElement.play();
        playbackStore.update(state => ({
          ...state,
          isPlaying: true
        }));
      }
    }

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  }

  /**
   * Start trimming a clip (dragging left or right edge)
   * @param {MouseEvent} e
   * @param {string} clipId
   * @param {'start' | 'end'} side
   */
  function startTrimDrag(e, clipId, side) {
    e.preventDefault();
    e.stopPropagation();

    const clip = $timelineStore.clips.find(c => c.id === clipId);
    if (!clip) return;

    const sourceClip = $clipsStore.find(c => c.id === clip.clipId);
    if (!sourceClip) return;

    // Save state for undo before starting trim
    undoManager.saveState();

    const startX = e.clientX;
    const startTrimValue = side === 'start' ? clip.trimStart : clip.trimEnd;
    const startTimelinePosition = clip.startTime;

    /** @param {MouseEvent} moveEvent */
    function handleMouseMove(moveEvent) {
      const deltaX = moveEvent.clientX - startX;
      const deltaTime = deltaX / zoom;

      if (side === 'start') {
        // Trimming from the start
        const newTrimStart = Math.max(0, Math.min(startTrimValue + deltaTime, clip.trimEnd - 0.1));
        const newStartTime = startTimelinePosition + deltaTime;
        const newDuration = clip.trimEnd - newTrimStart;

        timelineStore.update(state => ({
          ...state,
          clips: state.clips.map(c =>
            c.id === clipId
              ? {
                  ...c,
                  trimStart: newTrimStart,
                  startTime: Math.max(0, newStartTime),
                  duration: newDuration
                }
              : c
          )
        }));
      } else {
        // Trimming from the end
        const newTrimEnd = Math.max(clip.trimStart + 0.1, Math.min(startTrimValue + deltaTime, sourceClip.duration));
        const newDuration = newTrimEnd - clip.trimStart;

        timelineStore.update(state => ({
          ...state,
          clips: state.clips.map(c =>
            c.id === clipId
              ? {
                  ...c,
                  trimEnd: newTrimEnd,
                  duration: newDuration
                }
              : c
          )
        }));
      }
    }

    function handleMouseUp() {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);

      // Recalculate timeline duration
      const maxDuration = $timelineStore.clips.reduce((max, c) => {
        const clipEnd = c.startTime + c.duration;
        return Math.max(max, clipEnd);
      }, 0);

      timelineStore.update(state => ({
        ...state,
        duration: maxDuration
      }));

      updateUndoRedoState();
    }

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  }
</script>

<div
  class="flex flex-col h-full border-t bg-background"
  style="max-height: 204px"
  onkeydown={handleKeyDown}
  tabindex="-1"
>
  <!-- Timeline Container -->
  <div class="flex flex-col h-full bg-card border-t" bind:this={timelineContainer}>
    <!-- Timeline Tracks (includes time ruler that scrolls with content) -->
    <ScrollArea orientation="horizontal" class="h-full">
      <div class="flex flex-col h-full">
        <!-- Time Ruler -->
        <div class="relative bg-muted border-b flex" style="height: 24px; flex-shrink: 0">
          <!-- Zoom controls in time ruler -->
          <div class="w-[120px] shrink-0 border-r flex items-center justify-center gap-0.5 px-1">
            <Button variant="ghost" size="sm" onclick={zoom_out} title="Zoom out" class="h-5 w-5 p-0" disabled={false}>
              <ZoomOut class="w-3 h-3" />
            </Button>
            <Button variant="ghost" size="sm" onclick={zoom_reset} title="Reset zoom" class="h-5 w-5 p-0" disabled={false}>
              <Maximize2 class="w-3 h-3" />
            </Button>
            <Button variant="ghost" size="sm" onclick={zoom_in} title="Zoom in" class="h-5 w-5 p-0" disabled={false}>
              <ZoomIn class="w-3 h-3" />
            </Button>
          </div>
          <div style="width: {timelineWidth}px" class="relative h-full">
            {#each getTimeMarkers(effectiveTimelineDuration) as time}
              <div style="left: {time * zoom}px" class="absolute text-[10px] text-muted-foreground border-l border-muted-foreground h-full pt-0.5 px-1">
                {formatTime(time)}
              </div>
            {/each}
          </div>
        </div>
        <!-- Track 1 (Main Video) -->
        <div
          bind:this={track1Element}
          class="relative border-b transition-colors flex flex-1"
          style="max-height: {MAX_TRACK_HEIGHT}px"
          role="region"
          aria-label="Timeline track 1"
        >
          <div class="w-[120px] px-2 py-1 bg-muted border-r text-xs font-semibold text-muted-foreground whitespace-nowrap flex items-center justify-center shrink-0">
            Track 1 (Main)
          </div>
          <div
            bind:this={timelineElement}
            class={`relative cursor-crosshair transition-all ${
              isDraggingOverTrack1
                ? 'bg-primary/10 ring-2 ring-primary ring-inset'
                : 'hover:bg-muted/30'
            }`}
            style="width: {timelineWidth}px; min-width: 100%;"
            onclick={(e) => handleTimelineClick(e, 0)}
            onkeydown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
              }
            }}
            ondrop={(e) => handleDrop(e, 0)}
            ondragover={(e) => handleDragOver(e, 0)}
            ondragleave={() => handleDragLeave(0)}
            role="button"
            tabindex="0"
          >
            <!-- Playhead for Track 1 -->
            {#if $timelineStore.clips.length > 0}
              <div
                class="absolute h-full z-50 cursor-ew-resize"
                style="left: {playheadPosition}px"
                title={formatTime($playbackStore.currentTime)}
                onmousedown={startPlayheadDrag}
                role="button"
                tabindex="0"
                aria-label="Playhead - drag to scrub"
              >
                <!-- Playhead line (entire line is draggable) -->
                <div class="absolute w-0.5 h-full bg-destructive shadow-md -translate-x-1/2"></div>
              </div>
            {/if}
            {#each getClipsForTrack(0) as timelineClip (timelineClip.id)}
              <div
                class={`absolute text-primary-foreground text-xs px-2 rounded flex items-center justify-between cursor-pointer select-none overflow-hidden transition-all hover:brightness-110 ${
                  $playbackStore.selectedTimelineClipId === timelineClip.id
                    ? 'ring-2 ring-ring shadow-lg brightness-110'
                    : ''
                }`}
                style="
                  top: 4px;
                  height: {trackHeight - 8}px;
                  left: {timelineClip.startTime * zoom}px;
                  width: {(timelineClip.trimEnd - timelineClip.trimStart) * zoom}px;
                  {getFilmstripStyle(
                    timelineClip.clipId,
                    timelineClip.trimStart,
                    timelineClip.trimEnd,
                    (timelineClip.trimEnd - timelineClip.trimStart) * zoom,
                    trackHeight
                  )}
                "
                onclick={(e) => {
                  e.stopPropagation();
                  selectTimelineClip(timelineClip.id);
                }}
                onkeydown={(e) => {
                  if (e.key === 'Enter' || e.key === ' ') {
                    e.preventDefault();
                    selectTimelineClip(timelineClip.id);
                  }
                }}
                role="button"
                tabindex="0"
              >
                <!-- Left trim handle -->
                <div
                  class="absolute left-0 top-0 w-2 h-full bg-primary-foreground/30 hover:bg-primary-foreground/50 cursor-ew-resize"
                  onmousedown={(e) => startTrimDrag(e, timelineClip.id, 'start')}
                  title="Trim start"
                ></div>

                <span class="truncate text-xs flex-1 px-1">
                  {getClipFilename(timelineClip.clipId)}
                </span>

                <!-- Right trim handle -->
                <div
                  class="absolute right-0 top-0 w-2 h-full bg-primary-foreground/30 hover:bg-primary-foreground/50 cursor-ew-resize"
                  onmousedown={(e) => startTrimDrag(e, timelineClip.id, 'end')}
                  title="Trim end"
                ></div>
              </div>
            {/each}
          </div>
        </div>

        <!-- Track 2 (Overlay/PiP) -->
        <div
          class="relative transition-colors flex flex-1"
          style="max-height: {MAX_TRACK_HEIGHT}px"
          role="region"
          aria-label="Timeline track 2"
        >
          <div class="w-[120px] px-2 py-1 bg-muted border-r text-xs font-semibold text-muted-foreground whitespace-nowrap flex items-center justify-center shrink-0">
            Track 2 (Overlay)
          </div>
          <div
            class={`relative cursor-crosshair transition-all ${
              isDraggingOverTrack2
                ? 'bg-primary/10 ring-2 ring-primary ring-inset'
                : 'hover:bg-muted/30'
            }`}
            style="width: {timelineWidth}px; min-width: 100%;"
            onclick={(e) => handleTimelineClick(e, 1)}
            onkeydown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
              }
            }}
            ondrop={(e) => handleDrop(e, 1)}
            ondragover={(e) => handleDragOver(e, 1)}
            ondragleave={() => handleDragLeave(1)}
            role="button"
            tabindex="0"
          >
            <!-- Playhead for Track 2 -->
            {#if $timelineStore.clips.length > 0}
              <div
                class="absolute h-full z-50 cursor-ew-resize"
                style="left: {playheadPosition}px"
                title={formatTime($playbackStore.currentTime)}
                onmousedown={startPlayheadDrag}
                role="button"
                tabindex="0"
                aria-label="Playhead - drag to scrub"
              >
                <!-- Playhead line (entire line is draggable) -->
                <div class="absolute w-0.5 h-full bg-destructive shadow-md -translate-x-1/2"></div>
              </div>
            {/if}
            {#each getClipsForTrack(1) as timelineClip (timelineClip.id)}
              <div
                class={`absolute text-primary-foreground text-xs px-2 rounded flex items-center justify-between cursor-pointer select-none overflow-hidden transition-all hover:brightness-110 ${
                  $playbackStore.selectedTimelineClipId === timelineClip.id
                    ? 'ring-2 ring-ring shadow-lg brightness-110'
                    : ''
                }`}
                style="
                  top: 4px;
                  height: {trackHeight - 8}px;
                  left: {timelineClip.startTime * zoom}px;
                  width: {(timelineClip.trimEnd - timelineClip.trimStart) * zoom}px;
                  {getFilmstripStyle(
                    timelineClip.clipId,
                    timelineClip.trimStart,
                    timelineClip.trimEnd,
                    (timelineClip.trimEnd - timelineClip.trimStart) * zoom,
                    trackHeight
                  )}
                "
                onclick={(e) => {
                  e.stopPropagation();
                  selectTimelineClip(timelineClip.id);
                }}
                onkeydown={(e) => {
                  if (e.key === 'Enter' || e.key === ' ') {
                    e.preventDefault();
                    selectTimelineClip(timelineClip.id);
                  }
                }}
                role="button"
                tabindex="0"
              >
                <!-- Left trim handle -->
                <div
                  class="absolute left-0 top-0 w-2 h-full bg-primary-foreground/30 hover:bg-primary-foreground/50 cursor-ew-resize"
                  onmousedown={(e) => startTrimDrag(e, timelineClip.id, 'start')}
                  title="Trim start"
                ></div>

                <span class="truncate text-xs flex-1 px-1">
                  {getClipFilename(timelineClip.clipId)}
                </span>

                <!-- Right trim handle -->
                <div
                  class="absolute right-0 top-0 w-2 h-full bg-primary-foreground/30 hover:bg-primary-foreground/50 cursor-ew-resize"
                  onmousedown={(e) => startTrimDrag(e, timelineClip.id, 'end')}
                  title="Trim end"
                ></div>
              </div>
            {/each}
          </div>
        </div>
      </div>
    </ScrollArea>
  </div>
</div>
