<script>
  import { timelineStore } from '../stores/timeline.js';
  import { playbackStore } from '../stores/playback.js';
  import { clipsStore } from '../stores/clips.js';

  /**
   * Timeline Component
   * Displays two-track timeline with playhead
   * Supports drag-and-drop clip placement and trimming
   */

  let timelineElement;
  let zoom = 100; // pixels per second
  const MIN_ZOOM = 20;
  const MAX_ZOOM = 300;

  $: timelineWidth = Math.max($timelineStore.duration * zoom, 500);
  $: playheadPosition = $playbackStore.currentTime * zoom;

  function getTimeMarkers(duration) {
    const markers = [];
    const step = duration > 60 ? 10 : 5;
    for (let i = 0; i <= duration; i += step) {
      markers.push(i);
    }
    return markers;
  }

  function formatTime(seconds) {
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = Math.floor(seconds % 60);
    if (h > 0) {
      return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
    }
    return `${m}:${s.toString().padStart(2, '0')}`;
  }

  function handleTimelineClick(e, trackIndex) {
    const rect = timelineElement.getBoundingClientRect();
    const clickX = e.clientX - rect.left;
    const newTime = clickX / zoom;

    playbackStore.update(state => ({
      ...state,
      currentTime: newTime
    }));
  }

  function handleDrop(e, trackIndex) {
    e.preventDefault();
    e.stopPropagation();

    const data = JSON.parse(e.dataTransfer.getData('application/json'));
    const rect = e.currentTarget.getBoundingClientRect();
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

  function handleDragOver(e) {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'copy';
  }

  function selectTimelineClip(clipId) {
    playbackStore.update(state => ({
      ...state,
      selectedTimelineClipId: clipId,
      selectedClipId: null
    }));
  }

  function getClipsForTrack(trackIndex) {
    return $timelineStore.clips.filter(c => c.track === trackIndex);
  }

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

<div class="timeline-wrapper">
  <div class="timeline-controls">
    <button on:click={zoom_out} title="Zoom out">−</button>
    <button on:click={zoom_reset} title="Reset zoom">Reset</button>
    <button on:click={zoom_in} title="Zoom in">+</button>
    <span class="zoom-label">{Math.round(zoom / 100 * 100)}%</span>
  </div>

  <div class="timeline-container">
    <!-- Time Ruler -->
    <div class="time-ruler">
      <div class="ruler-content" style="width: {timelineWidth}px">
        {#each getTimeMarkers($timelineStore.duration) as time}
          <div class="marker" style="left: {time * zoom}px">
            {formatTime(time)}
          </div>
        {/each}
      </div>
    </div>

    <!-- Timeline Tracks -->
    <div bind:this={timelineElement} class="timeline-body">
      <div class="timeline-scroll" style="width: {timelineWidth}px; position: relative;">
        <!-- Playhead -->
        <div
          class="playhead"
          style="left: {playheadPosition}px"
          title={formatTime($playbackStore.currentTime)}
        />

        <!-- Track 1 (Main Video) -->
        <div
          class="track"
          on:click={(e) => handleTimelineClick(e, 0)}
          on:drop={(e) => handleDrop(e, 0)}
          on:dragover={handleDragOver}
        >
          <div class="track-label">Track 1 (Main)</div>
          <div class="clips-area">
            {#each getClipsForTrack(0) as timelineClip (timelineClip.id)}
              <div
                class="timeline-clip"
                class:selected={$playbackStore.selectedTimelineClipId === timelineClip.id}
                style="
                  left: {timelineClip.startTime * zoom}px;
                  width: {(timelineClip.trimEnd - timelineClip.trimStart) * zoom}px;
                "
                on:click={() => selectTimelineClip(timelineClip.id)}
              >
                <span class="clip-text">
                  {getClipFilename(timelineClip.clipId)}
                </span>
              </div>
            {/each}
          </div>
        </div>

        <!-- Track 2 (Overlay/PiP) -->
        <div
          class="track"
          on:click={(e) => handleTimelineClick(e, 1)}
          on:drop={(e) => handleDrop(e, 1)}
          on:dragover={handleDragOver}
        >
          <div class="track-label">Track 2 (Overlay)</div>
          <div class="clips-area">
            {#each getClipsForTrack(1) as timelineClip (timelineClip.id)}
              <div
                class="timeline-clip overlay"
                class:selected={$playbackStore.selectedTimelineClipId === timelineClip.id}
                style="
                  left: {timelineClip.startTime * zoom}px;
                  width: {(timelineClip.trimEnd - timelineClip.trimStart) * zoom}px;
                "
                on:click={() => selectTimelineClip(timelineClip.id)}
              >
                <span class="clip-text">
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

<style>
  .timeline-wrapper {
    display: flex;
    flex-direction: column;
    height: 150px;
    background: #fafafa;
    border-top: 1px solid #ddd;
  }

  .timeline-controls {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 5px 10px;
    background: #f5f5f5;
    border-bottom: 1px solid #ddd;
    height: 35px;
  }

  .timeline-controls button {
    padding: 4px 8px;
    font-size: 12px;
    background: #fff;
    border: 1px solid #ddd;
    border-radius: 3px;
    cursor: pointer;
  }

  .timeline-controls button:hover {
    background: #f0f0f0;
  }

  .zoom-label {
    font-size: 11px;
    color: #666;
    margin-left: 5px;
    min-width: 30px;
  }

  .timeline-container {
    display: flex;
    flex-direction: column;
    flex: 1;
    background: #fff;
    border-top: 1px solid #ddd;
  }

  .time-ruler {
    height: 25px;
    background: #f0f0f0;
    border-bottom: 1px solid #ddd;
    position: relative;
    overflow: hidden;
  }

  .ruler-content {
    position: relative;
    height: 100%;
  }

  .marker {
    position: absolute;
    font-size: 10px;
    color: #666;
    padding: 3px 0 0 0;
    border-left: 1px solid #ddd;
    height: 100%;
  }

  .timeline-body {
    flex: 1;
    overflow-x: auto;
    overflow-y: hidden;
    position: relative;
  }

  .timeline-scroll {
    position: relative;
    display: flex;
    flex-direction: column;
  }

  .playhead {
    position: absolute;
    width: 2px;
    height: 100%;
    background: #ff4444;
    z-index: 100;
    box-shadow: 0 0 4px rgba(255, 68, 68, 0.5);
    top: 0;
  }

  .track {
    flex: 1;
    position: relative;
    border-bottom: 1px solid #ddd;
    background: #fafafa;
    display: flex;
    align-items: stretch;
  }

  .track-label {
    position: absolute;
    left: 0;
    top: 0;
    width: 120px;
    padding: 5px 8px;
    font-size: 11px;
    font-weight: 600;
    color: #666;
    background: #f5f5f5;
    border-right: 1px solid #ddd;
    white-space: nowrap;
    z-index: 50;
  }

  .clips-area {
    position: relative;
    flex: 1;
    margin-left: 120px;
    padding: 2px 0;
  }

  .timeline-clip {
    position: absolute;
    top: 3px;
    height: calc(100% - 6px);
    background: #4CAF50;
    color: white;
    padding: 4px 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid #388e3c;
    border-radius: 2px;
    cursor: pointer;
    user-select: none;
    overflow: hidden;
    transition: all 0.1s;
  }

  .timeline-clip:hover {
    filter: brightness(1.1);
  }

  .timeline-clip.selected {
    border: 2px solid #fff;
    box-shadow: 0 0 0 1px #1976d2;
    background: #1976d2;
  }

  .timeline-clip.overlay {
    background: #ff9800;
    border-color: #e65100;
  }

  .timeline-clip.overlay.selected {
    background: #ff6f00;
    box-shadow: 0 0 0 1px #ff6f00;
  }

  .clip-text {
    font-size: 11px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
