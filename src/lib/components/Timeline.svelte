<script>
  import { timelineStore } from '../stores/timeline.js';
  import { playbackStore } from '../stores/playback.js';
  import { clipsStore } from '../stores/clips.js';
  import { Button } from "$lib/components/ui/button";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { ZoomIn, ZoomOut, Maximize2 } from "@lucide/svelte";

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
      currentTime: newTime
    }));

    // Seek video to clicked position
    if (videoElement) {
      videoElement.currentTime = newTime;
    }
  }

  /** @param {DragEvent} e
   *  @param {number} trackIndex */
  function handleDrop(e, trackIndex) {
    e.preventDefault();
    e.stopPropagation();

    if (!e.dataTransfer || !e.currentTarget) return;
    const data = JSON.parse(e.dataTransfer.getData('application/json'));

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

  /** @param {DragEvent} e */
  function handleDragOver(e) {
    e.preventDefault();
    if (e.dataTransfer) {
      e.dataTransfer.dropEffect = 'copy';
    }
  }

  /** @param {string} clipId */
  function selectTimelineClip(clipId) {
    playbackStore.update(state => ({
      ...state,
      selectedTimelineClipId: clipId,
      selectedClipId: null
    }));
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

<div class="flex flex-col h-[150px] border-t bg-background">
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
    <!-- Time Ruler -->
    <div class="relative h-6 bg-muted border-b overflow-hidden flex">
      <div class="w-[120px] shrink-0 border-r"></div>
      <div style="width: {timelineWidth}px" class="relative h-full">
        {#each getTimeMarkers(effectiveTimelineDuration) as time}
          <div style="left: {time * zoom}px" class="absolute text-[10px] text-muted-foreground border-l border-muted-foreground h-full pt-0.5 px-1">
            {formatTime(time)}
          </div>
        {/each}
      </div>
    </div>

    <!-- Timeline Tracks -->
    <ScrollArea orientation="horizontal" class="flex-1">
      <div class="relative flex flex-col">
        <!-- Track 1 (Main Video) -->
        <div
          class="relative border-b hover:bg-muted/50 transition-colors h-[45px] flex"
          role="region"
          aria-label="Timeline track 1"
        >
          <div class="w-[120px] px-2 py-1 bg-muted border-r text-xs font-semibold text-muted-foreground whitespace-nowrap flex items-center justify-center shrink-0">
            Track 1 (Main)
          </div>
          <div
            bind:this={timelineElement}
            class="relative cursor-crosshair"
            style="width: {timelineWidth}px"
            onclick={(e) => handleTimelineClick(e, 0)}
            onkeydown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
              }
            }}
            ondrop={(e) => handleDrop(e, 0)}
            ondragover={handleDragOver}
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
                class={`absolute top-1 h-7 bg-primary text-primary-foreground text-xs px-2 rounded flex items-center cursor-pointer select-none overflow-hidden transition-all hover:brightness-110 ${
                  $playbackStore.selectedTimelineClipId === timelineClip.id
                    ? 'ring-2 ring-ring shadow-md'
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
                <span class="truncate text-xs">
                  {getClipFilename(timelineClip.clipId)}
                </span>
              </div>
            {/each}
          </div>
        </div>

        <!-- Track 2 (Overlay/PiP) -->
        <div
          class="relative hover:bg-muted/50 transition-colors h-[45px] flex"
          role="region"
          aria-label="Timeline track 2"
        >
          <div class="w-[120px] px-2 py-1 bg-muted border-r text-xs font-semibold text-muted-foreground whitespace-nowrap flex items-center justify-center shrink-0">
            Track 2 (Overlay)
          </div>
          <div
            class="relative cursor-crosshair"
            style="width: {timelineWidth}px"
            onclick={(e) => handleTimelineClick(e, 1)}
            onkeydown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
              }
            }}
            ondrop={(e) => handleDrop(e, 1)}
            ondragover={handleDragOver}
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
                class={`absolute top-1 h-7 bg-accent text-accent-foreground text-xs px-2 rounded flex items-center cursor-pointer select-none overflow-hidden transition-all hover:brightness-110 ${
                  $playbackStore.selectedTimelineClipId === timelineClip.id
                    ? 'ring-2 ring-ring shadow-md'
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
                <span class="truncate text-xs">
                  {getClipFilename(timelineClip.clipId)}
                </span>
              </div>
            {/each}
          </div>
        </div>
      </div>
    </ScrollArea>
  </div>
</div>
