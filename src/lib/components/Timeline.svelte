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

  // Trim drag state - tracks active trim without updating the clip
  let activeTrimClipId = $state(null);
  let activeTrimSide = $state(null); // 'start' | 'end'
  let trimPreviewOffset = $state(0); // pixels offset from original position
  let isCurrentlyTrimming = $state(false); // disable transitions during drag

  // Clip move drag state - for repositioning clips on timeline
  let isDraggingClip = $state(false);
  let draggedClipId = $state(null);
  let clipDragOffsetX = $state(0); // pixels offset from original position
  let clipDragOffsetY = $state(0); // pixels offset for track detection
  let clipDragTargetTrack = $state(null); // target track during drag

  // Snap state - magnetic alignment to other clips
  const SNAP_THRESHOLD = 0.3; // seconds - snap within 300ms
  let isSnapping = $state(false);
  let snapTargetTime = $state(null); // time position of snap line

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

  /**
   * Get time markers based on duration and zoom level
   * Dynamically adjusts marker density for optimal readability
   * @param {number} duration - Timeline duration in seconds
   * @returns {number[]} Array of time values for markers
   */
  function getTimeMarkers(duration) {
    const markers = [];
    // If timeline is empty (only default duration), just show 0:00
    if ($timelineStore.clips.length === 0) {
      return [0];
    }

    // Target: 80-150px between markers for good readability
    const MIN_MARKER_SPACING = 80; // pixels

    // Calculate optimal step based on zoom level
    // At current zoom, how many seconds fit in MIN_MARKER_SPACING pixels?
    const secondsPerMinSpacing = MIN_MARKER_SPACING / zoom;

    // Choose step from nice round intervals: 0.1s, 0.5s, 1s, 2s, 5s, 10s, 30s, 60s, 120s
    const niceIntervals = [0.1, 0.5, 1, 2, 5, 10, 30, 60, 120, 300, 600];
    let step = niceIntervals[niceIntervals.length - 1]; // Default to largest

    for (const interval of niceIntervals) {
      if (interval >= secondsPerMinSpacing) {
        step = interval;
        break;
      }
    }

    // Generate markers
    for (let i = 0; i <= Math.ceil(duration); i += step) {
      markers.push(i);
    }

    return markers;
  }

  /**
   * Format time for display on ruler
   * Shows subsecond precision when zoomed in
   * @param {number} seconds - Time in seconds
   * @returns {string} Formatted time string
   */
  function formatTime(seconds) {
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = Math.floor(seconds % 60);
    const ms = Math.floor((seconds % 1) * 10); // tenths of a second

    // Show subseconds when zoomed in enough (> 200 pixels per second)
    if (zoom > 200 && ms > 0) {
      if (h > 0) {
        return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}.${ms}`;
      }
      return `${m}:${s.toString().padStart(2, '0')}.${ms}`;
    }

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

    const clip = $timelineStore.clips.find(c => c.id === clipId);
    if (!clip) return;

    // Update selection and move playhead to start of clip
    playbackStore.update(state => ({
      ...state,
      selectedTimelineClipId: clipId,
      selectedClipId: null,
      currentTime: clip.startTime, // Move playhead to clip start
      isPlaying: false // Pause playback when selecting a clip
    }));

    // Seek video to clip position after a brief delay to allow source to load
    if (videoElement) {
      setTimeout(() => {
        if (videoElement) {
          videoElement.pause();
          videoElement.currentTime = clip.trimStart;
        }
      }, 100);
    }
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
      return;
    }

    // S key: Split clip at playhead
    if (e.key === 's' || e.key === 'S') {
      e.preventDefault();
      splitTimelineClip();
      return;
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

  // Listen for custom events from Controls component
  $effect(() => {
    /**
     * Handle split clip event from Controls
     */
    function handleSplitEvent() {
      splitTimelineClip();
    }

    /**
     * Handle delete clip event from Controls
     */
    function handleDeleteEvent() {
      deleteSelectedTimelineClip();
    }

    window.addEventListener('split-clip', handleSplitEvent);
    window.addEventListener('delete-clip', handleDeleteEvent);

    return () => {
      window.removeEventListener('split-clip', handleSplitEvent);
      window.removeEventListener('delete-clip', handleDeleteEvent);
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
   * Split timeline clip at playhead position
   * Creates two clips from the selected clip at the current playhead position
   */
  function splitTimelineClip() {
    const selectedId = $playbackStore.selectedTimelineClipId;
    const splitTime = $playbackStore.currentTime;

    if (!selectedId) {
      console.log("No clip selected for split");
      return;
    }

    const clip = $timelineStore.clips.find(c => c.id === selectedId);
    if (!clip) {
      console.log("Selected clip not found");
      return;
    }

    // Validate: playhead must be within clip bounds (not at edges)
    const clipEnd = clip.startTime + clip.duration;
    if (splitTime <= clip.startTime || splitTime >= clipEnd) {
      console.log("Playhead must be within clip bounds to split");
      return;
    }

    // Ensure split point is not too close to edges (minimum 0.1s on each side)
    if (splitTime - clip.startTime < 0.1 || clipEnd - splitTime < 0.1) {
      console.log("Split point too close to clip edge");
      return;
    }

    console.log(`Splitting clip ${selectedId} at ${splitTime}s`);

    // Save state for undo
    undoManager.saveState();

    // Calculate split points
    const offsetInClip = splitTime - clip.startTime;

    // Clip 1 (before split)
    const clip1 = {
      id: `timeline-clip-${Date.now()}-${Math.random()}`,
      clipId: clip.clipId,
      track: clip.track,
      startTime: clip.startTime,
      trimStart: clip.trimStart,
      trimEnd: clip.trimStart + offsetInClip,
      duration: offsetInClip
    };

    // Clip 2 (after split)
    const clip2 = {
      id: `timeline-clip-${Date.now() + 1}-${Math.random()}`,
      clipId: clip.clipId,
      track: clip.track,
      startTime: splitTime,
      trimStart: clip.trimStart + offsetInClip,
      trimEnd: clip.trimEnd,
      duration: clip.duration - offsetInClip
    };

    console.log("Split clips:", { clip1, clip2 });

    // Update timeline store: replace original clip with two new clips
    timelineStore.update(state => {
      const otherClips = state.clips.filter(c => c.id !== selectedId);
      return {
        ...state,
        clips: [...otherClips, clip1, clip2]
      };
    });

    // Recalculate timeline duration
    let maxDuration = 0;
    for (const c of $timelineStore.clips) {
      const clipEnd = c.startTime + c.duration;
      if (clipEnd > maxDuration) {
        maxDuration = clipEnd;
      }
    }

    timelineStore.update(state => ({
      ...state,
      duration: maxDuration
    }));

    // Select the second clip (after split) and move playhead to its start
    playbackStore.update(state => ({
      ...state,
      selectedTimelineClipId: clip2.id,
      currentTime: splitTime
    }));

    updateUndoRedoState();
  }

  /**
   * Start trimming a clip (dragging left or right edge)
   * Filmstrip stays static, grey overlay slides to show trim preview
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

    // Set active trim state for visual feedback
    activeTrimClipId = clipId;
    activeTrimSide = side;
    trimPreviewOffset = 0;
    isCurrentlyTrimming = true;

    const startX = e.clientX;
    const startTrimValue = side === 'start' ? clip.trimStart : clip.trimEnd;
    const startTimelinePosition = clip.startTime;

    // Store final values to apply on mouseup
    let finalTrimStart = clip.trimStart;
    let finalTrimEnd = clip.trimEnd;
    let finalStartTime = clip.startTime;

    /** @param {MouseEvent} moveEvent */
    function handleMouseMove(moveEvent) {
      const deltaX = moveEvent.clientX - startX;
      const deltaTime = deltaX / zoom;

      // Just update the preview offset - don't touch the store
      trimPreviewOffset = deltaX;

      // Calculate final values for mouseup
      if (side === 'start') {
        finalTrimStart = Math.max(0, Math.min(startTrimValue + deltaTime, clip.trimEnd - 0.1));
        finalStartTime = Math.max(0, startTimelinePosition + deltaTime);

        // Update playhead to show the frame at the trim start position
        playbackStore.update(state => ({
          ...state,
          currentTime: finalStartTime,
          isPlaying: false
        }));

        // Update video to show the trimmed frame
        if (videoElement) {
          videoElement.currentTime = finalTrimStart;
        }
      } else {
        finalTrimEnd = Math.max(clip.trimStart + 0.1, Math.min(startTrimValue + deltaTime, sourceClip.duration));

        // Update playhead to show the frame at the trim end position
        const endTimelinePosition = clip.startTime + (finalTrimEnd - clip.trimStart);
        playbackStore.update(state => ({
          ...state,
          currentTime: endTimelinePosition,
          isPlaying: false
        }));

        // Update video to show the trimmed frame
        if (videoElement) {
          videoElement.currentTime = finalTrimEnd;
        }
      }
    }

    function handleMouseUp() {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);

      // Clear preview overlay immediately
      activeTrimClipId = null;
      activeTrimSide = null;
      trimPreviewOffset = 0;

      // Apply the trim changes to the store
      const newDuration = finalTrimEnd - finalTrimStart;

      timelineStore.update(state => ({
        ...state,
        clips: state.clips.map(c =>
          c.id === clipId
            ? {
                ...c,
                trimStart: finalTrimStart,
                trimEnd: finalTrimEnd,
                startTime: finalStartTime,
                duration: newDuration
              }
            : c
        )
      }));

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

      // Re-enable transitions after a brief delay
      setTimeout(() => {
        isCurrentlyTrimming = false;
      }, 300); // Match CSS transition duration
    }

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  }

  /**
   * Start dragging a clip to move it on the timeline
   * @param {MouseEvent} e
   * @param {string} clipId
   */
  function startClipDrag(e, clipId) {
    // Only start drag on left mouse button
    if (e.button !== 0) return;

    e.preventDefault();
    e.stopPropagation();

    const clip = $timelineStore.clips.find(c => c.id === clipId);
    if (!clip) return;

    // Save state for undo before starting drag
    undoManager.saveState();

    // Set active drag state
    isDraggingClip = true;
    draggedClipId = clipId;
    clipDragOffsetX = 0;
    clipDragOffsetY = 0;
    clipDragTargetTrack = clip.track;

    const startX = e.clientX;
    const startY = e.clientY;
    const originalStartTime = clip.startTime;
    const originalTrack = clip.track;

    // Store final values to apply on mouseup
    let finalStartTime = clip.startTime;
    let finalTrack = clip.track;

    /** @param {MouseEvent} moveEvent */
    function handleMouseMove(moveEvent) {
      const deltaX = moveEvent.clientX - startX;
      const deltaY = moveEvent.clientY - startY;

      // Update preview offsets
      clipDragOffsetX = deltaX;
      clipDragOffsetY = deltaY;

      // Calculate new start time
      const deltaTime = deltaX / zoom;
      let rawStartTime = Math.max(0, originalStartTime + deltaTime);

      // Determine target track based on vertical movement
      // If moved up significantly from Track 1, go to Track 0 (and vice versa)
      if (originalTrack === 0 && deltaY > 50) {
        finalTrack = 1;
        clipDragTargetTrack = 1;
      } else if (originalTrack === 1 && deltaY < -50) {
        finalTrack = 0;
        clipDragTargetTrack = 0;
      } else {
        finalTrack = originalTrack;
        clipDragTargetTrack = originalTrack;
      }

      // Magnetic snap to other clips on target track
      const otherClips = $timelineStore.clips.filter(c =>
        c.id !== clipId && c.track === finalTrack
      );

      // Build snap points: clip starts and ends
      const snapPoints = [];
      for (const otherClip of otherClips) {
        snapPoints.push(otherClip.startTime); // Snap to start
        snapPoints.push(otherClip.startTime + otherClip.duration); // Snap to end
      }

      // Also snap to timeline start (0)
      snapPoints.push(0);

      // Find closest snap point within threshold
      let closestSnapPoint = null;
      let minDistance = SNAP_THRESHOLD;

      for (const snapPoint of snapPoints) {
        const distance = Math.abs(rawStartTime - snapPoint);
        if (distance < minDistance) {
          minDistance = distance;
          closestSnapPoint = snapPoint;
        }
      }

      // Apply snap if found
      if (closestSnapPoint !== null) {
        finalStartTime = closestSnapPoint;
        isSnapping = true;
        snapTargetTime = closestSnapPoint;
      } else {
        finalStartTime = rawStartTime;
        isSnapping = false;
        snapTargetTime = null;
      }

      // Update playhead to show current position
      playbackStore.update(state => ({
        ...state,
        currentTime: finalStartTime,
        isPlaying: false
      }));
    }

    function handleMouseUp() {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);

      // Clear drag state
      isDraggingClip = false;
      draggedClipId = null;
      clipDragOffsetX = 0;
      clipDragOffsetY = 0;
      clipDragTargetTrack = null;

      // Clear snap state
      isSnapping = false;
      snapTargetTime = null;

      // Apply the position changes to the store
      timelineStore.update(state => ({
        ...state,
        clips: state.clips.map(c =>
          c.id === clipId
            ? {
                ...c,
                startTime: finalStartTime,
                track: finalTrack
              }
            : c
        )
      }));

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
  onkeydown={handleKeyDown}
  tabindex="-1"
  role="application"
  aria-label="Timeline editor"
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
            <Button variant="ghost" size="sm" onclick={zoom_out} title="Zoom out" class="h-5 w-5 p-0 active:scale-90 transition-transform">
              <ZoomOut class="w-3 h-3" />
            </Button>
            <Button variant="ghost" size="sm" onclick={zoom_reset} title="Reset zoom" class="h-5 w-5 p-0 active:scale-90 transition-transform">
              <Maximize2 class="w-3 h-3" />
            </Button>
            <Button variant="ghost" size="sm" onclick={zoom_in} title="Zoom in" class="h-5 w-5 p-0 active:scale-90 transition-transform">
              <ZoomIn class="w-3 h-3" />
            </Button>
          </div>
          <div style="width: {timelineWidth}px" class="relative h-full">
            {#each getTimeMarkers(effectiveTimelineDuration) as time (time)}
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
              isDraggingOverTrack1 || (isDraggingClip && clipDragTargetTrack === 0)
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
            <!-- Snap indicator line -->
            {#if isSnapping && snapTargetTime !== null}
              <div
                class="absolute h-full z-40 pointer-events-none"
                style="left: {snapTargetTime * zoom}px"
              >
                <div class="absolute w-1 h-full bg-yellow-400 shadow-lg -translate-x-1/2 animate-pulse"></div>
              </div>
            {/if}

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
                class={`absolute text-primary-foreground text-xs px-2 rounded flex items-center justify-between cursor-move select-none overflow-hidden hover:brightness-105 ${
                  $playbackStore.selectedTimelineClipId === timelineClip.id
                    ? 'ring-2 ring-primary shadow-lg brightness-110'
                    : ''
                } ${draggedClipId === timelineClip.id ? 'ring-4 ring-primary/50 brightness-125 shadow-2xl opacity-80' : ''} ${activeTrimClipId !== timelineClip.id && !isCurrentlyTrimming && !isDraggingClip ? 'transition-all duration-200 ease-out' : ''}`}
                style="
                  top: 4px;
                  height: {trackHeight - 8}px;
                  left: {timelineClip.startTime * zoom + (draggedClipId === timelineClip.id ? clipDragOffsetX : 0)}px;
                  width: {(timelineClip.trimEnd - timelineClip.trimStart) * zoom}px;
                  transform: translateY({draggedClipId === timelineClip.id ? clipDragOffsetY : 0}px);
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
                <!-- Grey overlay for trimmed-out area (left side) -->
                {#if activeTrimClipId === timelineClip.id && activeTrimSide === 'start'}
                  <div
                    class="absolute top-0 left-0 h-full bg-black/60 pointer-events-none"
                    style="width: {Math.max(0, trimPreviewOffset)}px;"
                  ></div>
                {/if}

                <!-- Grey overlay for trimmed-out area (right side) -->
                {#if activeTrimClipId === timelineClip.id && activeTrimSide === 'end'}
                  <div
                    class="absolute top-0 right-0 h-full bg-black/60 pointer-events-none"
                    style="width: {Math.max(0, -trimPreviewOffset)}px;"
                  ></div>
                {/if}

                <!-- Left trim handle -->
                <div
                  class="absolute left-0 top-0 w-2 h-full bg-primary-foreground/30 hover:bg-primary hover:scale-x-150 cursor-ew-resize z-10 transition-all duration-150"
                  onmousedown={(e) => startTrimDrag(e, timelineClip.id, 'start')}
                  role="slider"
                  aria-label="Trim clip start"
                  tabindex="0"
                  title="Trim start"
                ></div>

                <span
                  class="truncate text-xs flex-1 px-1 relative z-0 cursor-move"
                  onmousedown={(e) => startClipDrag(e, timelineClip.id)}
                  role="button"
                  tabindex="0"
                  title="Drag to move clip"
                >
                  {getClipFilename(timelineClip.clipId)}
                </span>

                <!-- Right trim handle -->
                <div
                  class="absolute right-0 top-0 w-2 h-full bg-primary-foreground/30 hover:bg-primary hover:scale-x-150 cursor-ew-resize z-10 transition-all duration-150"
                  onmousedown={(e) => startTrimDrag(e, timelineClip.id, 'end')}
                  role="slider"
                  aria-label="Trim clip end"
                  tabindex="0"
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
              isDraggingOverTrack2 || (isDraggingClip && clipDragTargetTrack === 1)
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
            <!-- Snap indicator line -->
            {#if isSnapping && snapTargetTime !== null}
              <div
                class="absolute h-full z-40 pointer-events-none"
                style="left: {snapTargetTime * zoom}px"
              >
                <div class="absolute w-1 h-full bg-yellow-400 shadow-lg -translate-x-1/2 animate-pulse"></div>
              </div>
            {/if}

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
                class={`absolute text-primary-foreground text-xs px-2 rounded flex items-center justify-between cursor-move select-none overflow-hidden hover:brightness-105 ${
                  $playbackStore.selectedTimelineClipId === timelineClip.id
                    ? 'ring-2 ring-primary shadow-lg brightness-110'
                    : ''
                } ${draggedClipId === timelineClip.id ? 'ring-4 ring-primary/50 brightness-125 shadow-2xl opacity-80' : ''} ${activeTrimClipId !== timelineClip.id && !isCurrentlyTrimming && !isDraggingClip ? 'transition-all duration-200 ease-out' : ''}`}
                style="
                  top: 4px;
                  height: {trackHeight - 8}px;
                  left: {timelineClip.startTime * zoom + (draggedClipId === timelineClip.id ? clipDragOffsetX : 0)}px;
                  width: {(timelineClip.trimEnd - timelineClip.trimStart) * zoom}px;
                  transform: translateY({draggedClipId === timelineClip.id ? clipDragOffsetY : 0}px);
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
                <!-- Grey overlay for trimmed-out area (left side) -->
                {#if activeTrimClipId === timelineClip.id && activeTrimSide === 'start'}
                  <div
                    class="absolute top-0 left-0 h-full bg-black/60 pointer-events-none"
                    style="width: {Math.max(0, trimPreviewOffset)}px;"
                  ></div>
                {/if}

                <!-- Grey overlay for trimmed-out area (right side) -->
                {#if activeTrimClipId === timelineClip.id && activeTrimSide === 'end'}
                  <div
                    class="absolute top-0 right-0 h-full bg-black/60 pointer-events-none"
                    style="width: {Math.max(0, -trimPreviewOffset)}px;"
                  ></div>
                {/if}

                <!-- Left trim handle -->
                <div
                  class="absolute left-0 top-0 w-2 h-full bg-primary-foreground/30 hover:bg-primary hover:scale-x-150 cursor-ew-resize z-10 transition-all duration-150"
                  onmousedown={(e) => startTrimDrag(e, timelineClip.id, 'start')}
                  role="slider"
                  aria-label="Trim clip start"
                  tabindex="0"
                  title="Trim start"
                ></div>

                <span
                  class="truncate text-xs flex-1 px-1 relative z-0 cursor-move"
                  onmousedown={(e) => startClipDrag(e, timelineClip.id)}
                  role="button"
                  tabindex="0"
                  title="Drag to move clip"
                >
                  {getClipFilename(timelineClip.clipId)}
                </span>

                <!-- Right trim handle -->
                <div
                  class="absolute right-0 top-0 w-2 h-full bg-primary-foreground/30 hover:bg-primary hover:scale-x-150 cursor-ew-resize z-10 transition-all duration-150"
                  onmousedown={(e) => startTrimDrag(e, timelineClip.id, 'end')}
                  role="slider"
                  aria-label="Trim clip end"
                  tabindex="0"
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
