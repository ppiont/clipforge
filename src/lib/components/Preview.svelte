<script>
  import { clipsStore } from '../stores/clips.js';
  import { playbackStore } from '../stores/playback.js';
  import { timelineStore } from '../stores/timeline.js';
  import { convertFileSrc } from '@tauri-apps/api/core';

  /**
   * Preview Component
   * Displays selected video clip with playback
   * Supports both timeline playback and media library preview
   */

  let {
    videoElement = $bindable(null)
  } = $props();
  let currentTime = $state(0);
  let duration = $state(0);
  let trimStart = $state(0);
  let trimEnd = $state(0);
  let currentlyPlayingClipId = $state(null);
  let syncFrameId = $state(null);

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

  // Determine which clip to display based on timeline state
  // Priority: Timeline playback > Selected timeline clip > Selected media library clip
  let activeTimelineClip = $derived.by(() => {
    // If we have timeline clips and playback is happening or playhead is >= 0, find clip at playhead
    if ($timelineStore.clips.length > 0 && ($playbackStore.isPlaying || $playbackStore.currentTime >= 0)) {
      // Find clip at current playhead position (prioritize track 0, main video)
      const clipAtPlayhead = $timelineStore.clips
        .filter(c => c.track === 0) // Only main track for now
        .find(c =>
          $playbackStore.currentTime >= c.startTime &&
          $playbackStore.currentTime < c.startTime + c.duration
        );

      if (clipAtPlayhead) {
        return clipAtPlayhead;
      }
    }

    // Fall back to selected timeline clip if available
    return selectedTimelineClip;
  });

  // Get the source clip from media library for the active timeline clip
  let displayClip = $derived.by(() => {
    if (activeTimelineClip) {
      return $clipsStore.find(c => c.id === activeTimelineClip.clipId);
    }
    return selectedClip;
  });

  // Convert file path to Tauri asset URL
  let videoSrc = $derived(
    displayClip ? convertFileSrc(displayClip.path) : ''
  );

  // Update trim range and video time when active clip changes
  $effect(() => {
    if (activeTimelineClip) {
      trimStart = activeTimelineClip.trimStart || 0;
      trimEnd = activeTimelineClip.trimEnd || (displayClip?.duration ?? 0);

      // Calculate offset within the clip
      const offsetInTimeline = $playbackStore.currentTime - activeTimelineClip.startTime;
      const videoTime = trimStart + offsetInTimeline;

      // Update video element to correct position within the trimmed range
      if (videoElement && displayClip && currentlyPlayingClipId !== activeTimelineClip.clipId) {
        // Source changed, need to reload
        currentlyPlayingClipId = activeTimelineClip.clipId;
        videoElement.currentTime = videoTime;
      } else if (videoElement) {
        // Same source, just sync time
        const timeDiff = Math.abs(videoElement.currentTime - videoTime);
        if (timeDiff > 0.1) { // Only update if difference is significant
          videoElement.currentTime = videoTime;
        }
      }
    } else if (selectedClip) {
      trimStart = 0;
      trimEnd = selectedClip.duration;
      currentlyPlayingClipId = selectedClip.id;
    } else {
      trimStart = 0;
      trimEnd = 0;
      currentlyPlayingClipId = null;
    }
  });

  /** @param {number} seconds */
  function formatTime(seconds) {
    const m = Math.floor(seconds / 60);
    const s = Math.floor(seconds % 60);
    return `${m}:${s.toString().padStart(2, '0')}`;
  }

  function onLoadedMetadata() {
    if (videoElement) {
      duration = videoElement.duration;
      trimEnd = duration;
    }
  }

  /**
   * Start RAF-based synchronization for smooth 60fps playhead updates
   */
  function startSync() {
    const sync = () => {
      if (videoElement && !videoElement.paused) {
        currentTime = videoElement.currentTime;

        // If we're playing from timeline, update timeline playhead position
        if (activeTimelineClip) {
          // Calculate timeline position from video position
          const timelinePosition = activeTimelineClip.startTime + (currentTime - trimStart);

          // Update both stores
          playbackStore.update(state => ({
            ...state,
            currentTime: timelinePosition
          }));
          timelineStore.update(state => ({
            ...state,
            playhead: timelinePosition
          }));

          // Check if we've reached the end of the trimmed clip
          if (currentTime >= trimEnd) {
            // Move to next clip or stop
            const nextClipStartTime = activeTimelineClip.startTime + activeTimelineClip.duration;
            const nextClip = $timelineStore.clips
              .filter(c => c.track === 0)
              .find(c => c.startTime >= nextClipStartTime);

            if (nextClip && $playbackStore.isPlaying) {
              // Jump to next clip
              playbackStore.update(state => ({
                ...state,
                currentTime: nextClip.startTime
              }));
            } else {
              // No next clip, pause
              stopSync();
              videoElement.pause();
              playbackStore.update(state => ({
                ...state,
                isPlaying: false
              }));
              return; // Don't schedule next frame
            }
          }
        } else {
          // Playing from media library, just update current time
          playbackStore.update(state => ({
            ...state,
            currentTime
          }));
          timelineStore.update(state => ({
            ...state,
            playhead: currentTime
          }));

          // Loop trim range if past trimEnd (for selected timeline clips)
          if (selectedTimelineClip && trimEnd > 0 && currentTime > trimEnd) {
            videoElement.currentTime = trimStart;
          }
        }

        syncFrameId = requestAnimationFrame(sync);
      }
    };
    sync();
  }

  /**
   * Stop RAF-based synchronization
   */
  function stopSync() {
    if (syncFrameId !== null) {
      cancelAnimationFrame(syncFrameId);
      syncFrameId = null;
    }
  }

  function onPlay() {
    playbackStore.update(state => ({
      ...state,
      isPlaying: true
    }));
    startSync();
  }

  function onPause() {
    playbackStore.update(state => ({
      ...state,
      isPlaying: false
    }));
    stopSync();
  }
</script>

<div class="relative w-full h-full bg-black rounded overflow-hidden flex items-center justify-center aspect-video">
  {#if displayClip && videoSrc}
    <video
      bind:this={videoElement}
      src={videoSrc}
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
  {:else}
    <div class="flex flex-col items-center justify-center w-full h-full text-gray-500 text-center">
      <p class="text-sm">No clip selected</p>
      <p class="text-xs text-gray-400 mt-1">Select a clip from the media library to preview</p>
    </div>
  {/if}
</div>
