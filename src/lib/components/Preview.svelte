<script>
  import { clipsStore } from '../stores/clips.js';
  import { playbackStore } from '../stores/playback.js';
  import { timelineStore } from '../stores/timeline.js';

  /**
   * Preview Component
   * Displays selected video clip with playback
   */

  /** @type {HTMLVideoElement | null} */
  export let videoElement = null;
  let currentTime = 0;
  let duration = 0;
  let trimStart = 0;
  let trimEnd = 0;

  // Get selected clip from media library
  $: selectedClip = $clipsStore.find(c => c.id === $playbackStore.selectedClipId);

  // Get selected timeline clip and its trim data
  $: selectedTimelineClip = $timelineStore.clips.find(
    c => c.id === $playbackStore.selectedTimelineClipId
  );

  // Determine which clip to display (use timeline clip if available, else media library clip)
  $: displayClip = selectedClip;

  // Update trim range when timeline clip selected
  $: {
    if (selectedTimelineClip) {
      trimStart = selectedTimelineClip.trimStart || 0;
      trimEnd = selectedTimelineClip.trimEnd || duration;
    } else {
      trimStart = 0;
      trimEnd = duration;
    }
  }

  /** @param {number} seconds */
  function formatTime(seconds) {
    const m = Math.floor(seconds / 60);
    const s = Math.floor(seconds % 60);
    return `${m}:${s.toString().padStart(2, '0')}`;
  }

  function onTimeUpdate() {
    if (videoElement) {
      currentTime = videoElement.currentTime;

      // Update playhead in stores
      playbackStore.update(state => ({
        ...state,
        currentTime
      }));
      timelineStore.update(state => ({
        ...state,
        playhead: currentTime
      }));

      // Loop trim range if past trimEnd
      if (trimEnd > 0 && currentTime > trimEnd) {
        videoElement.currentTime = trimStart;
      }
    }
  }

  function onLoadedMetadata() {
    if (videoElement) {
      duration = videoElement.duration;
      trimEnd = duration;
    }
  }

  function onPlay() {
    playbackStore.update(state => ({
      ...state,
      isPlaying: true
    }));
  }

  function onPause() {
    playbackStore.update(state => ({
      ...state,
      isPlaying: false
    }));
  }
</script>

<div class="preview-container">
  {#if displayClip}
    <video
      bind:this={videoElement}
      src={displayClip.path}
      on:timeupdate={onTimeUpdate}
      on:loadedmetadata={onLoadedMetadata}
      on:play={onPlay}
      on:pause={onPause}
      width="640"
      height="360"
      controls
    />
    <div class="time-display">
      {formatTime(currentTime)} / {formatTime(duration)}
    </div>
  {:else}
    <div class="placeholder">
      <p>No clip selected</p>
      <p style="font-size: 12px; color: #999;">Select a clip from the media library to preview</p>
    </div>
  {/if}
</div>

<style>
  .preview-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    background: #000;
    width: 100%;
    height: 100%;
    aspect-ratio: 16 / 9;
    position: relative;
    border-radius: 4px;
    overflow: hidden;
  }

  video {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  .placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    color: #666;
    text-align: center;
  }

  .time-display {
    position: absolute;
    bottom: 10px;
    right: 10px;
    color: #fff;
    background: rgba(0, 0, 0, 0.7);
    padding: 5px 10px;
    border-radius: 4px;
    font-size: 12px;
    font-family: monospace;
  }
</style>
