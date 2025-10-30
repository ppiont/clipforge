<script>
  import { onMount } from 'svelte';
  import { playbackStore } from '../stores/playback.js';
  import { timelineStore } from '../stores/timeline.js';
  import { Button } from "$lib/components/ui/button";
  import { Play, Pause, Square, Scissors, Trash2 } from "@lucide/svelte";

  /**
   * Controls Component
   * Playback controls and timeline editing buttons
   */

  let {
    videoElement = $bindable(null)
  } = $props();

  /**
   * Check if split operation is available
   * Split is enabled when:
   * - A timeline clip is selected
   * - Playhead is within the selected clip bounds (not at edges)
   */
  let canSplit = $derived.by(() => {
    const selectedId = $playbackStore.selectedTimelineClipId;
    const playheadTime = $playbackStore.currentTime;

    if (!selectedId) return false;

    const clip = $timelineStore.clips.find(c => c.id === selectedId);
    if (!clip) return false;

    const clipEnd = clip.startTime + clip.duration;

    // Check if playhead is within clip bounds (with 0.1s margin from edges)
    return playheadTime > clip.startTime + 0.1 && playheadTime < clipEnd - 0.1;
  });

  /**
   * Check if delete operation is available
   */
  let canDelete = $derived.by(() => {
    return $playbackStore.selectedTimelineClipId !== null;
  });

  function togglePlayPause() {
    if (!videoElement) {
      console.error("No video element found");
      return;
    }

    console.log("Toggle play/pause, current state:", $playbackStore.isPlaying);

    if ($playbackStore.isPlaying) {
      videoElement.pause();
      playbackStore.update(state => ({ ...state, isPlaying: false }));
    } else {
      videoElement.play().catch((/** @type {any} */ err) => {
        console.error("Error playing video:", err);
      });
      playbackStore.update(state => ({ ...state, isPlaying: true }));
    }
  }

  function handleStop() {
    if (!videoElement) return;
    videoElement.pause();
    videoElement.currentTime = 0;
    playbackStore.update(state => ({
      ...state,
      isPlaying: false,
      currentTime: 0
    }));
  }

  /**
   * Handle split clip button click
   * Dispatches a custom 'split-clip' event to the window
   */
  function handleSplitClick() {
    // Dispatch custom event that Timeline will listen to
    window.dispatchEvent(new CustomEvent('split-clip'));
  }

  /**
   * Handle delete clip button click
   * Dispatches a custom 'delete-clip' event to the window
   */
  function handleDeleteClick() {
    // Dispatch custom event that Timeline will listen to
    window.dispatchEvent(new CustomEvent('delete-clip'));
  }

  /** @param {KeyboardEvent} e */
  function handleKeyPress(e) {
    // Don't process shortcuts if user is typing in an input
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
      return;
    }

    // Space: Play/Pause
    if (e.code === 'Space') {
      e.preventDefault();
      togglePlayPause();
      return;
    }

    // K: Pause (professional editing standard)
    if (e.key === 'k' || e.key === 'K') {
      e.preventDefault();
      if (videoElement && !videoElement.paused) {
        videoElement.pause();
        playbackStore.update(state => ({ ...state, isPlaying: false }));
      }
      return;
    }

    // J: Rewind/Play Backward (for now, just seek backward 1 second)
    if (e.key === 'j' || e.key === 'J') {
      e.preventDefault();
      if (videoElement) {
        const newTime = Math.max(0, $playbackStore.currentTime - 1);
        playbackStore.update(state => ({ ...state, currentTime: newTime }));
        videoElement.currentTime = newTime;
      }
      return;
    }

    // L: Play Forward (for now, play or seek forward 1 second)
    if (e.key === 'l' || e.key === 'L') {
      e.preventDefault();
      if (videoElement) {
        if (videoElement.paused) {
          videoElement.play();
          playbackStore.update(state => ({ ...state, isPlaying: true }));
        } else {
          const newTime = Math.min(videoElement.duration || 0, $playbackStore.currentTime + 1);
          playbackStore.update(state => ({ ...state, currentTime: newTime }));
          videoElement.currentTime = newTime;
        }
      }
      return;
    }

    // Left Arrow: Frame backward (0.1 second)
    if (e.key === 'ArrowLeft') {
      e.preventDefault();
      if (videoElement) {
        const newTime = Math.max(0, $playbackStore.currentTime - 0.1);
        playbackStore.update(state => ({ ...state, currentTime: newTime }));
        videoElement.currentTime = newTime;
      }
      return;
    }

    // Right Arrow: Frame forward (0.1 second)
    if (e.key === 'ArrowRight') {
      e.preventDefault();
      if (videoElement) {
        const newTime = Math.min(videoElement.duration || 0, $playbackStore.currentTime + 0.1);
        playbackStore.update(state => ({ ...state, currentTime: newTime }));
        videoElement.currentTime = newTime;
      }
      return;
    }
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeyPress);
    return () => {
      window.removeEventListener('keydown', handleKeyPress);
    };
  });
</script>

<footer class="flex items-center gap-4 h-[50px] px-4 bg-background border-t">
  <div class="flex gap-2">
    <Button
      variant="default"
      size="sm"
      class="active:scale-95 transition-transform"
      onclick={togglePlayPause}
      title="Play/Pause (Space)"
    >
      {#if $playbackStore.isPlaying}
        <Pause />
        Pause
      {:else}
        <Play />
        Play
      {/if}
    </Button>

    <Button
      variant="outline"
      size="sm"
      class="active:scale-95 transition-transform"
      onclick={handleStop}
      title="Stop"
    >
      <Square />
      Stop
    </Button>
  </div>

  <div class="flex gap-2">
    <Button
      variant="outline"
      size="sm"
      disabled={!canSplit}
      class={canSplit ? "active:scale-95 transition-transform" : "cursor-not-allowed"}
      onclick={handleSplitClick}
      title={canSplit ? "Split clip at playhead (S)" : "Move playhead within a selected clip to split"}
    >
      <Scissors />
      Split
    </Button>
    <Button
      variant="outline"
      size="sm"
      disabled={!canDelete}
      class={canDelete ? "active:scale-95 transition-transform" : "cursor-not-allowed"}
      onclick={handleDeleteClick}
      title={canDelete ? "Delete selected clip (Delete/Backspace)" : "Select a clip to delete"}
    >
      <Trash2 />
      Delete
    </Button>
  </div>

  <div class="flex-1"></div>

  <div class="flex items-center gap-2 px-3 py-1 bg-muted rounded-md text-xs text-muted-foreground font-medium">
    {#if $playbackStore.isPlaying}
      <Play class="w-3 h-3" />
      Playing
    {:else}
      <Pause class="w-3 h-3" />
      Paused
    {/if}
  </div>
</footer>
