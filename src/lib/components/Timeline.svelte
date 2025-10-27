<script>
  import { timelineStore } from '../stores/timeline.js';
  import { playbackStore } from '../stores/playback.js';
  import { clipsStore } from '../stores/clips.js';
  import { Button } from "$lib/components/ui/button";
  import { Separator } from "$lib/components/ui/separator";

  /**
   * Timeline Component
   * Displays two-track timeline with playhead
   * Supports drag-and-drop clip placement and trimming
   */

  /** @type {HTMLElement | null} */
  let timelineElement = null;
  let zoom = 100; // pixels per second
  const MIN_ZOOM = 20;
  const MAX_ZOOM = 300;

  $: timelineWidth = Math.max($timelineStore.duration * zoom, 500);
  $: playheadPosition = $playbackStore.currentTime * zoom;

  /** @param {number} duration */
  function getTimeMarkers(duration) {
    const markers = [];
    const step = duration > 60 ? 10 : 5;
    for (let i = 0; i <= duration; i += step) {
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
  }

  /** @param {DragEvent} e
   *  @param {number} trackIndex */
  function handleDrop(e, trackIndex) {
    e.preventDefault();
    e.stopPropagation();

    if (!e.dataTransfer || !e.currentTarget) return;
    const data = JSON.parse(e.dataTransfer.getData('application/json'));
    const rect = /** @type {HTMLElement} */ (e.currentTarget).getBoundingClientRect();
    const dropX = e.clientX - rect.left;
    const startTime = Math.max(0, dropX / zoom);

    const timelineClip = {
      id: `clip-${Date.now()}-${Math.random()}`,
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
    <Button variant="ghost" size="sm" on:click={zoom_out} title="Zoom out" class="h-7 w-7 p-0">
      −
    </Button>
    <Button variant="ghost" size="sm" on:click={zoom_reset} title="Reset zoom" class="h-7 px-2">
      Reset
    </Button>
    <Button variant="ghost" size="sm" on:click={zoom_in} title="Zoom in" class="h-7 w-7 p-0">
      +
    </Button>
    <span class="text-xs text-muted-foreground ml-2 min-w-[30px]">{Math.round(zoom / 100 * 100)}%</span>
  </div>

  <!-- Timeline Container -->
  <div class="flex flex-col flex-1 bg-card border-t">
    <!-- Time Ruler -->
    <div class="relative h-6 bg-muted border-b overflow-hidden">
      <div style="width: {timelineWidth}px" class="relative h-full">
        {#each getTimeMarkers($timelineStore.duration) as time}
          <div style="left: {time * zoom}px" class="absolute text-[10px] text-muted-foreground border-l border-muted-foreground h-full pt-0.5">
            {formatTime(time)}
          </div>
        {/each}
      </div>
    </div>

    <!-- Timeline Tracks -->
    <div bind:this={timelineElement} class="flex-1 overflow-x-auto overflow-y-hidden relative">
      <div style="width: {timelineWidth}px; position: relative;" class="flex flex-col">
        <!-- Playhead -->
        <div
          class="absolute w-0.5 h-full bg-red-500 shadow-md z-50"
          style="left: {playheadPosition}px"
          title={formatTime($playbackStore.currentTime)}
        />

        <!-- Track 1 (Main Video) -->
        <div
          class="flex-1 relative border-b hover:bg-muted/50 transition-colors cursor-pointer"
          on:click={(e) => handleTimelineClick(e, 0)}
          on:drop={(e) => handleDrop(e, 0)}
          on:dragover={handleDragOver}
          role="button"
          tabindex="0"
        >
          <div class="absolute left-0 top-0 w-[120px] px-2 py-1 bg-muted border-r border-b text-xs font-semibold text-muted-foreground whitespace-nowrap z-40">
            Track 1 (Main)
          </div>
          <div class="relative ml-[120px] py-0.5">
            {#each getClipsForTrack(0) as timelineClip (timelineClip.id)}
              <div
                class={`absolute top-1 h-7 bg-green-600 text-white text-xs px-2 rounded flex items-center cursor-pointer select-none overflow-hidden transition-all hover:brightness-110 ${
                  $playbackStore.selectedTimelineClipId === timelineClip.id
                    ? 'ring-2 ring-primary shadow-md bg-primary'
                    : ''
                }`}
                style="
                  left: {timelineClip.startTime * zoom}px;
                  width: {(timelineClip.trimEnd - timelineClip.trimStart) * zoom}px;
                "
                on:click={() => selectTimelineClip(timelineClip.id)}
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
          class="flex-1 relative hover:bg-muted/50 transition-colors cursor-pointer"
          on:click={(e) => handleTimelineClick(e, 1)}
          on:drop={(e) => handleDrop(e, 1)}
          on:dragover={handleDragOver}
          role="button"
          tabindex="0"
        >
          <div class="absolute left-0 top-0 w-[120px] px-2 py-1 bg-muted border-r text-xs font-semibold text-muted-foreground whitespace-nowrap z-40">
            Track 2 (Overlay)
          </div>
          <div class="relative ml-[120px] py-0.5">
            {#each getClipsForTrack(1) as timelineClip (timelineClip.id)}
              <div
                class={`absolute top-1 h-7 bg-orange-600 text-white text-xs px-2 rounded flex items-center cursor-pointer select-none overflow-hidden transition-all hover:brightness-110 ${
                  $playbackStore.selectedTimelineClipId === timelineClip.id
                    ? 'ring-2 ring-primary shadow-md bg-primary'
                    : ''
                }`}
                style="
                  left: {timelineClip.startTime * zoom}px;
                  width: {(timelineClip.trimEnd - timelineClip.trimStart) * zoom}px;
                "
                on:click={() => selectTimelineClip(timelineClip.id)}
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
    </div>
  </div>
</div>
