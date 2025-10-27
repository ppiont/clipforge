<script>
  import { clipsStore } from '../stores/clips.js';
  import { playbackStore } from '../stores/playback.js';
  import { timelineStore } from '../stores/timeline.js';

  /**
   * Preview Component
   * Displays selected video clip with playback
   */

  let {
    videoElement = $bindable(null)
  } = $props();
  let currentTime = $state(0);
  let duration = $state(0);
  let trimStart = $state(0);
  let trimEnd = $state(0);

  // Get selected clip from media library
  let selectedClip = $derived(
    /** @type {{id: string; filename: string; path: string; duration: number; resolution: string} | undefined} */
    ($clipsStore.find(
      /** @param {{id: string; filename: string; path: string; duration: number; resolution: string}} c */
      c => c.id === $playbackStore.selectedClipId
    ))
  );

  // Get selected timeline clip and its trim data
  let selectedTimelineClip = $derived(
    /** @type {{id: string; clipId: string; track: number; startTime: number; trimStart: number; trimEnd: number; duration: number} | undefined} */
    ($timelineStore.clips.find(
      /** @param {{id: string; clipId: string; track: number; startTime: number; trimStart: number; trimEnd: number; duration: number}} c */
      c => c.id === $playbackStore.selectedTimelineClipId
    ))
  );

  // Determine which clip to display (use timeline clip if available, else media library clip)
  let displayClip = $derived(selectedClip);

  // Update trim range when timeline clip selected
  $effect(() => {
    if (selectedTimelineClip) {
      trimStart = selectedTimelineClip.trimStart || 0;
      trimEnd = selectedTimelineClip.trimEnd || duration;
    } else {
      trimStart = 0;
      trimEnd = duration;
    }
  });

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

<div class="relative w-full h-full bg-black rounded overflow-hidden flex items-center justify-center aspect-video">
  {#if displayClip}
    <video
      bind:this={videoElement}
      src={displayClip.path}
      ontimeupdate={onTimeUpdate}
      onloadedmetadata={onLoadedMetadata}
      onplay={onPlay}
      onpause={onPause}
      width="640"
      height="360"
      class="w-full h-full object-contain"
      controls
    >
      <track kind="captions" />
    </video>
    <div class="absolute bottom-2 right-2 bg-black/70 text-white text-xs px-2 py-1 rounded font-mono">
      {formatTime(currentTime)} / {formatTime(duration)}
    </div>
  {:else}
    <div class="flex flex-col items-center justify-center w-full h-full text-gray-500 text-center">
      <p class="text-sm">No clip selected</p>
      <p class="text-xs text-gray-400 mt-1">Select a clip from the media library to preview</p>
    </div>
  {/if}
</div>
