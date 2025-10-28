<script>
  import { timelineStore } from '../stores/timeline.js';
  import { playbackStore } from '../stores/playback.js';
  import { clipsStore } from '../stores/clips.js';
  import { Button } from "$lib/components/ui/button";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { ZoomIn, ZoomOut, Maximize2, X } from "@lucide/svelte";

  /**
   * Timeline Component
   * Displays two-track timeline with playhead
   * Supports drag-and-drop clip placement and trimming
   */

  let { videoElement = $bindable(null) } = $props();

  /** @type {HTMLElement | null} */
  let timelineElement = null;
  let zoom = $state(100); // pixels per second
  const MIN_ZOOM = 20;
  const MAX_ZOOM = 300;

  let isDraggingOverTrack1 = $state(false);
  let isDraggingOverTrack2 = $state(false);

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

    timelineStore.update(state => ({
      ...state,
      clips: [...state.clips, timelineClip],
      duration: Math.max(state.duration, startTime + data.duration)
    }));
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
  }

  /** Handle keyboard shortcuts */
  function handleKeyDown(e) {
    // Delete or Backspace key
    if (e.key === 'Delete' || e.key === 'Backspace') {
      // Don't delete if user is typing in an input
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
        return;
      }

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

  function zoom_in() {
    zoom = Math.min(zoom + 20, MAX_ZOOM);
  }

  function zoom_out() {
    zoom = Math.max(zoom - 20, MIN_ZOOM);
  }

  function zoom_reset() {
    zoom = 100;
  }
</script>

<div
  class="flex flex-col h-[150px] border-t bg-background"
  onkeydown={handleKeyDown}
  tabindex="-1"
>
  <!-- Timeline Controls -->
  <div class="flex items-center gap-2 px-3 py-2 bg-muted border-b h-9">
    <Button variant="ghost" size="sm" onclick={zoom_out} title="Zoom out" class="h-7 w-7 p-0" disabled={false}>
      <ZoomOut class="w-3 h-3" />
    </Button>
    <Button variant="ghost" size="sm" onclick={zoom_reset} title="Reset zoom" class="h-7 px-2" disabled={false}>
      <Maximize2 class="w-3 h-3" />
    </Button>
    <Button variant="ghost" size="sm" onclick={zoom_in} title="Zoom in" class="h-7 w-7 p-0" disabled={false}>
      <ZoomIn class="w-3 h-3" />
    </Button>
    <span class="text-xs text-muted-foreground ml-2 min-w-[30px]">{Math.round(zoom / 100 * 100)}%</span>
  </div>

  <!-- Timeline Container -->
  <div class="flex flex-col flex-1 bg-card border-t">
    <!-- Timeline Tracks (includes time ruler that scrolls with content) -->
    <ScrollArea orientation="horizontal" class="flex-1">
      <div class="flex flex-col">
        <!-- Time Ruler -->
        <div class="relative h-6 bg-muted border-b flex">
          <div class="w-[120px] shrink-0 border-r"></div>
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
          class="relative border-b transition-colors h-[45px] flex"
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
            style="width: {timelineWidth}px"
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
            <div
              class="absolute w-0.5 h-full bg-destructive shadow-md z-50 pointer-events-none"
              style="left: {playheadPosition}px"
              title={formatTime($playbackStore.currentTime)}
            ></div>
            {#each getClipsForTrack(0) as timelineClip (timelineClip.id)}
              <div
                class={`absolute top-1 h-7 bg-primary text-primary-foreground text-xs px-2 rounded flex items-center justify-between cursor-pointer select-none overflow-hidden transition-all hover:brightness-110 ${
                  $playbackStore.selectedTimelineClipId === timelineClip.id
                    ? 'ring-2 ring-ring shadow-lg brightness-110'
                    : ''
                }`}
                style="
                  left: {timelineClip.startTime * zoom}px;
                  width: {(timelineClip.trimEnd - timelineClip.trimStart) * zoom}px;
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
                <span class="truncate text-xs flex-1">
                  {getClipFilename(timelineClip.clipId)}
                </span>
                {#if $playbackStore.selectedTimelineClipId === timelineClip.id}
                  <button
                    class="ml-1 shrink-0 w-5 h-5 flex items-center justify-center rounded bg-destructive/80 hover:bg-destructive transition-colors"
                    onclick={(e) => {
                      e.stopPropagation();
                      deleteSelectedTimelineClip();
                    }}
                    title="Delete (Backspace)"
                  >
                    <X class="w-3 h-3" />
                  </button>
                {/if}
              </div>
            {/each}
          </div>
        </div>

        <!-- Track 2 (Overlay/PiP) -->
        <div
          class="relative transition-colors h-[45px] flex"
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
            style="width: {timelineWidth}px"
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
            <div
              class="absolute w-0.5 h-full bg-destructive shadow-md z-50 pointer-events-none"
              style="left: {playheadPosition}px"
              title={formatTime($playbackStore.currentTime)}
            ></div>
            {#each getClipsForTrack(1) as timelineClip (timelineClip.id)}
              <div
                class={`absolute top-1 h-7 bg-primary text-primary-foreground text-xs px-2 rounded flex items-center justify-between cursor-pointer select-none overflow-hidden transition-all hover:brightness-110 ${
                  $playbackStore.selectedTimelineClipId === timelineClip.id
                    ? 'ring-2 ring-ring shadow-lg brightness-110'
                    : ''
                }`}
                style="
                  left: {timelineClip.startTime * zoom}px;
                  width: {(timelineClip.trimEnd - timelineClip.trimStart) * zoom}px;
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
                <span class="truncate text-xs flex-1">
                  {getClipFilename(timelineClip.clipId)}
                </span>
                {#if $playbackStore.selectedTimelineClipId === timelineClip.id}
                  <button
                    class="ml-1 shrink-0 w-5 h-5 flex items-center justify-center rounded bg-destructive/80 hover:bg-destructive transition-colors"
                    onclick={(e) => {
                      e.stopPropagation();
                      deleteSelectedTimelineClip();
                    }}
                    title="Delete (Backspace)"
                  >
                    <X class="w-3 h-3" />
                  </button>
                {/if}
              </div>
            {/each}
          </div>
        </div>
      </div>
    </ScrollArea>
  </div>
</div>
